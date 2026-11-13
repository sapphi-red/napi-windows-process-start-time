use core::ffi::CStr;
use core::ptr;

use windows_sys::Win32::Foundation::{CloseHandle, FARPROC, HANDLE, HMODULE};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetProcessTimes, OpenProcess, TerminateProcess, WaitForSingleObject,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
};

pub use windows_sys::Win32::Foundation::{FILETIME as FileTime, WAIT_TIMEOUT};

pub struct ProcessHandle(HANDLE);

pub fn open_process(pid: u32) -> Option<ProcessHandle> {
    let handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            pid,
        )
    };
    if handle.is_null() {
        None
    } else {
        Some(ProcessHandle(handle))
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        // SAFETY: Only open_process constructs this owner, and it closes the handle once.
        unsafe {
            CloseHandle(self.0);
        }
    }
}

pub fn wait_for_single_object(process: &ProcessHandle, timeout: u32) -> u32 {
    unsafe { WaitForSingleObject(process.0, timeout) }
}

pub fn get_process_times(
    process: &ProcessHandle,
    creation: &mut FileTime,
    exit: &mut FileTime,
    kernel: &mut FileTime,
    user: &mut FileTime,
) -> bool {
    // SAFETY: The owner keeps the process handle open, and all outputs are writable references.
    unsafe { GetProcessTimes(process.0, creation, exit, kernel, user) != 0 }
}

// A borrowed pseudo-handle must not be closed by ProcessHandle's destructor.
pub struct CurrentProcess(HANDLE);

pub fn get_current_process() -> CurrentProcess {
    CurrentProcess(unsafe { GetCurrentProcess() })
}

pub fn terminate_process(process: &CurrentProcess, code: u32) -> bool {
    unsafe { TerminateProcess(process.0, code) != 0 }
}

// Only the host executable is exposed, so its module cannot be unloaded while in use.
pub struct Module(HMODULE);

pub fn get_module_handle() -> Module {
    Module(unsafe { GetModuleHandleW(ptr::null()) })
}

pub fn get_proc_address(module: &Module, name: &CStr) -> FARPROC {
    // SAFETY: The module belongs to the host executable and the name is NUL-terminated.
    unsafe { GetProcAddress(module.0, name.as_ptr().cast()) }
}
