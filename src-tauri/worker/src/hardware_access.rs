use crate::protocol::ErrorCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessError {
    DriverUnavailable,
    ModuleMissing,
    MsrFailed,
    TopologyFailed,
    #[allow(dead_code)] // reserved for internal backend faults
    Internal,
}

impl AccessError {
    pub fn code(&self) -> ErrorCode {
        match self {
            AccessError::DriverUnavailable => ErrorCode::DriverUnavailable,
            AccessError::ModuleMissing => ErrorCode::ModuleMissing,
            AccessError::MsrFailed => ErrorCode::MsrFailed,
            AccessError::TopologyFailed => ErrorCode::TopologyFailed,
            AccessError::Internal => ErrorCode::Internal,
        }
    }
}

pub trait HardwareAccess {
    fn name(&self) -> &'static str;

    fn open(&mut self) -> Result<(), AccessError>;

    fn is_available(&self) -> bool;

    /// Reads a 64-bit MSR on the given logical processor. The implementation is
    /// responsible for pinning the calling thread to that processor first.
    fn read_msr(&mut self, msr: u64, logical_cpu: usize) -> Result<u64, AccessError>;

    fn close(&mut self);
}
