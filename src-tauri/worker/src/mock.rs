use crate::hardware_access::{AccessError, HardwareAccess};

pub const MSR_TEMPERATURE_TARGET: u64 = 0x1A2;
pub const MSR_THERM_STATUS: u64 = 0x19C;
pub const MSR_PACKAGE_THERM_STATUS: u64 = 0x1B1;

#[derive(Debug, Clone)]
pub struct MockHardware {
    open_ok: bool,
    tj_max: u8,
    per_logical_delta: Vec<u8>,
    package_delta: u8,
}

impl MockHardware {
    pub fn new(tj_max: u8, per_logical_delta: Vec<u8>, package_delta: u8) -> Self {
        Self {
            open_ok: true,
            tj_max,
            per_logical_delta,
            package_delta,
        }
    }

    pub fn unavailable(mut self) -> Self {
        self.open_ok = false;
        self
    }
}

impl HardwareAccess for MockHardware {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn open(&mut self) -> Result<(), AccessError> {
        if self.open_ok {
            Ok(())
        } else {
            Err(AccessError::DriverUnavailable)
        }
    }

    fn is_available(&self) -> bool {
        self.open_ok
    }

    fn read_msr(&mut self, msr: u64, logical_cpu: usize) -> Result<u64, AccessError> {
        if !self.open_ok {
            return Err(AccessError::DriverUnavailable);
        }
        match msr {
            MSR_TEMPERATURE_TARGET => Ok((self.tj_max as u64) << 16),
            MSR_THERM_STATUS => {
                let delta = *self.per_logical_delta.get(logical_cpu).ok_or(AccessError::MsrFailed)?;
                Ok(0x8000_0000u64 | ((delta as u64) << 16))
            }
            MSR_PACKAGE_THERM_STATUS => Ok(0x8000_0000u64 | ((self.package_delta as u64) << 16)),
            _ => Err(AccessError::MsrFailed),
        }
    }

    fn close(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_reads_expected_msrs() {
        let mut m = MockHardware::new(100, vec![20, 20, 22, 25, 20, 20, 22, 25], 26);
        m.open().unwrap();
        assert_eq!(m.read_msr(MSR_TEMPERATURE_TARGET, 0).unwrap(), 100 << 16);
        assert_eq!(m.read_msr(MSR_THERM_STATUS, 3).unwrap(), 0x8000_0000 | (25 << 16));
        assert_eq!(m.read_msr(MSR_PACKAGE_THERM_STATUS, 0).unwrap(), 0x8000_0000 | (26 << 16));
    }

    #[test]
    fn mock_rejects_unknown_msr() {
        let mut m = MockHardware::new(100, vec![20], 26);
        m.open().unwrap();
        assert_eq!(m.read_msr(0xDEAD, 0).unwrap_err(), AccessError::MsrFailed);
    }

    #[test]
    fn mock_unavailable() {
        let mut m = MockHardware::new(100, vec![20], 26).unavailable();
        assert_eq!(m.open().unwrap_err(), AccessError::DriverUnavailable);
        assert!(!m.is_available());
    }
}
