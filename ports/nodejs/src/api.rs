pub mod data;

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
use crate::api::data::*;


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
// BUFFER CONSTRUCTION
///////////////////////////////////////////////////////////////////////////////

pub fn buffer_open(
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
        out .map(|x| Buffer::new(env, x).to_napi_value(env))
            .map_err(|x| to_napi_value(env, x))
    }
    offload_work!(env, input, Input, Output, compute, finalize)
}


///////////////////////////////////////////////////////////////////////////////
// BUFFER METHODS
///////////////////////////////////////////////////////////////////////////////

pub fn buffer_save(
    env: NapiEnv,
    buffer: NapiValue,
    path: NapiValue,
) -> Result<NapiValue, String> {
    type Input = (Buffer, String);
    type Output = Result<(), String>;
    let input: Input = (
        Buffer::from_napi_value(env, buffer)?,
        from_napi_value::<String>(env, path)?,
    );
    fn compute(payload: Input) -> Output {
        let (buffer, path) = payload;
        std::fs::write(path, buffer.as_vec_ref())
            .map_err(|x| format!("{}", x))
    }
    fn finalize(env: NapiEnv, out: Output) -> Result<NapiValue, NapiValue> {
        out .map(|x| to_napi_value(env, x))
            .map_err(|x| to_napi_value(env, x))
    }
    offload_work!(env, input, Input, Output, compute, finalize)
}

pub fn buffer_opt(
    env: NapiEnv,
    image: NapiValue,
    resize: NapiValue,
) -> Result<NapiValue, String> {
    type Input = (Buffer, imager::api::OutputSize);
    type Output = Result<Vec<u8>, String>;
    let input: Input = (
        Buffer::from_napi_value(env, image)?,
        from_napi_value::<imager::api::OutputSize>(env, resize)?,
    );
    fn compute(payload: Input) -> Output {
        let (input_image, resize) = payload;
        imager::api::opt(input_image.as_vec_ref(), resize)
    }
    fn finalize(env: NapiEnv, out: Output) -> Result<NapiValue, NapiValue> {
        out .map(|x| Buffer::new(env, x).to_napi_value(env))
            .map_err(|x| to_napi_value(env, x))
    }
    offload_work!(env, input, Input, Output, compute, finalize)
}




