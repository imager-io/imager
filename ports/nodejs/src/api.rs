pub mod data;

use std::str::FromStr;
use std::cell::{RefCell, RefMut, Ref};
use std::rc::Rc;
use std::iter::FromIterator;
use std::path::PathBuf;
use std::convert::{From, TryFrom};
use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use libc::size_t;

use crate::napi::utils::*;
use crate::napi::sys::*;
use crate::offload_work;
use crate::api::data::U8Vec;


///////////////////////////////////////////////////////////////////////////////
// DEBUG
///////////////////////////////////////////////////////////////////////////////

pub fn version(
    env: NapiEnv,
) -> Result<NapiValue, String> {
    type Input = ();
    type Output = Result<String, String>;
    let input: Input = ();
    fn compute(path: Input) -> Output {
        Ok(imager::api::version())
    }
    fn finalize(env: NapiEnv, out: Output) -> Result<NapiValue, NapiValue> {
        out .and_then(|x| {
                Ok(to_napi_value(env, x))
            })
            .map_err(|x| to_napi_value(env, x))
    }
    offload_work!(env, input, Input, Output, compute, finalize)
}


///////////////////////////////////////////////////////////////////////////////
// VECTOR CONSTRUCTION
///////////////////////////////////////////////////////////////////////////////

pub fn u8vec_open(
    env: NapiEnv,
    path: NapiValue
) -> Result<NapiValue, String> {
    type Input = String;
    type Output = Result<Vec<u8>, String>;
    let input: Input = from_napi_value::<String>(env, path)?;
    fn compute(path: Input) -> Output {
        std::fs::read(path).map_err(|x| format!("{}", x))
    }
    fn finalize(env: NapiEnv, out: Output) -> Result<NapiValue, NapiValue> {
        out .map(|x| U8Vec::new(env, x).to_napi_value(env))
            .map_err(|x| to_napi_value(env, x))
    }
    offload_work!(env, input, Input, Output, compute, finalize)
}

pub fn u8vec_from_buffer(
    env: NapiEnv,
    buffer: NapiValue,
) -> Result<NapiValue, String> {
    // INIT - JS PROMISE
    let mut deferred: NapiDeferred = std::ptr::null_mut();
    let mut promise: NapiValue = std::ptr::null_mut();
    unsafe {
        let status = napi_create_promise(
            env,
            &mut deferred,
            &mut promise,
        );
        if status != NAPI_OK {
            return Err(String::from("napi_create_promise failed"));
        }
    };
    // RESOLVE
    let data = from_buffer(env, buffer)?;
    let js_value = U8Vec::new(env, data).to_napi_value(env);
    let status = unsafe {
        napi_resolve_deferred(env, deferred, js_value)
    };
    if status != NAPI_OK {
        eprintln!("napi_resolve_deferred failed!");
    }
    // DONE
    Ok(promise)
}


///////////////////////////////////////////////////////////////////////////////
// BUFFER METHODS
///////////////////////////////////////////////////////////////////////////////

pub fn u8vec_save(
    env: NapiEnv,
    buffer: NapiValue,
    path: NapiValue,
) -> Result<NapiValue, String> {
    type Input = (U8Vec, String);
    type Output = Result<(), String>;
    let input: Input = (
        U8Vec::from_napi_value(env, buffer)?,
        from_napi_value::<String>(env, path)?,
    );
    fn compute(payload: Input) -> Output {
        let (buffer, path) = payload;
        PathBuf::from(&path)
            .parent()
            .map(|x| std::fs::create_dir_all(x));
        std::fs::write(path, buffer.as_vec_ref()).map_err(|x| format!("{}", x))
    }
    fn finalize(env: NapiEnv, out: Output) -> Result<NapiValue, NapiValue> {
        out .map(|x| to_napi_value(env, x))
            .map_err(|x| to_napi_value(env, x))
    }
    offload_work!(env, input, Input, Output, compute, finalize)
}

pub fn u8vec_to_buffer(
    env: NapiEnv,
    buffer: NapiValue,
) -> Result<NapiValue, String> {
    // INIT - JS PROMISE
    let mut deferred: NapiDeferred = std::ptr::null_mut();
    let mut promise: NapiValue = std::ptr::null_mut();
    unsafe {
        let status = napi_create_promise(
            env,
            &mut deferred,
            &mut promise,
        );
        if status != NAPI_OK {
            return Err(String::from("napi_create_promise failed"));
        }
    };
    // CONVERT
    let buffer: U8Vec = U8Vec::from_napi_value(env, buffer)?;
    let js_value = to_buffer(env, buffer.as_vec_ref())?;
    // RESOLVE
    let status = unsafe {
        napi_resolve_deferred(env, deferred, js_value)
    };
    if status != NAPI_OK {
        eprintln!("napi_resolve_deferred failed!");
    }
    // DONE
    Ok(promise)
}

pub fn u8vec_opt(
    env: NapiEnv,
    image: NapiValue,
    resize: NapiValue,
) -> Result<NapiValue, String> {
    type Input = (U8Vec, imager::api::OutputSize);
    type Output = Result<Vec<u8>, String>;
    let input: Input = (
        U8Vec::from_napi_value(env, image)?,
        from_napi_value::<imager::api::OutputSize>(env, resize)?,
    );
    fn compute(payload: Input) -> Output {
        let (input_image, resize) = payload;
        imager::api::opt(input_image.as_vec_ref(), resize)
    }
    fn finalize(env: NapiEnv, out: Output) -> Result<NapiValue, NapiValue> {
        out .map(|x| U8Vec::new(env, x).to_napi_value(env))
            .map_err(|x| to_napi_value(env, x))
    }
    offload_work!(env, input, Input, Output, compute, finalize)
}




