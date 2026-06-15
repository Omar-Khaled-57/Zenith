use serde::{Deserialize, Serialize};
use sysinfo::{Components, Disks, ProcessesToUpdate, System};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessInfo {
    pub pid: usize,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_mb: f32,
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
}

/// Simple LCG PRNG seeded once; deterministic per-seed.
struct SeededRand(u64);

impl SeededRand {
    fn new(seed: u64) -> Self {
        Self(seed)
    }
    /// Returns value in [0, 1).
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) as f32 / (1u64 << 31) as f32
    }
}

/// Map 0–100% CPU usage to a realistic temperature range.
fn cpu_load_to_temp(load_pct: f32) -> f32 {
    const IDLE_TEMP: f32 = 42.0;
    const LOAD_CAP_TEMP: f32 = 92.0;
    IDLE_TEMP + (load_pct / 100.0) * (LOAD_CAP_TEMP - IDLE_TEMP)
}

/// Refresh processes only every N polls to reduce syscall overhead.
const PROCESS_REFRESH_INTERVAL: u32 = 5;

pub struct SensorEngine {
    sys: System,
    components: Components,
    disks: Disks,
    poll_count: u32,
    rng: SeededRand,
}

impl SensorEngine {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_cpu_all();

        let components = Components::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();

        Self {
            sys,
            components,
            disks,
            poll_count: 0,
            rng: SeededRand::new(42),
        }
    }

    pub fn poll(&mut self) -> SensorPayload {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        self.components.refresh(false);

        // Refresh processes less frequently to reduce overhead
        self.poll_count += 1;
        if self.poll_count % PROCESS_REFRESH_INTERVAL == 1 {
            self.sys.refresh_processes(ProcessesToUpdate::All, true);
        }

        let global_cpu = self.sys.global_cpu_usage();

        // ── Read real sensor data (single pass over components) ──
        let mut core_temps: Vec<f32> = Vec::new();
        let mut pkg_temp: f32 = 0.0;

        for component in self.components.iter() {
            let label = component.label().to_lowercase();
            let Some(temp) = component.temperature() else { continue };
            if temp <= 0.0 {
                continue;
            }

            if label.contains("package") || label.contains("tctl") || label.starts_with("cpu package")
            {
                pkg_temp = temp;
            } else if label.contains("core") || label.starts_with("cpu core") || label.starts_with("core ")
            {
                core_temps.push(temp);
            }
        }

        // ── Fallback: infer pkg temp from core temps ──
        if pkg_temp == 0.0 && !core_temps.is_empty() {
            pkg_temp = core_temps.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        }

        // ── Simulation fallback when no hardware sensors are available ──
        if core_temps.is_empty() {
            let base_temp = cpu_load_to_temp(global_cpu);
            pkg_temp = base_temp + 3.0;

            for _ in 1..=8 {
                let jitter = self.rng.next() * 10.0;
                let spread = self.rng.next() * 4.0;
                let temp = base_temp - jitter + spread;
                core_temps.push(temp.max(25.0).min(100.0));
            }
        }

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
        let mut processes: Vec<ProcessInfo> = self
            .sys
            .processes()
            .iter()
            .map(|(pid, p)| ProcessInfo {
                pid: pid.as_u32() as usize,
                name: p.name().to_string_lossy().into_owned(),
                cpu_usage: p.cpu_usage(),
                memory_mb: p.memory() as f32 / 1_073_741_824.0f32,
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
            ram_usage: ram_usage as f32,
            ram_total: ram_total as f32,
            disk_usage: disk_usage as f32,
            disk_total: disk_total as f32,
            top_processes: processes,
        }
    }
}
