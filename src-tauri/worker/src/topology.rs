use crate::hardware_access::AccessError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreType {
    Performance,
    Efficiency,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuThread {
    pub logical_id: usize,
    pub physical_core_id: usize,
    pub package_id: usize,
    pub core_type: CoreType,
}

#[derive(Debug, Clone)]
pub struct Topology {
    pub vendor: String,
    pub family: u32,
    pub model: u32,
    pub package_count: usize,
    pub threads: Vec<CpuThread>,
    /// One logical processor per physical core, used as the affinity target for
    /// per-core MSR reads.
    pub core_first_logical: Vec<usize>,
}

impl Topology {
    pub fn detect() -> Result<Self, AccessError> {
        let (vendor, family, model, hybrid) = cpu_signature();
        let threads = enumerate_threads(hybrid)?;
        if threads.is_empty() {
            return Err(AccessError::TopologyFailed);
        }

        let package_count = threads
            .iter()
            .map(|t| t.package_id)
            .max()
            .map(|m| m + 1)
            .unwrap_or(1);

        let mut core_first_logical: Vec<usize> = Vec::new();
        for core_id in 0..(threads.iter().map(|t| t.physical_core_id).max().unwrap_or(0) + 1) {
            if let Some(t) = threads.iter().find(|t| t.physical_core_id == core_id) {
                core_first_logical.push(t.logical_id);
            }
        }

        Ok(Self {
            vendor,
            family,
            model,
            package_count,
            threads,
            core_first_logical,
        })
    }
}

fn cpu_signature() -> (String, u32, u32, bool) {
    #[cfg(target_arch = "x86_64")]
    {
        use std::arch::x86_64::__cpuid;
        let eax0 = __cpuid(0);
        let vendor: String = {
            let mut b = [0u8; 12];
            b[0..4].copy_from_slice(&eax0.ebx.to_le_bytes());
            b[4..8].copy_from_slice(&eax0.edx.to_le_bytes());
            b[8..12].copy_from_slice(&eax0.ecx.to_le_bytes());
            b.iter().map(|c| *c as char).collect()
        };

        let eax1 = __cpuid(1);
        let mut family = (eax1.eax >> 8) & 0xF;
        let mut model = (eax1.eax >> 4) & 0xF;
        if family == 0xF {
            family += (eax1.eax >> 20) & 0xFF;
        }
        if family == 0x6 || family == 0xF {
            model += ((eax1.eax >> 16) & 0xF) << 4;
        }

        let hybrid = __cpuid(7).edx & (1 << 15) != 0;
        (vendor, family, model, hybrid)
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        (String::new(), 0, 0, false)
    }
}

#[cfg(target_arch = "x86_64")]
fn enumerate_threads(hybrid: bool) -> Result<Vec<CpuThread>, AccessError> {
    use windows_sys::Win32::System::SystemInformation::{
        GetLogicalProcessorInformationEx, LOGICAL_PROCESSOR_RELATIONSHIP, SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
        RelationProcessorCore, RelationProcessorPackage,
    };

    let rel_all: LOGICAL_PROCESSOR_RELATIONSHIP = 0xFFFF;
    let mut len = 0u32;
    unsafe {
        GetLogicalProcessorInformationEx(rel_all, std::ptr::null_mut(), &mut len);
    }
    let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
    if err != 122 && err != 0 {
        // ERROR_INSUFFICIENT_BUFFER
        return Err(AccessError::TopologyFailed);
    }

    let mut buf = vec![0u8; len as usize];
    let ok = unsafe {
        GetLogicalProcessorInformationEx(rel_all, buf.as_mut_ptr() as *mut _, &mut len)
    };
    if ok == 0 {
        return Err(AccessError::TopologyFailed);
    }

    // Collect package groups and core groups.
    let mut package_assign: Vec<(usize, Vec<usize>)> = Vec::new(); // (pkg id, logical ids)
    let mut core_assign: Vec<(usize, Vec<usize>)> = Vec::new(); // (core id, logical ids)
    let mut cpu_types: std::collections::HashMap<usize, u8> = std::collections::HashMap::new();

    let mut offset = 0usize;
    while offset < buf.len() {
        let info = unsafe { &*(buf.as_ptr().add(offset) as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX) };
        if info.Relationship == RelationProcessorPackage {
            let proc = unsafe { &info.Anonymous.Processor };
            let mut logicals = Vec::new();
            for g in 0..proc.GroupCount as usize {
                let group = proc.GroupMask[g].Group as usize;
                let mask = proc.GroupMask[g].Mask as usize;
                for bit in 0..64 {
                    if mask & (1usize << bit) != 0 {
                        logicals.push(group * 64 + bit);
                    }
                }
            }
            package_assign.push((package_assign.len(), logicals));
        } else if info.Relationship == RelationProcessorCore {
            let proc = unsafe { &info.Anonymous.Processor };
            let mut logicals = Vec::new();
            for g in 0..proc.GroupCount as usize {
                let group = proc.GroupMask[g].Group as usize;
                let mask = proc.GroupMask[g].Mask as usize;
                for bit in 0..64 {
                    if mask & (1usize << bit) != 0 {
                        logicals.push(group * 64 + bit);
                    }
                }
            }
            let core_id = core_assign.len();
            if hybrid {
                for &l in &logicals {
                    cpu_types.insert(l, proc.EfficiencyClass);
                }
            }
            core_assign.push((core_id, logicals));
        }
        if info.Size == 0 {
            break;
        }
        offset += info.Size as usize;
    }

    let total_logical = core_assign.iter().flat_map(|(_, l)| l.iter()).max().map(|m| m + 1).unwrap_or(0);

    let mut threads = Vec::new();
    for logical_id in 0..total_logical {
        let core_id = core_assign
            .iter()
            .find(|(_, l)| l.contains(&logical_id))
            .map(|(id, _)| *id)
            .unwrap_or(0);
        let package_id = package_assign
            .iter()
            .find(|(_, l)| l.contains(&logical_id))
            .map(|(id, _)| *id)
            .unwrap_or(0);
        let core_type = if hybrid {
            match cpu_types.get(&logical_id).copied().unwrap_or(0) {
                0 => CoreType::Performance,
                _ => CoreType::Efficiency,
            }
        } else {
            CoreType::Unknown
        };
        threads.push(CpuThread {
            logical_id,
            physical_core_id: core_id,
            package_id,
            core_type,
        });
    }

    Ok(threads)
}

#[cfg(not(target_arch = "x86_64"))]
fn enumerate_threads(_hybrid: bool) -> Result<Vec<CpuThread>, AccessError> {
    Err(AccessError::TopologyFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_is_wellformed() {
        let (vendor, family, model, _hybrid) = cpu_signature();
        assert!(!vendor.is_empty());
        assert!(family > 0);
        assert!(model > 0);
    }

    #[test]
    fn detect_returns_threads() {
        let topo = Topology::detect().unwrap();
        assert!(!topo.threads.is_empty());
        assert_eq!(topo.core_first_logical.len(), topo.threads.iter().map(|t| t.physical_core_id).max().unwrap() + 1);
        assert_eq!(topo.threads.len(), topo.threads.iter().map(|t| t.logical_id).max().unwrap() + 1);
        // SMT: two threads share a core
        let c0: Vec<&CpuThread> = topo.threads.iter().filter(|t| t.physical_core_id == 0).collect();
        assert!(!c0.is_empty());
    }
}

