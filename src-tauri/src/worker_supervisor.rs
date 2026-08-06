use std::io::BufRead;
use std::io::BufReader;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use zenith_sensor_worker::protocol::{Status, WorkerMessage};
use zenith_sensor_worker::supervisor::{CircuitBreaker, NextAction};

const HANG_TIMEOUT: Duration = Duration::from_secs(3);
const WAIT_POLL_INTERVAL: Duration = Duration::from_millis(50);

pub fn resolve_worker_exe() -> PathBuf {
    #[cfg(debug_assertions)]
    if let Ok(env_path) = std::env::var("ZENITH_WORKER_PATH") {
        let p = PathBuf::from(env_path);
        if p.is_file() {
            return p;
        }
    }
    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
    {
        let worker_root = exe_dir.join("..").join("..").join("worker");
        for profile in ["release", "debug"] {
            let candidate = worker_root
                .join("target")
                .join(profile)
                .join("zenith-sensor-worker.exe");
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    PathBuf::from("zenith-sensor-worker.exe")
}

enum ReaderEvent {
    Message(WorkerMessage),
    Ended,
}

/// Spawns and supervises the sensor worker child process. All lifecycle logic
/// runs on a dedicated controller thread; consumers only read the latest
/// message or error.
pub struct WorkerSupervisor {
    running: Arc<AtomicBool>,
    latest: Arc<Mutex<Option<WorkerMessage>>>,
    last_error: Arc<Mutex<Option<String>>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl WorkerSupervisor {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(true)),
            latest: Arc::new(Mutex::new(None)),
            last_error: Arc::new(Mutex::new(None)),
            handle: None,
        }
    }

    pub fn start(&mut self, exe: &Path) {
        if self.handle.is_some() {
            return;
        }
        let running = Arc::clone(&self.running);
        let latest = Arc::clone(&self.latest);
        let last_error = Arc::clone(&self.last_error);
        let exe = exe.to_path_buf();
        self.handle = Some(
            std::thread::Builder::new()
                .name("worker-supervisor".into())
                .spawn(move || controller(exe, running, latest, last_error))
                .expect("spawn worker supervisor thread"),
        );
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    pub fn latest(&self) -> Option<WorkerMessage> {
        self.latest
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn last_error(&self) -> Option<String> {
        self.last_error
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

impl Drop for WorkerSupervisor {
    fn drop(&mut self) {
        self.stop();
    }
}

fn controller(
    exe: PathBuf,
    running: Arc<AtomicBool>,
    latest: Arc<Mutex<Option<WorkerMessage>>>,
    last_error: Arc<Mutex<Option<String>>>,
) {
    let mut breaker = CircuitBreaker::new();
    while running.load(Ordering::Relaxed) {
        if let Err(msg) = run_child(&exe, &running, &latest, &mut breaker) {
            if let Ok(mut e) = last_error.lock() {
                *e = Some(msg);
            }
        }
        if !running.load(Ordering::Relaxed) {
            break;
        }
        match breaker.on_failure() {
            NextAction::Wait(delay) => {
                wait_interruptible(&running, delay);
                if !running.load(Ordering::Relaxed) {
                    break;
                }
                breaker.on_respawn();
            }
            NextAction::Failed => {
                if let Ok(mut e) = last_error.lock() {
                    *e = Some("worker stopped after repeated failures".into());
                }
                break;
            }
        }
    }
}

fn run_child(
    exe: &Path,
    running: &AtomicBool,
    latest: &Mutex<Option<WorkerMessage>>,
    breaker: &mut CircuitBreaker,
) -> Result<(), String> {
    let mock = mock_override();

    let mut cmd = Command::new(exe);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
    cmd.creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW);
    if mock {
        cmd.arg("--mock");
    }
    let mut child: Child = cmd
        .spawn()
        .map_err(|e| format!("failed to spawn {}: {e}", exe.display()))?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "worker stdin unavailable".to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "worker stdout unavailable".to_string())?;

    let (tx, rx) = mpsc::channel::<ReaderEvent>();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            match line {
                Ok(text) => {
                    if let Ok(msg) = serde_json::from_str::<WorkerMessage>(text.trim_end()) {
                        if tx.send(ReaderEvent::Message(msg)).is_err() {
                            break;
                        }
                    }
                }
                Err(_) => break,
            }
        }
        let _ = tx.send(ReaderEvent::Ended);
    });

    loop {
        match rx.recv_timeout(HANG_TIMEOUT) {
            Ok(ReaderEvent::Message(msg)) => {
                if matches!(&msg, WorkerMessage::Sample(s) if s.status == Status::Healthy) {
                    breaker.on_health();
                }
                if let Ok(mut l) = latest.lock() {
                    *l = Some(msg);
                }
            }
            Ok(ReaderEvent::Ended) | Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => break,
        }
        if !running.load(Ordering::Relaxed) {
            break;
        }
    }

    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
    Ok(())
}

/// `ZENITH_WORKER_MOCK` is a development-only escape hatch. In release builds it
/// is never honored so an arbitrary process cannot silently switch the elevated
/// app to fake readings, and `ZENITH_WORKER_PATH` cannot point it at arbitrary
/// binaries (see `resolve_worker_exe`).
#[cfg(debug_assertions)]
fn mock_override() -> bool {
    std::env::var("ZENITH_WORKER_MOCK")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[cfg(not(debug_assertions))]
fn mock_override() -> bool {
    false
}

fn wait_interruptible(running: &AtomicBool, delay: Duration) {
    let start = Instant::now();
    while running.load(Ordering::Relaxed) && start.elapsed() < delay {
        std::thread::sleep(WAIT_POLL_INTERVAL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use zenith_sensor_worker::protocol::Source;

    #[test]
    fn supervisor_streams_mock_samples() {
        let exe = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("worker")
            .join("target")
            .join("release")
            .join("zenith-sensor-worker.exe");
        assert!(
            exe.is_file(),
            "build the worker first: cargo build --release --manifest-path {}/worker/Cargo.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        std::env::set_var("ZENITH_WORKER_PATH", &exe);
        std::env::set_var("ZENITH_WORKER_MOCK", "1");

        let mut sup = WorkerSupervisor::new();
        sup.start(&exe);
        let deadline = Instant::now() + Duration::from_secs(8);
        let mut saw_mock_sample = false;
        while Instant::now() < deadline {
            if let Some(WorkerMessage::Sample(s)) = sup.latest() {
                if s.status == Status::Healthy && s.source == Source::Mock {
                    saw_mock_sample = true;
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        sup.stop();

        assert!(
            saw_mock_sample,
            "no healthy mock sample observed; last_error={:?}",
            sup.last_error()
        );
    }
}
