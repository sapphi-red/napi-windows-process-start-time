#![no_std]

use core::ptr;

mod napi;
mod win32;

use napi::{Env, Info, Value};
use win32::{FileTime, Handle};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe {
        win32::TerminateProcess(win32::GetCurrentProcess(), 0xC000_0409);
    }
    loop {
        core::hint::spin_loop();
    }
}

struct ProcessHandle(Handle);
impl Drop for ProcessHandle {
    fn drop(&mut self) {
        unsafe {
            win32::CloseHandle(self.0);
        }
    }
}

fn process_ticks(pid: u32) -> Option<u64> {
    let handle = unsafe {
        win32::OpenProcess(
            win32::PROCESS_QUERY_LIMITED_INFORMATION | win32::PROCESS_SYNCHRONIZE,
            0,
            pid,
        )
    };
    if handle.is_null() {
        return None;
    }
    let handle = ProcessHandle(handle);
    if unsafe { win32::WaitForSingleObject(handle.0, 0) } != win32::WAIT_TIMEOUT {
        return None;
    }
    let mut creation = FileTime::default();
    let mut exit = FileTime::default();
    let mut kernel = FileTime::default();
    let mut user = FileTime::default();
    if unsafe {
        win32::GetProcessTimes(
            handle.0,
            &raw mut creation,
            &raw mut exit,
            &raw mut kernel,
            &raw mut user,
        )
    } == 0
    {
        return None;
    }
    Some(
        ((u64::from(creation.dwHighDateTime) << 32) | u64::from(creation.dwLowDateTime))
            + 504_911_232_000_000_000,
    )
}

unsafe extern "C" fn get_process_start_time(env: Env, info: Info) -> Value {
    let mut argc = 1;
    let mut arg = ptr::null_mut();
    if napi::napi_get_cb_info(
        env,
        info,
        &raw mut argc,
        &raw mut arg,
        ptr::null_mut(),
        ptr::null_mut(),
    ) != 0
    {
        return ptr::null_mut();
    }
    // The JavaScript wrapper validates the PID's integer value and uint32 range.
    let mut pid = 0;
    if argc == 0 || napi::napi_get_value_uint32(env, arg, &raw mut pid) != 0 {
        napi::napi_throw_type_error(env, ptr::null(), c"pid must be a uint32".as_ptr());
        return ptr::null_mut();
    }
    let mut result = ptr::null_mut();
    if let Some(ticks) = process_ticks(pid) {
        if napi::napi_create_bigint_uint64(env, ticks, &raw mut result) != 0 {
            return ptr::null_mut();
        }
    } else if napi::napi_get_null(env, &raw mut result) != 0 {
        return ptr::null_mut();
    }
    result
}

#[no_mangle]
pub extern "C" fn node_api_module_get_api_version_v1() -> i32 {
    6
}

#[no_mangle]
/// Initializes an addon environment when Node.js loads this DLL.
///
/// # Safety
/// Node.js must provide a valid environment and exports object for the current
/// thread. The callback pointers registered here use the Node-API C ABI.
pub unsafe extern "C" fn napi_register_module_v1(env: Env, exports: Value) -> Value {
    let name = b"getProcessStartTime\0";
    let mut function = ptr::null_mut();
    if napi::napi_create_function(
        env,
        name.as_ptr().cast(),
        name.len() - 1,
        get_process_start_time,
        ptr::null_mut(),
        &raw mut function,
    ) != 0
    {
        return ptr::null_mut();
    }
    if napi::napi_set_named_property(env, exports, name.as_ptr().cast(), function) != 0 {
        return ptr::null_mut();
    }
    exports
}
