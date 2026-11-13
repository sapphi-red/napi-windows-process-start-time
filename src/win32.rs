use core::ffi::CStr;
use core::ptr;

use windows_sys::Win32::Foundation::{
    CloseHandle, FARPROC, HANDLE, HMODULE, WAIT_ABANDONED, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetProcessTimes, OpenProcess, TerminateProcess, WaitForSingleObject,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
};

pub use windows_sys::Win32::Foundation::FILETIME as FileTime;

// Preserve the signed NTSTATUS bit pattern as an unsigned process exit code.
pub const STATUS_STACK_BUFFER_OVERRUN: u32 =
    u32::from_ne_bytes(windows_sys::Win32::Foundation::STATUS_STACK_BUFFER_OVERRUN.to_ne_bytes());

// The const parameters track limited-query and synchronize access rights.
pub struct ProcessHandle<const QUERY: bool, const SYNCHRONIZE: bool>(HANDLE);

pub fn open_process<const QUERY: bool, const SYNCHRONIZE: bool>(
    pid: u32,
) -> Option<ProcessHandle<QUERY, SYNCHRONIZE>> {
    let access = if QUERY {
        PROCESS_QUERY_LIMITED_INFORMATION
    } else {
        0
    } | if SYNCHRONIZE { PROCESS_SYNCHRONIZE } else { 0 };
    // SAFETY: All arguments are scalar values; Windows validates the PID and requested access.
    // A successful handle is owned by ProcessHandle and closed on drop.
    let handle = unsafe { OpenProcess(access, 0, pid) };
    if handle.is_null() {
        None
    } else {
        Some(ProcessHandle(handle))
    }
}

impl<const QUERY: bool, const SYNCHRONIZE: bool> Drop for ProcessHandle<QUERY, SYNCHRONIZE> {
    fn drop(&mut self) {
        // SAFETY: Only open_process constructs this owner, and it closes the handle once.
        unsafe {
            CloseHandle(self.0);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitResult {
    Signaled,
    Abandoned,
    Timeout,
    Failed,
}

pub fn wait_for_single_object<const QUERY: bool>(
    process: &ProcessHandle<QUERY, true>,
    timeout: u32,
) -> WaitResult {
    // SAFETY: The borrow keeps the handle open throughout the wait, and the handle's
    // SYNCHRONIZE = true type parameter guarantees the required access right.
    match unsafe { WaitForSingleObject(process.0, timeout) } {
        WAIT_OBJECT_0 => WaitResult::Signaled,
        WAIT_ABANDONED => WaitResult::Abandoned,
        WAIT_TIMEOUT => WaitResult::Timeout,
        // WAIT_FAILED or an unexpected return value.
        _ => WaitResult::Failed,
    }
}

#[derive(Default)]
pub struct ProcessTimes {
    pub creation: FileTime,
    pub exit: FileTime,
    pub kernel: FileTime,
    pub user: FileTime,
}

pub fn get_process_times<const SYNCHRONIZE: bool>(
    process: &ProcessHandle<true, SYNCHRONIZE>,
) -> Option<ProcessTimes> {
    let mut times = ProcessTimes::default();
    // SAFETY: QUERY = true guarantees limited-query access, the borrow keeps the handle open,
    // and all outputs point to writable fields.
    if unsafe {
        GetProcessTimes(
            process.0,
            &raw mut times.creation,
            &raw mut times.exit,
            &raw mut times.kernel,
            &raw mut times.user,
        )
    } == 0
    {
        None
    } else {
        Some(times)
    }
}

// A borrowed pseudo-handle must not be closed by ProcessHandle's destructor.
pub struct CurrentProcess(HANDLE);

pub fn get_current_process() -> CurrentProcess {
    CurrentProcess(unsafe { GetCurrentProcess() })
}

/// Successful termination of the current process never returns, so callers only
/// observe `false` when termination fails.
pub fn terminate_process(process: &CurrentProcess, code: u32) -> bool {
    // SAFETY: CurrentProcess wraps GetCurrentProcess's valid pseudo-handle, which
    // grants PROCESS_TERMINATE access and is never closed by this wrapper.
    unsafe { TerminateProcess(process.0, code) != 0 }
}

// Only the host executable is exposed, so its module cannot be unloaded while in use.
pub struct Module(HMODULE);

pub fn get_module_handle() -> Module {
    // SAFETY: A null name requests the host executable's module handle without
    // dereferencing a pointer. The executable stays loaded for the process lifetime.
    Module(unsafe { GetModuleHandleW(ptr::null()) })
}

pub fn get_proc_address(module: &Module, name: &CStr) -> FARPROC {
    // SAFETY: The module belongs to the host executable and the name is NUL-terminated.
    unsafe { GetProcAddress(module.0, name.as_ptr().cast()) }
}
