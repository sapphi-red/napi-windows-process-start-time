use core::ffi::{c_char, c_void};

use crate::win32;

pub type Env = *mut c_void;
pub type Value = *mut c_void;
pub type Info = *mut c_void;
type Status = i32;
type Callback = unsafe extern "C" fn(Env, Info) -> Value;

// Resolve the small subset of Node-API we actually use. No globals, allocations,
// import library, or Rust runtime are required, including in Node workers.
macro_rules! symbol {
    ($(fn $name:ident($($arg:ident: $ty:ty),* $(,)?) -> Status;)+) => {
        $(pub unsafe fn $name($($arg: $ty),*) -> Status {
            let module = win32::get_module_handle();
            // SAFETY: An identifier contains no NUL bytes, and concat! appends the terminator.
            let name = unsafe {
                core::ffi::CStr::from_bytes_with_nul_unchecked(concat!(stringify!($name), "\0").as_bytes())
            };
            let Some(address) = win32::get_proc_address(&module, name) else {
                return 9; // napi_generic_failure
            };
            // SAFETY: Each declaration matches the named Node-API export's C ABI and signature.
            let function = unsafe {
                core::mem::transmute::<
                    unsafe extern "system" fn() -> isize,
                    unsafe extern "C" fn($($ty),*) -> Status,
                >(address)
            };
            // SAFETY: The caller must uphold the declared Node-API function's requirements.
            unsafe { function($($arg),*) }
        })+
    };
}

symbol! {
    fn napi_get_cb_info(
        env: Env,
        info: Info,
        argc: *mut usize,
        argv: *mut Value,
        this_arg: *mut Value,
        data: *mut *mut c_void,
    ) -> Status;
    fn napi_get_value_uint32(env: Env, value: Value, result: *mut u32) -> Status;
    fn napi_throw_type_error(env: Env, code: *const c_char, message: *const c_char) -> Status;
    fn napi_create_bigint_uint64(env: Env, value: u64, result: *mut Value) -> Status;
    fn napi_get_null(env: Env, result: *mut Value) -> Status;
    fn napi_create_function(
        env: Env,
        name: *const c_char,
        length: usize,
        callback: Callback,
        data: *mut c_void,
        result: *mut Value,
    ) -> Status;
    fn napi_set_named_property(env: Env, object: Value, name: *const c_char, value: Value) -> Status;
}
