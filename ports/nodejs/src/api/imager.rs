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


// pub fn hello_world(
//     env: NapiEnv,
//     arg1: NapiValue,
// ) -> Result<NapiValue, String> {
//     type Input = String;
//     type Output = Result<String, String>;
//     let input: Input = from_napi_value::<String>(env, arg1)?;
//     fn compute(path: Input) -> Output {
//         Ok(String::from("hello world -imager"))
//     }
//     fn finalize(env: NapiEnv, out: Output) -> Result<NapiValue, NapiValue> {
//         out .and_then(|x| {
//                 Ok(to_napi_value(env, x))
//             })
//             .map_err(|x| to_napi_value(env, x))
//     }
//     offload_work!(env, input, Input, Output, compute, finalize)
// }


