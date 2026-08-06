use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Hardware,
    Mock,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Healthy,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Quality {
    Valid,
    Suspicious,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SensorKind {
    Package,
    Core,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorSample {
    pub kind: SensorKind,
    pub index: usize,
    pub name: String,
    pub value_c: f64,
    pub quality: Quality,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CpuInfo {
    pub vendor: String,
    pub family: u32,
    pub model: u32,
    pub cores: usize,
    pub threads: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Derived {
    pub hottest_core_c: f64,
    pub coolest_core_c: f64,
    pub core_delta_c: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SampleEnvelope {
    pub protocol: u32,
    pub timestamp_ms: u64,
    pub status: Status,
    pub source: Source,
    pub cpu: CpuInfo,
    pub sensors: Vec<SensorSample>,
    pub derived: Derived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    DriverUnavailable,
    ModuleMissing,
    MsrFailed,
    TopologyFailed,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    pub protocol: u32,
    pub timestamp_ms: u64,
    pub status: Status,
    pub source: Source,
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkerMessage {
    Sample(SampleEnvelope),
    Error(ErrorEnvelope),
}

impl WorkerMessage {
    pub fn to_jsonl(&self) -> String {
        let mut s = serde_json::to_string(self).expect("message serializes");
        s.push('\n');
        s
    }
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_round_trip() {
        let msg = WorkerMessage::Sample(SampleEnvelope {
            protocol: PROTOCOL_VERSION,
            timestamp_ms: 1234567890,
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
                    value_c: 47.25,
                    quality: Quality::Valid,
                },
                SensorSample {
                    kind: SensorKind::Core,
                    index: 0,
                    name: "Core #0".into(),
                    value_c: 45.0,
                    quality: Quality::Valid,
                },
            ],
            derived: Derived {
                hottest_core_c: 45.0,
                coolest_core_c: 45.0,
                core_delta_c: 0.0,
            },
        });
        let jsonl = msg.to_jsonl();
        let back: WorkerMessage = serde_json::from_str(jsonl.trim_end()).unwrap();
        assert_eq!(msg, back);
    }

    #[test]
    fn error_round_trip() {
        let msg = WorkerMessage::Error(ErrorEnvelope {
            protocol: PROTOCOL_VERSION,
            timestamp_ms: 1,
            status: Status::Unavailable,
            source: Source::None,
            error: ErrorDetail {
                code: ErrorCode::DriverUnavailable,
                message: "device not reachable".into(),
            },
        });
        let jsonl = msg.to_jsonl();
        let back: WorkerMessage = serde_json::from_str(jsonl.trim_end()).unwrap();
        assert_eq!(msg, back);
        assert!(jsonl.ends_with('\n'));
    }

    #[test]
    fn error_code_serde_names() {
        let s = serde_json::to_string(&ErrorCode::DriverUnavailable).unwrap();
        assert_eq!(s, "\"DRIVER_UNAVAILABLE\"");
        let s = serde_json::to_string(&ErrorCode::ModuleMissing).unwrap();
        assert_eq!(s, "\"MODULE_MISSING\"");
    }

    #[test]
    fn quality_suspicious() {
        let s = serde_json::to_string(&Quality::Suspicious).unwrap();
        assert_eq!(s, "\"suspicious\"");
    }
}
