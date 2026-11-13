#![no_std]

use core::ptr;

mod napi;
mod win32;

use napi::{Env, Info, Value};

// 584,388 days between the .NET (0001-01-01) and FILETIME (1601-01-01) epochs
const FILETIME_TO_DOTNET_SECONDS: u64 = 584_388 * 24 * 60 * 60;
// Matches .NET's FileTimeOffset:
// https://github.com/dotnet/runtime/blob/main/src/libraries/System.Private.CoreLib/src/System/DateTime.cs
const FILETIME_TO_DOTNET_TICKS: u64 = FILETIME_TO_DOTNET_SECONDS * (1_000_000_000 / 100);

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    win32::terminate_process(
        &win32::get_current_process(),
        win32::STATUS_STACK_BUFFER_OVERRUN,
    );
    loop {
        core::hint::spin_loop();
    }
}

fn process_ticks(pid: u32) -> Option<u64> {
    let handle = win32::open_process(pid)?;
    if win32::wait_for_single_object(&handle, 0) != win32::WaitResult::Timeout {
        return None;
    }
    let creation = win32::get_process_times(&handle)?.creation;
    Some(
        ((u64::from(creation.dwHighDateTime) << 32) | u64::from(creation.dwLowDateTime))
            + FILETIME_TO_DOTNET_TICKS,
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
