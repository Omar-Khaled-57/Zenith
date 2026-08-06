use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use zenith_sensor_worker::hardware_access::{AccessError, HardwareAccess};
use zenith_sensor_worker::protocol::{ErrorCode, ErrorDetail, Source, Status, WorkerMessage};
use zenith_sensor_worker::topology::Topology;

const SAMPLE_INTERVAL: Duration = Duration::from_secs(1);
const MODULE_BLOB: &[u8] = include_bytes!("../res/IntelMSR.bin");

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let use_mock = args.iter().any(|a| a == "--mock");
    let mock_unavailable = args.iter().any(|a| a == "--unavailable");
    let debug = args.iter().any(|a| a == "--debug");

    let topology = match Topology::detect() {
        Ok(t) => t,
        Err(e) => {
            emit_error(ErrorDetail {
                code: ErrorCode::TopologyFailed,
                message: format!("topology detection failed: {e:?}"),
            });
            std::process::exit(1);
        }
    };

    if debug {
        eprintln!(
            "[debug] cpuid vendor={} family=0x{:X} model=0x{:X} cores={} threads={} packages={}",
            topology.vendor,
            topology.family,
            topology.model,
            topology.core_first_logical.len(),
            topology.threads.len(),
            topology.package_count,
        );
    }

    let backend: Box<dyn HardwareAccess> = if use_mock {
        let deltas: Vec<u8> = (0..topology.threads.len()).map(|i| (30 + (i % 5)) as u8).collect();
        let package_delta = *deltas.first().unwrap_or(&30);
        let mock = zenith_sensor_worker::mock::MockHardware::new(100, deltas, package_delta);
        if mock_unavailable {
            Box::new(mock.unavailable())
        } else {
            Box::new(mock)
        }
    } else {
        Box::new(zenith_sensor_worker::pawnio::PawnIo::new(MODULE_BLOB.to_vec()))
    };

    let mut backend = backend;
    if let Err(e) = backend.open() {
        eprintln!("[debug] backend {} open failed: {e:?}", backend.name());
        let (code, msg) = access_error_parts(&e);
        emit_error(ErrorDetail {
            code,
            message: format!("{}: {msg}", backend.name()),
        });
        std::process::exit(1);
    }

    if debug {
        eprintln!("[debug] backend {} opened (module {})", backend.name(), if use_mock { "mock" } else { "intelmsr" });
    }

    let mut decoder = match zenith_sensor_worker::intel::IntelDecoder::new(backend.as_mut(), &topology.core_first_logical) {
        Ok(d) => d,
        Err(e) => {
            let (code, msg) = access_error_parts(&e);
            emit_error(ErrorDetail { code, message: msg });
            backend.close();
            std::process::exit(1);
        }
    };

    if debug {
        eprintln!("[debug] tjmax={}", decoder.tj_max);
    }

    let shutdown = Arc::new(AtomicBool::new(false));
    watch_stdin(Arc::clone(&shutdown));

    let mut next = Instant::now();
    while !shutdown.load(Ordering::Relaxed) {
        next += SAMPLE_INTERVAL;
        let start = Instant::now();

        match decoder.sample(backend.as_mut(), &topology.core_first_logical) {
            Ok((sensors, derived)) => {
                let msg = zenith_sensor_worker::protocol::SampleEnvelope {
                    protocol: zenith_sensor_worker::protocol::PROTOCOL_VERSION,
                    timestamp_ms: zenith_sensor_worker::protocol::now_ms(),
                    status: Status::Healthy,
                    source: if use_mock { Source::Mock } else { Source::Hardware },
                    cpu: zenith_sensor_worker::protocol::CpuInfo {
                        vendor: topology.vendor.clone(),
                        family: topology.family,
                        model: topology.model,
                        cores: topology.core_first_logical.len(),
                        threads: topology.threads.len(),
                    },
                    sensors,
                    derived,
                };
                emit(&zenith_sensor_worker::protocol::WorkerMessage::Sample(msg));
            }
            Err(e) => {
                let (code, msg) = access_error_parts(&e);
                emit(&zenith_sensor_worker::protocol::WorkerMessage::Error(zenith_sensor_worker::protocol::ErrorEnvelope {
                    protocol: zenith_sensor_worker::protocol::PROTOCOL_VERSION,
                    timestamp_ms: zenith_sensor_worker::protocol::now_ms(),
                    status: Status::Degraded,
                    source: Source::None,
                    error: ErrorDetail { code, message: msg },
                }));
            }
        }

        if debug {
            eprintln!(
                "[debug] sample took {} ms (of {} ms budget)",
                start.elapsed().as_millis(),
                SAMPLE_INTERVAL.as_millis()
            );
        }

        let now = Instant::now();
        if next > now {
            std::thread::sleep(next - now);
        } else {
            next = now;
        }
    }

    backend.close();
}

fn access_error_parts(e: &AccessError) -> (ErrorCode, String) {
    (e.code(), format!("{e:?}"))
}

fn watch_stdin(shutdown: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = [0u8; 16];
        let mut stdin = std::io::stdin();
        loop {
            match stdin.read(&mut buf) {
                Ok(0) => break, // EOF: parent gone
                Ok(_) => {}
                Err(_) => break,
            }
        }
        shutdown.store(true, Ordering::Relaxed);
    });
}

fn emit(msg: &WorkerMessage) {
    let s = msg.to_jsonl();
    let mut out = std::io::stdout().lock();
    if out.write_all(s.as_bytes()).is_err() {
        std::process::exit(0);
    }
    let _ = out.flush();
}

fn emit_error(detail: ErrorDetail) {
    emit(&zenith_sensor_worker::protocol::WorkerMessage::Error(zenith_sensor_worker::protocol::ErrorEnvelope {
        protocol: zenith_sensor_worker::protocol::PROTOCOL_VERSION,
        timestamp_ms: zenith_sensor_worker::protocol::now_ms(),
        status: Status::Unavailable,
        source: Source::None,
        error: detail,
    }));
}

