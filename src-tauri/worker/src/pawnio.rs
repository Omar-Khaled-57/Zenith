use std::ffi::c_void;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::SystemInformation::GROUP_AFFINITY;
use windows_sys::Win32::System::Threading::{GetCurrentThread, SetThreadGroupAffinity};

use crate::hardware_access::{AccessError, HardwareAccess};

const DEVICE_TYPE: u32 = 41394;
const IOCTL_LOAD_BINARY: u32 = (DEVICE_TYPE << 16) | (0x821 << 2);
const IOCTL_EXECUTE_FN: u32 = (DEVICE_TYPE << 16) | (0x841 << 2);

const DEVICE_PATH: &str = "\\\\.\\GLOBALROOT\\Device\\PawnIO";
const FN_NAME_LENGTH: usize = 32;
const READ_MSR_FN: &str = "ioctl_read_msr";

const GENERIC_READ: u32 = 0x8000_0000;
const GENERIC_WRITE: u32 = 0x4000_0000;
const FILE_SHARE_READ: u32 = 1;
const FILE_SHARE_WRITE: u32 = 2;
const OPEN_EXISTING: u32 = 3;
const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;

pub struct PawnIo {
    handle: HANDLE,
    blob: Option<Vec<u8>>,
    loaded: bool,
}

impl PawnIo {
    pub fn new(module_blob: Vec<u8>) -> Self {
        Self {
            handle: INVALID_HANDLE_VALUE,
            blob: Some(module_blob),
            loaded: false,
        }
    }

    fn open_device(&mut self) -> Result<(), AccessError> {
        let handle = unsafe { create_device_handle() };
        if handle == INVALID_HANDLE_VALUE {
            return Err(AccessError::DriverUnavailable);
        }
        self.handle = handle;
        Ok(())
    }

    fn load_module(&mut self) -> Result<(), AccessError> {
        let blob = self.blob.take().ok_or(AccessError::ModuleMissing)?;
        let mut written = 0u32;
        let ok = unsafe {
            windows_sys::Win32::System::IO::DeviceIoControl(
                self.handle,
                IOCTL_LOAD_BINARY,
                blob.as_ptr() as *const c_void,
                blob.len() as u32,
                std::ptr::null_mut(),
                0,
                &mut written,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(AccessError::ModuleMissing);
        }
        self.loaded = true;
        Ok(())
    }

    fn execute(&mut self, name: &str, input: &[u64], out_len: usize) -> Result<Vec<u64>, AccessError> {
        if !self.loaded {
            return Err(AccessError::ModuleMissing);
        }
        let mut total = vec![0u8; FN_NAME_LENGTH + input.len() * 8];
        for (i, b) in name.bytes().take(FN_NAME_LENGTH - 1).enumerate() {
            total[i] = b;
        }
        for (i, v) in input.iter().enumerate() {
            total[FN_NAME_LENGTH + i * 8..FN_NAME_LENGTH + (i + 1) * 8]
                .copy_from_slice(&v.to_le_bytes());
        }
        let mut output = vec![0u8; out_len * 8];
        let mut written = 0u32;
        let ok = unsafe {
            windows_sys::Win32::System::IO::DeviceIoControl(
                self.handle,
                IOCTL_EXECUTE_FN,
                total.as_ptr() as *const c_void,
                total.len() as u32,
                output.as_mut_ptr() as *mut c_void,
                output.len() as u32,
                &mut written,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(AccessError::MsrFailed);
        }
        let mut vals = Vec::with_capacity(out_len);
        for i in 0..out_len {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&output[i * 8..i * 8 + 8]);
            vals.push(u64::from_le_bytes(arr));
        }
        Ok(vals)
    }
}

impl HardwareAccess for PawnIo {
    fn name(&self) -> &'static str {
        "pawnio"
    }

    fn open(&mut self) -> Result<(), AccessError> {
        self.open_device()?;
        self.load_module()
    }

    fn is_available(&self) -> bool {
        self.loaded && self.handle != INVALID_HANDLE_VALUE
    }

    fn read_msr(&mut self, msr: u64, logical_cpu: usize) -> Result<u64, AccessError> {
        if logical_cpu >= 64 {
            return Err(AccessError::TopologyFailed);
        }
        let mask = 1u64 << logical_cpu;
        let group = GROUP_AFFINITY {
            Mask: mask as usize,
            Group: 0,
            Reserved: [0; 3],
        };
        let mut prev = GROUP_AFFINITY {
            Mask: 0,
            Group: 0,
            Reserved: [0; 3],
        };
        let ok = unsafe {
            SetThreadGroupAffinity(GetCurrentThread(), &group, &mut prev)
        };
        if ok == 0 {
            return Err(AccessError::TopologyFailed);
        }
        let out = self.execute(READ_MSR_FN, &[msr], 1)?;
        Ok(out[0])
    }

    fn close(&mut self) {
        if self.handle != INVALID_HANDLE_VALUE {
            unsafe { CloseHandle(self.handle) };
            self.handle = INVALID_HANDLE_VALUE;
        }
        self.loaded = false;
    }
}

unsafe fn create_device_handle() -> HANDLE {
    let path: Vec<u16> = DEVICE_PATH.encode_utf16().chain(std::iter::once(0)).collect();
    windows_sys::Win32::Storage::FileSystem::CreateFileW(
        path.as_ptr(),
        GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        std::ptr::null(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        std::ptr::null_mut(),
    )
}
