use std::io::BufRead;
use std::process::{Child, Command, Stdio};

use zenith_sensor_worker::protocol::{
    ErrorCode, SensorKind, Source, Status, WorkerMessage,
};

fn spawn(args: &[&str]) -> (Child, impl BufRead) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zenith-sensor-worker"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn worker");
    let stdout = std::io::BufReader::new(child.stdout.take().unwrap());
    (child, stdout)
}

fn next_line<R: BufRead>(stdout: &mut R) -> WorkerMessage {
    let mut line = String::new();
    stdout.read_line(&mut line).expect("read line");
    assert!(!line.is_empty(), "worker produced no output");
    serde_json::from_str(line.trim_end()).expect("parse jsonl")
}

#[test]
fn mock_emits_healthy_samples() {
    let (mut child, mut stdout) = spawn(&["--mock"]);

    let WorkerMessage::Sample(sample) = next_line(&mut stdout) else {
        panic!("expected sample, got error");
    };
    assert_eq!(sample.protocol, 1);
    assert_eq!(sample.status, Status::Healthy);
    assert_eq!(sample.source, Source::Mock);
    assert!(sample.cpu.cores >= 1);
    assert!(sample.cpu.threads >= sample.cpu.cores);

    assert!(sample.sensors.iter().any(|s| s.kind == SensorKind::Package));
    assert!(sample.sensors.iter().any(|s| s.kind == SensorKind::Core));
    assert!(sample.derived.core_delta_c >= 0.0);

    let WorkerMessage::Sample(second) = next_line(&mut stdout) else {
        panic!("expected second sample");
    };
    assert!(second.timestamp_ms >= sample.timestamp_ms);

    child.kill().ok();
}

#[test]
fn mock_unavailable_emits_error_and_exits() {
    let (mut child, mut stdout) = spawn(&["--mock", "--unavailable"]);

    let WorkerMessage::Error(err) = next_line(&mut stdout) else {
        panic!("expected error envelope");
    };
    assert_eq!(err.status, Status::Unavailable);
    assert_eq!(err.source, Source::None);
    assert!(matches!(err.error.code, ErrorCode::DriverUnavailable));

    let status = child.wait().expect("wait");
    assert_eq!(status.code(), Some(1));
}
