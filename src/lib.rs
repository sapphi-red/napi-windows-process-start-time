#![no_std]

use core::ffi::{c_char, c_void};
use core::ptr;

type Handle = *mut c_void;
type Env = *mut c_void;
type Value = *mut c_void;
type Info = *mut c_void;
type Status = i32;
type Callback = unsafe extern "C" fn(Env, Info) -> Value;

#[repr(C)]
#[derive(Default)]
struct FileTime {
    low: u32,
    high: u32,
}

#[link(name = "kernel32")]
extern "system" {
    fn OpenProcess(access: u32, inherit: i32, pid: u32) -> Handle;
    fn GetProcessTimes(
        process: Handle,
        creation: *mut FileTime,
        exit: *mut FileTime,
        kernel: *mut FileTime,
        user: *mut FileTime,
    ) -> i32;
    fn WaitForSingleObject(handle: Handle, timeout: u32) -> u32;
    fn CloseHandle(handle: Handle) -> i32;
    fn GetModuleHandleW(name: *const u16) -> Handle;
    fn GetProcAddress(module: Handle, name: *const c_char) -> *mut c_void;
    fn GetCurrentProcess() -> Handle;
    fn TerminateProcess(process: Handle, code: u32) -> i32;
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe {
        TerminateProcess(GetCurrentProcess(), 0xC000_0409);
    }
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub extern "system" fn DllMain(_: Handle, _: u32, _: *mut c_void) -> i32 {
    1
}

// Resolve the small subset of Node-API we actually use. No globals, allocations,
// import library, or Rust runtime are required, including in Node workers.
macro_rules! symbol {
    ($module:expr, $name:literal, $ty:ty) => {{
        let address = GetProcAddress($module, concat!($name, "\0").as_ptr().cast());
        if address.is_null() {
            return ptr::null_mut();
        }
        core::mem::transmute::<*mut c_void, $ty>(address)
    }};
}

struct ProcessHandle(Handle);
impl Drop for ProcessHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

fn process_ticks(pid: u32) -> Option<u64> {
    let handle = unsafe { OpenProcess(0x1000 | 0x0010_0000, 0, pid) };
    if handle.is_null() {
        return None;
    }
    let handle = ProcessHandle(handle);
    if unsafe { WaitForSingleObject(handle.0, 0) } != 0x102 {
        return None;
    }
    let mut creation = FileTime::default();
    let mut exit = FileTime::default();
    let mut kernel = FileTime::default();
    let mut user = FileTime::default();
    if unsafe {
        GetProcessTimes(
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
    Some(((u64::from(creation.high) << 32) | u64::from(creation.low)) + 504_911_232_000_000_000)
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    reason = "PID validation checks the uint32 range and exact integer round trip before use"
)]
unsafe extern "C" fn get_process_start_time(env: Env, info: Info) -> Value {
    let module = GetModuleHandleW(ptr::null());
    let get_info = symbol!(
        module,
        "napi_get_cb_info",
        unsafe extern "C" fn(
            Env,
            Info,
            *mut usize,
            *mut Value,
            *mut Value,
            *mut *mut c_void,
        ) -> Status
    );
    let get_number = symbol!(
        module,
        "napi_get_value_double",
        unsafe extern "C" fn(Env, Value, *mut f64) -> Status
    );
    let throw_type_error = symbol!(
        module,
        "napi_throw_type_error",
        unsafe extern "C" fn(Env, *const c_char, *const c_char) -> Status
    );
    let mut argc = 1;
    let mut arg = ptr::null_mut();
    if get_info(
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
    let mut pid = 0.0;
    if argc == 0
        || get_number(env, arg, &raw mut pid) != 0
        || !pid.is_finite()
        || pid < 0.0
        || pid > f64::from(u32::MAX)
        || pid != f64::from(pid as u32)
    {
        throw_type_error(env, ptr::null(), c"pid must be a uint32".as_ptr());
        return ptr::null_mut();
    }
    let mut result = ptr::null_mut();
    if let Some(mut ticks) = process_ticks(pid as u32) {
        // Six prefix bytes and at most 20 decimal digits for a u64.
        let mut buffer = [0u8; 26];
        let mut cursor = buffer.len();
        loop {
            cursor -= 1;
            buffer[cursor] = b'0' + (ticks % 10) as u8;
            ticks /= 10;
            if ticks == 0 {
                break;
            }
        }
        cursor -= 6;
        buffer[cursor..cursor + 6].copy_from_slice(b"win32:");
        let create_string = symbol!(
            module,
            "napi_create_string_utf8",
            unsafe extern "C" fn(Env, *const c_char, usize, *mut Value) -> Status
        );
        if create_string(
            env,
            buffer.as_ptr().add(cursor).cast(),
            buffer.len() - cursor,
            &raw mut result,
        ) != 0
        {
            return ptr::null_mut();
        }
    } else {
        let get_null = symbol!(
            module,
            "napi_get_null",
            unsafe extern "C" fn(Env, *mut Value) -> Status
        );
        if get_null(env, &raw mut result) != 0 {
            return ptr::null_mut();
        }
    }
    result
}

#[no_mangle]
pub extern "C" fn node_api_module_get_api_version_v1() -> i32 {
    1
}

#[no_mangle]
/// Initializes an addon environment when Node.js loads this DLL.
///
/// # Safety
/// Node.js must provide a valid environment and exports object for the current
/// thread. The callback pointers registered here use the Node-API C ABI.
pub unsafe extern "C" fn napi_register_module_v1(env: Env, exports: Value) -> Value {
    let module = GetModuleHandleW(ptr::null());
    let create_function = symbol!(
        module,
        "napi_create_function",
        unsafe extern "C" fn(
            Env,
            *const c_char,
            usize,
            Callback,
            *mut c_void,
            *mut Value,
        ) -> Status
    );
    let set_property = symbol!(
        module,
        "napi_set_named_property",
        unsafe extern "C" fn(Env, Value, *const c_char, Value) -> Status
    );
    let name = b"getProcessStartTime\0";
    let mut function = ptr::null_mut();
    if create_function(
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
    if set_property(env, exports, name.as_ptr().cast(), function) != 0 {
        return ptr::null_mut();
    }
    exports
}
