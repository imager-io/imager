use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::iter::FromIterator;
use std::convert::{From, TryFrom};
use std::collections::{HashMap, HashSet};
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::os::raw::c_int;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use libc::size_t;
use crate::napi::sys::*;



///////////////////////////////////////////////////////////////////////////////
// ASYNC
///////////////////////////////////////////////////////////////////////////////


pub struct AsyncData<I, O> {
    pub input: RefCell<Option<I>>,
    pub deferred: NapiDeferred,
    pub output: Option<O>,
    pub finalize: fn(NapiEnv, O)->Result<NapiValue, NapiValue>,
    pub async_work_ctx: NapiAsyncWork,
}

pub unsafe fn offload_work_impl<I, O>(
    env: NapiEnv,
    input: I,
    worker: unsafe extern "C" fn(NapiEnv, *mut std::os::raw::c_void),
    finalize: fn(NapiEnv, O)->Result<NapiValue, NapiValue>,
) -> Result<NapiValue, String> {
    unsafe extern "C" fn on_complete<I, O>(env: NapiEnv, status: NapiStatus, data: *mut std::os::raw::c_void) {
        if status != NAPI_OK {
            eprintln!("offload_work_impl failed!");
            return;
        }
        assert!(!data.is_null());
        let data = Box::from_raw(data as *mut AsyncData<I, O>);
        let deferred = data.deferred;
        let output = data.output.expect("[offload_work_impl] missing output; not set!");
        match (data.finalize)(env, output) {
            Ok(x) => {
                let status = napi_resolve_deferred(env, deferred, x);
                if status != NAPI_OK {
                    eprintln!("napi_resolve_deferred failed!");
                }
            }
            Err(x) => {
                let status = napi_reject_deferred(env, deferred, x);
                if status != NAPI_OK {
                    eprintln!("napi_reject_deferred failed!");
                }
            }
        }
        assert!(!data.async_work_ctx.is_null());
        let status = napi_delete_async_work(env, data.async_work_ctx);
        if status != NAPI_OK {
            eprintln!("napi_delete_async_work failed: {:?}", debug_format_napi_status(status));
        }
    }
    // INIT - JS PROMISE
    let mut deferred: NapiDeferred = std::ptr::null_mut();
    let mut promise: NapiValue = std::ptr::null_mut();
    let status = napi_create_promise(
        env,
        &mut deferred,
        &mut promise,
    );
    if status != NAPI_OK {
        return Err(String::from("napi_create_promise failed"));
    }
    // INIT - RUST CONTEXT
    let data = AsyncData {
        input: RefCell::new(Some(input)),
        deferred,
        output: None,
        finalize,
        async_work_ctx: std::ptr::null_mut(),
    };
    // INIT - FOREIGN CONTEXT
    let data = Box::new(data);
    let data = Box::into_raw(data);
    let async_resource = std::ptr::null_mut();
    let async_resource_name = to_napi_value(env, "web_images_async_task");
    let mut async_handle: NapiAsyncWork = std::ptr::null_mut();
    let status = napi_create_async_work(
        env,
        async_resource,
        async_resource_name,
        Some(worker),
        Some(on_complete::<I, O>),
        data as *mut libc::c_void,
        &mut async_handle,
    );
    if status != NAPI_OK {
        return Err(String::from("napi_create_async_work failed"));
    }
    // SET - FOREIGN HANDLE
    let data_tmp = Box::from_raw(data as *mut AsyncData<I, O>);
    let data_tmp: &mut AsyncData<I, O> = Box::leak(data_tmp);
    data_tmp.async_work_ctx = async_handle;
    // RUN - FOREIGN HANDLE
    let status = napi_queue_async_work(env, async_handle);
    if status != NAPI_OK {
        return Err(String::from("napi_queue_async_work failed"));
    }
    // DONE
    Ok(promise)
}


/// Example:
/// ```
/// use std::cell::{RefCell, RefMut, Ref};
/// use std::rc::Rc;
/// use std::iter::FromIterator;
/// use std::path::PathBuf;
/// use std::convert::{From, TryFrom};
/// use std::collections::{HashMap, HashSet};
/// use std::ffi::{CStr, CString};
/// use std::os::raw::{c_char, c_int};
/// use serde::{Serialize, Deserialize, de::DeserializeOwned};
/// use libc::size_t;
/// 
/// use crate::napi::utils::*;
/// use crate::napi::sys::*;
/// 
/// pub fn hello_world(
///     env: NapiEnv,
///     arg1: NapiValue,
/// ) -> Result<NapiValue, String> {
///     type Input = String;
///     type Output = Result<String, String>;
///     let input: Input = from_napi_value::<String>(env, arg1)?;
///     fn compute(path: Input) -> Output {
///         Ok(String::from("hello world -imager"))
///     }
///     fn finalize(env: NapiEnv, out: Output) -> Result<NapiValue, NapiValue> {
///         out .and_then(|x| {
///                 Ok(to_napi_value(env, x))
///             })
///             .map_err(|x| to_napi_value(env, x))
///     }
///     offload_work!(env, input, Input, Output, compute, finalize)
/// }
/// ```
#[macro_export]
macro_rules! offload_work {
    ($env:expr, $input:expr, $in_ty:ty, $out_ty:ty, $worker:ident, $finalize:ident $(,)*) => {{
        use $crate::napi::utils::*;
        unsafe extern "C" fn on_exec(env: NapiEnv, data: *mut std::os::raw::c_void) {
            let f: fn($in_ty)->$out_ty = $worker;
            assert!(!data.is_null());
            let data = Box::from_raw(data as *mut AsyncData<$in_ty, $out_ty>);
            let data: &mut AsyncData<$in_ty, $out_ty> = Box::leak(data);
            let input: Option<$in_ty> = data.input.replace(None);
            let input: $in_ty = input.expect("offload_work macro and offload_work_impl failed");
            data.output = Some(f(input));
        }
        let input: $in_ty = $input;
        let output = unsafe {
            offload_work_impl($env, input, on_exec, $finalize)
        };
        output
    }};
}



///////////////////////////////////////////////////////////////////////////////
// JS OBJECTS
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
pub struct JsObject(NapiValue);

impl JsObject {
    pub fn into_raw(self) -> NapiValue {
        self.0
    }
    pub fn from_raw(env: NapiEnv, value: NapiValue) -> Result<Self, String> {
        if value.is_null() {
            return Err(String::from("invalid argument"));
        }
        let value_type = get_value_type(env, value)?;
        if value_type == NAPI_OBJECT {
            Ok(JsObject(value))
        } else {
            Err(String::from("[JsObject::from_raw] value is not an object"))
        }
    }
    pub fn new(env: NapiEnv) -> Result<Self, String> {
        unsafe {
            let mut output: NapiValue = std::mem::zeroed();
            let status = napi_create_object(env, &mut output);
            if status != NAPI_OK {
                return Err(String::from("napi_create_object failed"));
            }
            if output.is_null() {
                return Err(String::from("napi_create_object output is null"));
            }
            Ok(JsObject(output))
        }
    }
    pub fn insert_raw<K: Serialize>(&mut self, env: NapiEnv, key: K, value: NapiValue) -> Result<(), String> {
        let key = to_napi_value(env, key);
        let status = unsafe {
            napi_set_property(env, self.0, key, value)
        };
        if status != NAPI_OK {
            return Err(String::from("napi_set_property failed"));
        }
        Ok(())
    }
    pub fn get_raw<K: Serialize>(&mut self, env: NapiEnv, key: K) -> Result<NapiValue, String> {
        let key_str = serde_json::to_string(&key).map_err(|x| format!("{}", x))?;
        let key = to_napi_value(env, key);
        let mut output: NapiValue;
        let status = unsafe {
            output = std::ptr::null_mut();
            napi_get_property(env, self.0, key, &mut output)
        };
        if status != NAPI_OK {
            return Err(format!("unable to get field named {:?}", key_str));
        }
        Ok(output)
    }
    pub fn insert<K: Serialize, V: Serialize>(&mut self, env: NapiEnv, key: K, value: V) -> Result<(), String> {
        self.insert_raw(env, key, to_napi_value(env, value))
    }
    pub fn get<K: Serialize, V: DeserializeOwned>(&mut self, env: NapiEnv, key: K) -> Result<V, String> {
        from_napi_value(env, self.get_raw(env, key)?)
    }
}


///////////////////////////////////////////////////////////////////////////////
// NODE<->RUST BINARY INTEROPERABILITY
///////////////////////////////////////////////////////////////////////////////

pub fn from_buffer(env: NapiEnv, value: NapiValue) -> Result<Vec<u8>, String> {
    // CHECK
    let mut is_buffer: bool = false;
    if get_value_type(env, value)? != NAPI_OBJECT {
        return Err(String::from("invalid type; expecting Buffer object"));
    }
    unsafe {
        if napi_is_buffer(env, value, &mut is_buffer) != NAPI_OK {
            return Err(String::from("napi_is_buffer failed"));
        }
    };
    if !is_buffer {
        return Err(String::from("invalid type; expecting Buffer object"));
    }
    // GET DATA
    let mut js_ptr: *mut libc::c_void = std::ptr::null_mut();
    let mut js_ptr_len: size_t = unsafe {
        std::mem::zeroed()
    };
    unsafe {
        let s = napi_get_buffer_info(
            env,
            value,
            &mut js_ptr,
            &mut js_ptr_len,
        );
        if s != NAPI_OK {
            return Err(String::from("napi_get_buffer_info failed"));
        }
    };
    if js_ptr.is_null() {
        return Err(String::from("napi_get_buffer_info failed (ptr is NULL)"));
    }
    // COPY
    let mut data = Vec::<u8>::with_capacity(js_ptr_len as usize);
    unsafe {
        data.set_len(js_ptr_len as usize);
    };
    unsafe {
        let js_ptr = js_ptr as *mut u8;
        std::ptr::copy(js_ptr, data.as_mut_ptr(), js_ptr_len as usize);
    };
    // DONE
    Ok(data)
}

pub fn to_buffer(env: NapiEnv, value: &Vec<u8>) -> Result<NapiValue, String> {
    let mut js_value: NapiValue = std::ptr::null_mut();
    unsafe {
        let s = napi_create_buffer_copy(
            env,
            value.len() as size_t,
            value.as_ptr() as *const libc::c_void,
            &mut std::ptr::null_mut(),
            &mut js_value
        );
        if s != NAPI_OK {
            return Err(String::from("napi_create_buffer_copy failed"));
        }
    };
    Ok(js_value)
}


///////////////////////////////////////////////////////////////////////////////
// MISC
///////////////////////////////////////////////////////////////////////////////

pub fn to_external<T>(env: NapiEnv, data: T) -> NapiValue {
    pub unsafe extern "C" fn finalize_value<T>(
        env: NapiEnv,
        finalize_data: *mut std::os::raw::c_void,
        finalize_hint: *mut std::os::raw::c_void,
    ) {
        // println!("finalize_value: called");
        assert!(!finalize_data.is_null());
        let data = Box::from_raw(finalize_data as *mut T);
        std::mem::drop(data);
        // println!("finalize_value: done!");
    }
    unsafe {
        let data = Box::new(data);
        let data = Box::into_raw(data);
        let mut output = std::mem::zeroed();
        let status = napi_create_external(
            env,
            data as *mut libc::c_void,
            Some(finalize_value::<T>),
            std::ptr::null_mut(),
            &mut output,
        );
        assert!(status == NAPI_OK);
        assert!(!output.is_null());
        output
    }
}


/// This is a function that “seems to work”.
/// 
/// The return value is a static reference because the data is effectively
/// owned by the JS GC. Deallocating such causes all sorts of issues because
/// each 'External' created from `to_extern` attaches a callback to be invoked
/// by the JS GC when such is scheduled for cleanup (this is what we want).
/// 
/// But with regards to the returned `&’static T` reference. Some interactions
/// between JS<->Rust causes issues that I have yet to understand when the
/// reference outlives the top-level native or rust function call...
/// 
/// In general this should be “safe” for the duration of the native function
/// call (please let me know if this assumption is incorrect!). For references
/// that persist longer than the function call, you should probably just `clone`
/// the data and pass by value instead. If such is too expensive, wrap it in
/// `Rc<T>` and clone that.
pub fn from_external<T>(env: NapiEnv, value: NapiValue) -> Result<&'static T, String> {
    unsafe {
        if get_value_type(env, value)? != NAPI_EXTERNAL {
            return Err(String::from("invalid type; expecting External"));
        }
        let mut output: *mut libc::c_void = std::ptr::null_mut();
        let status = napi_get_value_external(env, value, &mut output);
        if status != NAPI_OK {
            return Err(format!("napi_get_value_external failed: {}", debug_format_napi_status(status)?));
        }
        if output.is_null() {
            return Err(String::from("napi_get_value_external failed: value is null"));
        }
        let output = Box::from_raw(output as *mut T);
        Ok(Box::leak(output))
    }
}

pub fn debug_format_napi_status(s: NapiStatus) -> Result<String, String> {
    if s == NAPI_OK {
        Ok(String::from("NAPI_OK"))
    }
    else if s == NAPI_INVALID_ARG {
        Ok(String::from("NAPI_INVALID_ARG"))
    }
    else if s == NAPI_OBJECT_EXPECTED {
        Ok(String::from("NAPI_OBJECT_EXPECTED"))
    }
    else if s == NAPI_STRING_EXPECTED {
        Ok(String::from("NAPI_STRING_EXPECTED"))
    }
    else if s == NAPI_NAME_EXPECTED {
        Ok(String::from("NAPI_NAME_EXPECTED"))
    }
    else if s == NAPI_FUNCTION_EXPECTED {
        Ok(String::from("NAPI_FUNCTION_EXPECTED"))
    }
    else if s == NAPI_NUMBER_EXPECTED {
        Ok(String::from("NAPI_NUMBER_EXPECTED"))
    }
    else if s == NAPI_BOOLEAN_EXPECTED {
        Ok(String::from("NAPI_BOOLEAN_EXPECTED"))
    }
    else if s == NAPI_ARRAY_EXPECTED {
        Ok(String::from("NAPI_ARRAY_EXPECTED"))
    }
    else if s == NAPI_GENERIC_FAILURE {
        Ok(String::from("NAPI_GENERIC_FAILURE"))
    }
    else if s == NAPI_PENDING_EXCEPTION {
        Ok(String::from("NAPI_PENDING_EXCEPTION"))
    }
    else if s == NAPI_CANCELLED {
        Ok(String::from("NAPI_CANCELLED"))
    }
    else if s == NAPI_ESCAPE_CALLED_TWICE {
        Ok(String::from("NAPI_ESCAPE_CALLED_TWICE"))
    }
    else if s == NAPI_HANDLE_SCOPE_MISMATCH {
        Ok(String::from("NAPI_HANDLE_SCOPE_MISMATCH"))
    }
    else if s == NAPI_CALLBACK_SCOPE_MISMATCH {
        Ok(String::from("NAPI_CALLBACK_SCOPE_MISMATCH"))
    }
    else if s == NAPI_QUEUE_FULL {
        Ok(String::from("NAPI_QUEUE_FULL"))
    }
    else if s == NAPI_CLOSING {
        Ok(String::from("NAPI_CLOSING"))
    }
    else if s == NAPI_BIGINT_EXPECTED {
        Ok(String::from("NAPI_BIGINT_EXPECTED"))
    }
    else if s == NAPI_DATE_EXPECTED {
        Ok(String::from("NAPI_DATE_EXPECTED"))
    } else {
        Err(format!("[error] invalid status code: {}", s))
    }
}

/// Calls `napi_adjust_external_memory`.
pub fn adjust_external_memory(env: NapiEnv, size: usize) -> Result<(), ()> {
    let mut status;
    unsafe {
        let mut result: i64 = std::mem::zeroed();
        status = napi_adjust_external_memory(env, size as i64, &mut result);
    }
    if status == NAPI_OK {
        Ok(())
    } else {
        Err(())
    }
}

pub fn throw_error(env: NapiEnv, msg: &str) -> Result<(), ()> {
    unsafe {
        eprintln!("{}", msg);
        let msg = CString::new(msg).expect("CString::new failed");
        let status = napi_throw_error(env, std::ptr::null_mut(), msg.as_ptr());
        if status != NAPI_OK || status != NAPI_PENDING_EXCEPTION  {
            Err(())
        } else {
            Ok(())
        }
    }
}

pub fn throw_type_error(env: NapiEnv, msg: &str) -> Result<(), ()> {
    unsafe {
        eprintln!("{}", msg);
        let msg = CString::new(msg).expect("CString::new failed");
        let status = napi_throw_type_error(env, std::ptr::null_mut(), msg.as_ptr());
        if status != NAPI_OK || status != NAPI_PENDING_EXCEPTION  {
            Err(())
        } else {
            Ok(())
        }
    }
}


pub fn get_value_type(env: NapiEnv, value: NapiValue) -> Result<NapiValueType, String> {
    unsafe {
        let mut value_type = std::mem::zeroed();
        if napi_typeof(env, value, &mut value_type) != NAPI_OK {
            return Err(String::from("napi_typeof failed"));
        }
        Ok(value_type)
    }
}

pub fn debug_format_value_type(value_type: NapiValueType) -> Option<String> {
    if value_type == NAPI_UNDEFINED {
        Some(String::from("undefined"))
    }
    else if value_type == NAPI_NULL {
        Some(String::from("Null"))
    }
    else if value_type == NAPI_BOOLEAN {
        Some(String::from("Boolean"))
    }
    else if value_type == NAPI_NUMBER {
        Some(String::from("Number"))
    }
    else if value_type == NAPI_STRING {
        Some(String::from("String"))
    }
    else if value_type == NAPI_SYMBOL {
        Some(String::from("Symbol"))
    }
    else if value_type == NAPI_OBJECT {
        Some(String::from("Object"))
    }
    else if value_type == NAPI_FUNCTION {
        Some(String::from("Function"))
    }
    else if value_type == NAPI_EXTERNAL {
        Some(String::from("External"))
    }
    else if value_type == NAPI_BIGINT {
        Some(String::from("BigInt"))
    } else {
        None
    }
}

pub fn is_undefined_or_null(env: NapiEnv, value: NapiValue) -> Option<bool> {
    if value.is_null() {
        return Some(true);
    }
    let value_type = get_value_type(env, value).ok()?;
    if value_type == NAPI_UNDEFINED {
        Some(true)
    }
    else if value_type == NAPI_NULL {
        Some(true)
    } else {
        Some(false)
    }
}

pub fn napi_value_to_string(env: NapiEnv, value: NapiValue) -> Option<String> {
    let value_type = get_value_type(env, value).ok()?;
    if value_type != NAPI_STRING {
        return None;
    }
    let get_length = || unsafe {
        let mut output_size: usize = std::mem::zeroed();
        let status = napi_get_value_string_utf16(
            env,
            value,
            std::ptr::null_mut(),
            0,
            &mut output_size,
        );
        if status != NAPI_OK {
            None
        } else {
            Some(output_size)
        }
    };
    let mut output_length = get_length()?;
    let mut output = Vec::with_capacity(output_length + 1);
    let status = unsafe {
        let x = napi_get_value_string_utf16(
            env,
            value,
            output.as_mut_ptr(),
            output.capacity(),
            &mut output_length,
        );
        x
    };
    if status != NAPI_OK {
        return None;
    }
    let result = unsafe {
        let data = std::slice::from_raw_parts(output.as_ptr() as *const u16, output_length);
        data.to_owned()
    };
    let output = String::from_utf16(&result).ok()?;
    Some(output)
}

pub fn napi_value_to_serde_json_number(env: NapiEnv, value: NapiValue) -> Option<serde_json::Number> {
    let as_u32 = || -> Option<serde_json::Number> {
        unsafe {
            let mut output: u32 = std::mem::zeroed();
            if napi_get_value_uint32(env, value, &mut output) != NAPI_OK {
                None
            } else {
                Some(From::from(output))
            }
        }
    };
    let as_i64 = || -> Option<serde_json::Number> {
        unsafe {
            let mut output: i64 = std::mem::zeroed();
            if napi_get_value_int64(env, value, &mut output) != NAPI_OK {
                None
            } else {
                Some(From::from(output))
            }
        }
    };
    let as_f64 = || -> Option<serde_json::Number> {
        unsafe {
            let mut output: f64 = std::mem::zeroed();
            if napi_get_value_double(env, value, &mut output) != NAPI_OK {
                None
            } else {
                Some(serde_json::Number::from_f64(output)?)
            }
        }
    };
    as_u32()
        .or(as_i64())
        .or(as_f64())
}



pub fn json_stringify(env: NapiEnv, value: NapiValue) -> Result<String, ()> {
    let get_function = || unsafe {
        let mut global: NapiValue = std::mem::zeroed();
        let mut json_object: NapiValue = std::mem::zeroed();
        let mut stringify_fn: NapiValue = std::mem::zeroed();
        let json_object_name = CString::new("JSON").expect("[json_stringify] CString::new failed");
        let stringify_name = CString::new("stringify").expect("[json_stringify] CString::new failed");
        if napi_get_global(env, &mut global) != NAPI_OK {
            return None;
        }
        if napi_get_named_property(env, global, json_object_name.as_ptr(), &mut json_object) != NAPI_OK {
            return None;
        }
        if napi_get_named_property(env, json_object, stringify_name.as_ptr(), &mut stringify_fn) != NAPI_OK {
            return None;
        }
        Some((json_object, stringify_fn))
    };
    let call_function = |global: NapiValue, function: NapiValue| unsafe {
        let mut return_val: NapiValue = std::mem::zeroed();
        if napi_call_function(env, global, function, 1, &value, &mut return_val) != NAPI_OK {
            return None;
        }
        Some(return_val)
    };
    let (global, function) = get_function().ok_or(())?;
    let result = call_function(global, function).ok_or(())?;
    let result = napi_value_to_string(env, result);
    result.ok_or(())
}


pub fn to_napi_value<T: Serialize>(env: NapiEnv, value: T) -> NapiValue {
    let value = serde_json::to_value(value).expect("to_napi_value failed");
    unsafe {
        match value {
            serde_json::Value::Null => {
                let mut output = std::ptr::null_mut();
                let _ = napi_get_null(env, &mut output);
                output
            }
            serde_json::Value::Bool(x) => {
                let mut output = std::ptr::null_mut();
                let _ = napi_get_boolean(env, x, &mut output);
                output
            }
            serde_json::Value::Number(x) => {
                if x.is_f64() {
                    let input = x.as_f64().expect("should be f64");
                    let mut output = std::ptr::null_mut();
                    let _ = napi_create_double(env, input, &mut output);
                    output
                } else if x.is_i64() {
                    let input = x.as_i64().expect("should be i64");
                    let mut output = std::ptr::null_mut();
                    let _ = napi_create_int64(env, input, &mut output);
                    output
                } else if x.is_u64() {
                    let input = x.as_u64().expect("should be u64");
                    let input: Option<u32> = TryFrom::try_from(input).ok();
                    match input {
                        Some(input) => {
                            let mut output = std::ptr::null_mut();
                            let _ = napi_create_uint32(env, input, &mut output);
                            output
                        }
                        None => {
                            let msg = "64-bit unsigned integer type not yet supported by napi stable; max is u32";
                            eprintln!("{}", msg);
                            let msg = CString::new(msg).expect("CString::new failed");
                            let _ = napi_throw_type_error(env, std::ptr::null_mut(), msg.as_ptr());
                            std::ptr::null_mut()
                        }
                    }
                } else {
                    panic!("to_napi_value unreachable");
                }
            }
            serde_json::Value::String(x) => {
                let input = CString::new(x).expect("CString::new failed");
                let mut output = std::ptr::null_mut();
                let _ = napi_create_string_utf8(
                    env,
                    input.as_ptr(),
                    NAPI_AUTO_LENGTH as usize,
                    &mut output,
                );
                output
            }
            serde_json::Value::Array(xs) => {
                let mut js_array = std::ptr::null_mut();
                let _ = napi_create_array(env, &mut js_array);
                for (ix, x) in xs.into_iter().enumerate() {
                    let _ = napi_set_element(env, js_array, ix as u32, to_napi_value(env, x));
                }
                js_array
            }
            serde_json::Value::Object(xs) => {
                let mut js_object = std::ptr::null_mut();
                let _ = napi_create_object(env, &mut js_object);
                for (k, v) in xs.into_iter() {
                    let key = to_napi_value(env, k);
                    let value = to_napi_value(env, v);
                    let _ = napi_set_property(env, js_object, key, value);
                }
                js_object
            }
        }
    }
}


pub fn from_napi_value<T: DeserializeOwned>(env: NapiEnv, value: NapiValue) -> Result<T, String> {
    json_stringify(env, value)
        .map_err(|_| String::from("napi call to JSON.stringify failed"))
        .and_then(|x| serde_json::from_str(&x).map_err(|e| format!("{}", e)))
}

///////////////////////////////////////////////////////////////////////////////
// HIGHER LEVEL - ALIASES
///////////////////////////////////////////////////////////////////////////////



///////////////////////////////////////////////////////////////////////////////
// HIGHER LEVEL - OBJECTS
///////////////////////////////////////////////////////////////////////////////





///////////////////////////////////////////////////////////////////////////////
// HIGHER LEVEL - MODULE-SYSTEM - HELPERS 
///////////////////////////////////////////////////////////////////////////////

pub trait ModuleExportable<I,O>: 'static {
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue;
}

impl<Func> ModuleExportable<(NapiEnv, NapiCallbackInfo), NapiValue> for Func
where Func: Fn(NapiEnv, NapiCallbackInfo)->NapiValue+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        self(env, info)
    }
}

impl<Func> ModuleExportable<(), ()> for Func where Func: Fn(NapiEnv)+'static {
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        self(env);
        std::ptr::null_mut()
    }
}

impl<Func> ModuleExportable<NapiValue, ()> for Func where Func: Fn(NapiEnv, NapiValue)+'static {
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 1;
            let mut argc = EXPECTED_ARGC;
            let mut argv: NapiValue = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                &mut argv,
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            (self)(env, argv);
            std::ptr::null_mut()
        }
    }
}

impl<Func> ModuleExportable<(), NapiValue> for Func where Func: Fn(NapiEnv)->NapiValue+'static {
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        (self)(env)
    }
}

impl<Func> ModuleExportable<(NapiValue, NapiValue), ()> for Func
where Func: Fn(NapiEnv, NapiValue, NapiValue)+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 2;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            (self)(env, argv[0], argv[1]);
            std::ptr::null_mut()
        }
    }
}

impl<Func> ModuleExportable<(NapiValue, NapiValue, NapiValue), ()> for Func
where Func: Fn(NapiEnv, NapiValue, NapiValue, NapiValue)+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 3;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            (self)(env, argv[0], argv[1], argv[2]);
            std::ptr::null_mut()
        }
    }
}

impl<Func> ModuleExportable<(NapiValue, NapiValue, NapiValue, NapiValue), ()> for Func
where Func: Fn(NapiEnv, NapiValue, NapiValue, NapiValue, NapiValue)+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 4;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            (self)(env, argv[0], argv[1], argv[2], argv[4]);
            std::ptr::null_mut()
        }
    }
}

impl<Func> ModuleExportable<NapiValue, NapiValue> for Func
where Func: Fn(NapiEnv, NapiValue)->NapiValue+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 1;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            (self)(env, argv[0])
        }
    }
}

impl<Func> ModuleExportable<(NapiValue, NapiValue), NapiValue> for Func
where Func: Fn(NapiEnv, NapiValue, NapiValue)->NapiValue+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 2;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            (self)(env, argv[0], argv[1])
        }
    }
}

impl<Func> ModuleExportable<(NapiValue, NapiValue, NapiValue), NapiValue> for Func
where Func: Fn(NapiEnv, NapiValue, NapiValue, NapiValue)->NapiValue+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 3;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            (self)(env, argv[0], argv[1], argv[2])
        }
    }
}

impl<Func> ModuleExportable<(NapiValue, NapiValue, NapiValue, NapiValue), NapiValue> for Func
where Func: Fn(NapiEnv, NapiValue, NapiValue, NapiValue, NapiValue)->NapiValue+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 4;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            (self)(env, argv[0], argv[1], argv[2], argv[3])
        }
    }
}







impl<Func> ModuleExportable<(), Option<NapiValue>> for Func
where Func: Fn(NapiEnv)->Option<NapiValue>+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            match (self)(env) {
                Some(x) => x,
                None => std::ptr::null_mut(),
            }
        }
    }
}

impl<Func> ModuleExportable<NapiValue, Option<NapiValue>> for Func
where Func: Fn(NapiEnv, NapiValue)->Option<NapiValue>+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 1;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            match (self)(env, argv[0]) {
                Some(x) => x,
                None => std::ptr::null_mut(),
            }
        }
    }
}

impl<Func> ModuleExportable<(NapiValue, NapiValue), Option<NapiValue>> for Func
where Func: Fn(NapiEnv, NapiValue, NapiValue)->Option<NapiValue>+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 2;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            match (self)(env, argv[0], argv[1]) {
                Some(x) => x,
                None => std::ptr::null_mut(),
            }
        }
    }
}

impl<Func> ModuleExportable<(NapiValue, NapiValue, NapiValue), Option<NapiValue>> for Func
where Func: Fn(NapiEnv, NapiValue, NapiValue, NapiValue)->Option<NapiValue>+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 3;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            match (self)(env, argv[0], argv[1], argv[2]) {
                Some(x) => x,
                None => std::ptr::null_mut(),
            }
        }
    }
}

impl<Func> ModuleExportable<(NapiValue, NapiValue, NapiValue, NapiValue), Option<NapiValue>> for Func
where Func: Fn(NapiEnv, NapiValue, NapiValue, NapiValue, NapiValue)->Option<NapiValue>+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 4;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            match (self)(env, argv[0], argv[1], argv[2], argv[3]) {
                Some(x) => x,
                None => std::ptr::null_mut()
            }
        }
    }
}








impl<Func> ModuleExportable<(), Result<NapiValue, String>> for Func
where Func: Fn(NapiEnv)->Result<NapiValue, String>+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            match (self)(env) {
                Ok(x) => x,
                Err(x) => {
                    throw_error(env, &x);
                    std::ptr::null_mut()
                }
            }
        }
    }
}

impl<Func> ModuleExportable<NapiValue, Result<NapiValue, String>> for Func
where Func: Fn(NapiEnv, NapiValue)->Result<NapiValue, String>+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 1;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this: NapiValue = std::mem::zeroed();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            match (self)(env, argv[0]) {
                Ok(x) => x,
                Err(x) => {
                    throw_error(env, &x);
                    std::ptr::null_mut()
                }
            }
        }
    }
}

impl<Func> ModuleExportable<(NapiValue, NapiValue), Result<NapiValue, String>> for Func
where Func: Fn(NapiEnv, NapiValue, NapiValue)->Result<NapiValue, String>+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 2;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            match (self)(env, argv[0], argv[1]) {
                Ok(x) => x,
                Err(x) => {
                    throw_error(env, &x);
                    std::ptr::null_mut()
                }
            }
        }
    }
}

impl<Func> ModuleExportable<(NapiValue, NapiValue, NapiValue), Result<NapiValue, String>> for Func
where Func: Fn(NapiEnv, NapiValue, NapiValue, NapiValue)->Result<NapiValue, String>+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 3;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            match (self)(env, argv[0], argv[1], argv[2]) {
                Ok(x) => x,
                Err(x) => {
                    throw_error(env, &x);
                    std::ptr::null_mut()
                }
            }
        }
    }
}

impl<Func> ModuleExportable<(NapiValue, NapiValue, NapiValue, NapiValue), Result<NapiValue, String>> for Func
where Func: Fn(NapiEnv, NapiValue, NapiValue, NapiValue, NapiValue)->Result<NapiValue, String>+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 4;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            match (self)(env, argv[0], argv[1], argv[2], argv[3]) {
                Ok(x) => x,
                Err(x) => {
                    throw_error(env, &x);
                    std::ptr::null_mut()
                }
            }
        }
    }
}






impl<Func> ModuleExportable<(), Result<(), String>> for Func
where Func: Fn(NapiEnv)->Result<(), String>+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            match (self)(env) {
                Ok(x) => std::ptr::null_mut(),
                Err(x) => {
                    throw_error(env, &x);
                    std::ptr::null_mut()
                }
            }
        }
    }
}

impl<Func> ModuleExportable<NapiValue, Result<(), String>> for Func
where Func: Fn(NapiEnv, NapiValue)->Result<(), String>+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 1;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            match (self)(env, argv[0]) {
                Ok(x) => std::ptr::null_mut(),
                Err(x) => {
                    throw_error(env, &x);
                    std::ptr::null_mut()
                }
            }
        }
    }
}

impl<Func> ModuleExportable<(NapiValue, NapiValue), Result<(), String>> for Func
where Func: Fn(NapiEnv, NapiValue, NapiValue)->Result<(), String>+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 2;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            match (self)(env, argv[0], argv[1]) {
                Ok(x) => std::ptr::null_mut(),
                Err(x) => {
                    throw_error(env, &x);
                    std::ptr::null_mut()
                }
            }
        }
    }
}

impl<Func> ModuleExportable<(NapiValue, NapiValue, NapiValue), Result<(), String>> for Func
where Func: Fn(NapiEnv, NapiValue, NapiValue, NapiValue)->Result<(), String>+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 3;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            match (self)(env, argv[0], argv[1], argv[2]) {
                Ok(x) => std::ptr::null_mut(),
                Err(x) => {
                    throw_error(env, &x);
                    std::ptr::null_mut()
                }
            }
        }
    }
}

impl<Func> ModuleExportable<(NapiValue, NapiValue, NapiValue, NapiValue), Result<(), String>> for Func
where Func: Fn(NapiEnv, NapiValue, NapiValue, NapiValue, NapiValue)->Result<(), String>+'static
{
    fn call_module_export(self, env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
        unsafe {
            const EXPECTED_ARGC: usize = 4;
            let mut argc = EXPECTED_ARGC;
            let mut argv: [NapiValue; EXPECTED_ARGC] = std::mem::zeroed();
            let mut raw_this = std::ptr::null_mut();
            let result = napi_get_cb_info(
                env,
                info,
                &mut argc,
                argv.as_mut_ptr(),
                &mut raw_this,
                std::ptr::null_mut(),
            );
            if result != NAPI_OK {
                eprintln!("napi_get_cb_info failed");
                throw_error(env, "module call failed");
                return std::ptr::null_mut();
            }
            if argc != EXPECTED_ARGC {
                throw_error(env, &format!(
                    "invalid number of arguments: given {given}; expecting {expecting}",
                    given=argc,
                    expecting=EXPECTED_ARGC,
                ));
            }
            match (self)(env, argv[0], argv[1], argv[2], argv[3]) {
                Ok(x) => std::ptr::null_mut(),
                Err(x) => {
                    throw_error(env, &x);
                    std::ptr::null_mut()
                }
            }
        }
    }
}




///////////////////////////////////////////////////////////////////////////////
// HIGHER LEVEL - MODULE-SYSTEM - CORE
///////////////////////////////////////////////////////////////////////////////

pub type RawExportFnSignature =
    unsafe extern "C" fn(env: NapiEnv, info: NapiCallbackInfo) -> NapiValue;


/// Internal
#[doc(hidden)]
#[macro_export]
macro_rules! i_add_module_exports {
    ($results:ident, $add_method:ident,,) => {};
    ($results:ident, $add_method:ident,) => {};
    ($results:ident, $add_method:ident, $name:ident => $value:path, $($rest:tt)*) => {{
        pub unsafe extern fn entry(env: NapiEnv, info: NapiCallbackInfo) -> NapiValue {
            use $crate::napi::utils::ModuleExportable;
            ModuleExportable::call_module_export($value, env, info)
        }
        $results.push($add_method(stringify!($name), entry));
        i_add_module_exports!($results, $add_method, $($rest)*);
    }};
    ($results:ident, $add_method:ident, $name:ident, $($rest:tt)*) => {{
        pub unsafe extern fn entry(env: napi_env, info: NapiCallbackInfo) -> NapiValue {
            use $crate::napi::utils::ModuleExportable;
            ModuleExportable::call_module_export($name, env, info)
        }
        $results.push($add_method(stringify!($name), entry));
        i_add_module_exports!($results, $add_method, $($rest)*);
    }};
    ($results:ident, $add_method:ident, $name:ident => $value:path) => {{
        pub unsafe extern fn entry(env: napi_env, info: NapiCallbackInfo) -> NapiValue {
            use $crate::napi::utils::ModuleExportable;
            ModuleExportable::call_module_export($value, env, info)
        }
        $results.push($add_method(stringify!($name), entry));
    }};
    ($results:ident, $add_method:ident, $name:ident) => {{
        pub unsafe extern fn entry(env: napi_env, info: NapiCallbackInfo) -> NapiValue {
            use $crate::napi::utils::ModuleExportable;
            ModuleExportable::call_module_export($name, env, info)
        }
        $results.push($add_method(stringify!($name), entry));
    }};
}


/// A relatively higher level interface for registering exposed library functions.
/// Works with any function that implements `ModuleExportable`.
/// 
/// ```
/// fn alpha(env: NapiEnv) {}
/// fn beta(env: NapiEnv, arg: NapiValue) -> NapiValue {str::ptr::null_mut()}
/// fn gamma(env: NapiEnv) {}
/// 
/// library_exports!{
///     omega => alpha,
///     beta => beta,
///     gamma,
/// }
/// ```
#[macro_export]
macro_rules! library_exports {
    ($($x:tt)*) => {
        #[no_mangle]
        #[cfg_attr(target_os = "linux", link_section = ".ctors")]
        #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
        #[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
        pub static __REGISTER_MODULE: extern "C" fn() = {
            use std::io::Write;
            use std::os::raw::c_char;
            use std::ptr;
            use napi_sys_dev::*;
            use $crate::napi::sys::*;
            // use $crate::*;

            #[no_mangle]
            pub unsafe extern fn init(env: napi_env, export: NapiValue) -> NapiValue {
                
                use $crate::napi::utils::RawExportFnSignature;
                let mut results: Vec<Result<(), ()>> = Vec::new();
                let add_method = |name: &str, callback: RawExportFnSignature| -> Result<(), ()> {
                    let callback_name = std::ffi::CString::new(name).expect("CString::new failed");
                    let mut js_function: NapiValue = std::mem::zeroed();
                    let status = napi_create_function(
                        env,
                        std::ptr::null(),
                        napi_sys_dev::NAPI_AUTO_LENGTH as usize,
                        Some(callback),
                        std::ptr::null_mut(),
                        &mut js_function
                    );
                    if status != NAPI_OK {
                        return Err(());
                    }
                    let status = napi_set_named_property(
                        env,
                        export,
                        callback_name.as_ptr(),
                        js_function
                    );
                    if status != NAPI_OK {
                        return Err(());
                    }
                    Ok(())
                };
                i_add_module_exports!(results, add_method, $($x)*);
                if results.iter().all(|x| x.is_ok()) {
                    export
                } else {
                    std::ptr::null_mut()
                }
            }

            extern "C" fn register_module() {
                static mut MODULE_DESCRIPTOR: Option<NapiModule> = None;
                unsafe {
                    MODULE_DESCRIPTOR = Some(NapiModule {
                        nm_version: 1,
                        nm_flags: 0,
                        nm_filename: concat!(file!(), "\0").as_ptr() as *const c_char,
                        nm_register_func: Some(init),
                        nm_modname: concat!(stringify!(index), "\0").as_ptr() as *const c_char,
                        nm_priv: 0 as *mut _,
                        reserved: [0 as *mut _; 4],
                    });
                    napi_module_register(MODULE_DESCRIPTOR.as_mut().unwrap() as *mut NapiModule);
                }
            }

            register_module
        };
    };
}
