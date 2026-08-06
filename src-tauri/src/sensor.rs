use serde::{Deserialize, Serialize};
use sysinfo::{Components, Disks, ProcessesToUpdate, System, MINIMUM_CPU_UPDATE_INTERVAL};

use zenith_sensor_worker::protocol::{SensorKind, Source, Status, WorkerMessage};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessInfo {
    pub pid: usize,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_gb: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SensorPayload {
    pub cpu_pkg_temp: f32,
    pub cpu_usage: f32,
    pub core_delta: f32,
    pub temps: Vec<f32>,
    pub ram_usage: f32,
    pub ram_total: f32,
    pub disk_usage: f32,
    pub disk_total: f32,
    pub top_processes: Vec<ProcessInfo>,
    pub source: String,
    pub status: String,
    pub worker_error: Option<String>,
}

/// Refresh processes only every N polls to reduce syscall overhead.
const PROCESS_REFRESH_INTERVAL: u32 = 5;

pub struct SensorEngine {
    sys: System,
    components: Components,
    disks: Disks,
    poll_count: u32,
}

impl SensorEngine {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_cpu_all();
        sys.refresh_memory();

        // Warm up CPU + process usage samples so the very first emitted payload
        // is accurate. sysinfo computes usage from the difference between two
        // refreshes and requires at least MINIMUM_CPU_UPDATE_INTERVAL between them.
        sys.refresh_processes(ProcessesToUpdate::All, true);
        std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_cpu_all();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let components = Components::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();

        Self {
            sys,
            components,
            disks,
            poll_count: 0,
        }
    }

    pub fn poll(
        &mut self,
        worker_latest: &Option<WorkerMessage>,
        worker_error: Option<&str>,
    ) -> SensorPayload {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();

        // Refresh processes less frequently to reduce overhead. The warmup in
        // `new()` covers poll #1, so the first throttled refresh lands on poll #2
        // (a full second later), guaranteeing a valid usage sample.
        self.poll_count += 1;
        if self.poll_count % PROCESS_REFRESH_INTERVAL == 2 {
            self.sys.refresh_processes(ProcessesToUpdate::All, true);
        }

        let global_cpu = self.sys.global_cpu_usage();

        // ── Temperature source: worker (hardware) or labeled ACPI fallback ──
        let (source, status, pkg_temp, core_temps, worker_error) =
            self.read_temps(worker_latest, worker_error);

        // ── Core delta (single pass for min & max) ──
        let core_delta = if core_temps.len() > 1 {
            let mut max_temp = f32::NEG_INFINITY;
            let mut min_temp = f32::INFINITY;
            for &t in &core_temps {
                if t > max_temp { max_temp = t; }
                if t < min_temp { min_temp = t; }
            }
            max_temp - min_temp
        } else {
            0.0
        };

        // ── Memory ──
        let ram_total = self.sys.total_memory() as f32 / 1_073_741_824.0;
        let ram_usage = self.sys.used_memory() as f32 / 1_073_741_824.0;

        // ── Disk (single pass) ──
        let mut disk_total_bytes: u64 = 0;
        let mut disk_avail_bytes: u64 = 0;
        for disk in self.disks.iter() {
            disk_total_bytes += disk.total_space();
            disk_avail_bytes += disk.available_space();
        }
        let disk_total = disk_total_bytes as f64 / 1_073_741_824.0;
        let disk_usage = (disk_total_bytes.saturating_sub(disk_avail_bytes)) as f64 / 1_073_741_824.0;

        // ── Top processes by CPU ──
        // sysinfo reports per-core CPU% (a multi-threaded process can exceed
        // 100%). Normalize to a 0-100% share of total CPU so it matches the
        // global CPU gauge and Task Manager conventions.
        let num_cpus = self.sys.cpus().len().max(1) as f32;
        let mut processes: Vec<ProcessInfo> = self
            .sys
            .processes()
            .iter()
            .map(|(pid, p)| {
                let name = p.name().to_string_lossy().into_owned();
                ProcessInfo {
                    pid: pid.as_u32() as usize,
                    name: if name.is_empty() { "Unknown".into() } else { name },
                    cpu_usage: (p.cpu_usage() / num_cpus).clamp(0.0, 100.0),
                    memory_gb: p.memory() as f32 / 1_073_741_824.0f32,
                }
            })
            .collect();

        processes.sort_unstable_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        processes.truncate(5);

        SensorPayload {
            cpu_pkg_temp: pkg_temp,
            cpu_usage: global_cpu,
            core_delta,
            temps: core_temps,
            ram_usage,
            ram_total,
            disk_usage: disk_usage as f32,
            disk_total: disk_total as f32,
            top_processes: processes,
            source,
            status,
            worker_error,
        }
    }

    /// Returns `(source, status, pkg_temp, core_temps, worker_error)`.
    fn read_temps(
        &mut self,
        worker_latest: &Option<WorkerMessage>,
        worker_error: Option<&str>,
    ) -> (String, String, f32, Vec<f32>, Option<String>) {
        // Fast path: a healthy worker sample is the real source, so skip the
        // ACPI component refresh that would be discarded by `merge_temps`.
        if matches!(worker_latest, Some(WorkerMessage::Sample(s)) if s.status == Status::Healthy) {
            return merge_temps(worker_latest, (0.0, Vec::new()), worker_error);
        }
        let acpi = self.read_acpi_temps();
        merge_temps(worker_latest, acpi, worker_error)
    }

    fn read_acpi_temps(&mut self) -> (f32, Vec<f32>) {
        self.components.refresh(false);
        let mut core_temps: Vec<f32> = Vec::new();
        let mut pkg_temp: f32 = 0.0;

        for component in self.components.iter() {
            let label = component.label().to_lowercase();
            let Some(temp) = component.temperature() else { continue };
            if temp <= 0.0 {
                continue;
            }
            if label == "computer" || label.contains("package") || label.contains("tctl") {
                pkg_temp = temp;
            } else if !label.contains("gpu") && (label.contains("core") || label.starts_with("core "))
            {
                core_temps.push(temp);
            }
        }

        if pkg_temp == 0.0 && !core_temps.is_empty() {
            pkg_temp = core_temps.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        }

        (pkg_temp, core_temps)
    }
}

/// Pure temperature-source merge. `acpi` is `(pkg_temp, core_temps)` from the
/// labeled ACPI fallback, if any.
fn merge_temps(
    worker_latest: &Option<WorkerMessage>,
    acpi: (f32, Vec<f32>),
    worker_error: Option<&str>,
) -> (String, String, f32, Vec<f32>, Option<String>) {
    if let Some(WorkerMessage::Sample(sample)) = worker_latest {
        if sample.status == Status::Healthy {
            let source = match sample.source {
                Source::Hardware => "hardware",
                Source::Mock => "mock",
                Source::None => "none",
            }
            .to_string();
            let mut pkg_temp = 0.0f32;
            let mut core_temps = Vec::new();
            for sensor in &sample.sensors {
                if sensor.kind == SensorKind::Package {
                    let value = sensor.value_c as f32;
                    if value.is_finite() {
                        pkg_temp = value;
                    }
                } else if sensor.kind == SensorKind::Core {
                    let value = sensor.value_c as f32;
                    // Guard against a malformed worker sample propagating NaN/Inf
                    // into `core_delta` (NaN comparisons would yield -inf).
                    if value.is_finite() {
                        core_temps.push(value);
                    }
                }
            }
            return (source, "healthy".to_string(), pkg_temp, core_temps, None);
        }
    }

    let (acpi_pkg, acpi_cores) = acpi;
    if acpi_pkg > 0.0 {
        return (
            "acpi".to_string(),
            "degraded".to_string(),
            acpi_pkg,
            acpi_cores,
            None,
        );
    }

    (
        "none".to_string(),
        "unavailable".to_string(),
        0.0,
        Vec::new(),
        worker_error.map(str::to_string),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use zenith_sensor_worker::protocol::{
        CpuInfo, Derived, ErrorCode, ErrorDetail, ErrorEnvelope, Quality, SampleEnvelope,
        SensorKind, SensorSample,
    };

    fn healthy_sample() -> WorkerMessage {
        WorkerMessage::Sample(SampleEnvelope {
            protocol: 1,
            timestamp_ms: 1,
            status: Status::Healthy,
            source: Source::Hardware,
            cpu: CpuInfo {
                vendor: "GenuineIntel".into(),
                family: 6,
                model: 158,
                cores: 4,
                threads: 8,
            },
            sensors: vec![
                SensorSample {
                    kind: SensorKind::Package,
                    index: 0,
                    name: "CPU Package".into(),
                    value_c: 70.0,
                    quality: Quality::Valid,
                },
                SensorSample {
                    kind: SensorKind::Core,
                    index: 0,
                    name: "Core #0".into(),
                    value_c: 70.0,
                    quality: Quality::Valid,
                },
                SensorSample {
                    kind: SensorKind::Core,
                    index: 1,
                    name: "Core #1".into(),
                    value_c: 66.0,
                    quality: Quality::Valid,
                },
            ],
            derived: Derived {
                hottest_core_c: 70.0,
                coolest_core_c: 66.0,
                core_delta_c: 4.0,
            },
        })
    }

    fn unavailable_error() -> WorkerMessage {
        WorkerMessage::Error(ErrorEnvelope {
            protocol: 1,
            timestamp_ms: 1,
            status: Status::Unavailable,
            source: Source::None,
            error: ErrorDetail {
                code: ErrorCode::DriverUnavailable,
                message: "device not reachable".into(),
            },
        })
    }

    #[test]
    fn hardware_sample_merges_sensors() {
        let (source, status, pkg, cores, err) =
            merge_temps(&Some(healthy_sample()), (0.0, Vec::new()), None);
        assert_eq!(source, "hardware");
        assert_eq!(status, "healthy");
        assert_eq!(pkg, 70.0);
        assert_eq!(cores, vec![70.0, 66.0]);
        assert!(err.is_none());
    }

    #[test]
    fn mock_sample_is_labeled_mock() {
        let mut msg = healthy_sample();
        if let WorkerMessage::Sample(s) = &mut msg {
            s.source = Source::Mock;
        }
        let (source, status, pkg, _, _) = merge_temps(&Some(msg), (0.0, Vec::new()), None);
        assert_eq!(source, "mock");
        assert_eq!(status, "healthy");
        assert_eq!(pkg, 70.0);
    }

    #[test]
    fn worker_error_falls_back_to_acpi_labeled() {
        let (source, status, pkg, cores, err) =
            merge_temps(&Some(unavailable_error()), (48.5, vec![47.0, 46.0]), None);
        assert_eq!(source, "acpi");
        assert_eq!(status, "degraded");
        assert_eq!(pkg, 48.5);
        assert_eq!(cores, vec![47.0, 46.0]);
        assert!(err.is_none());
    }

    #[test]
    fn non_finite_core_values_are_filtered() {
        let mut msg = healthy_sample();
        if let WorkerMessage::Sample(s) = &mut msg {
            s.sensors.push(SensorSample {
                kind: SensorKind::Core,
                index: 2,
                name: "Core #2".into(),
                value_c: f64::NAN,
                quality: Quality::Invalid,
            });
            s.sensors.push(SensorSample {
                kind: SensorKind::Package,
                index: 0,
                name: "CPU Package".into(),
                value_c: f64::NAN,
                quality: Quality::Invalid,
            });
        }
        let (_, _, pkg, cores, _) = merge_temps(&Some(msg), (0.0, Vec::new()), None);
        assert_eq!(pkg, 70.0, "package stays from the valid sensor");
        assert_eq!(cores, vec![70.0, 66.0], "NaN core filtered out");
    }

    #[test]
    fn no_worker_no_acpi_is_unavailable_with_error() {
        let (source, status, pkg, cores, err) = merge_temps(
            &Some(unavailable_error()),
            (0.0, Vec::new()),
            Some("spawn failed: file not found"),
        );
        assert_eq!(source, "none");
        assert_eq!(status, "unavailable");
        assert_eq!(pkg, 0.0);
        assert!(cores.is_empty());
        assert_eq!(err.as_deref(), Some("spawn failed: file not found"));
    }

    #[test]
    fn no_worker_no_acpi_no_error_is_unavailable() {
        let (source, status, _, _, err) = merge_temps(&None, (0.0, Vec::new()), None);
        assert_eq!(source, "none");
        assert_eq!(status, "unavailable");
        assert!(err.is_none());
    }
}
