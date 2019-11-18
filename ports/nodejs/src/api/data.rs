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

#[repr(C)]
pub struct U8Vec(Rc<Vec<u8>>);

impl U8Vec {
    pub fn as_vec_ref(&self) -> &Vec<u8> {
        use std::borrow::Borrow;
        self.0.borrow()
    }
    pub fn new(env: NapiEnv, xs: Vec<u8>) -> Self {
        let size = xs.len();
        assert!(adjust_external_memory(env, size).is_ok());
        U8Vec(Rc::new(xs))
    }
    pub fn to_napi_value(self, env: NapiEnv) -> NapiValue {
        let go = || -> Result<NapiValue, String> {
            let js_ptr = to_external(env, self);
            let mut output = JsObject::new(env)?;
            output.insert_raw(env, "ptr", js_ptr)?;
            output.insert(env, "type", "U8Vec")?;
            Ok(output.into_raw())
        };
        go().expect("U8Vec to js ptr failed")
    }
    pub fn from_napi_value(env: NapiEnv, value: NapiValue) -> Result<Self, String> {
        if value.is_null() {
            return Err(String::from("value is null"));
        }
        let mut object = JsObject::from_raw(env, value)?;
        let js_ptr = object.get_raw(env, "ptr")?;
        if js_ptr.is_null() {
            return Err(String::from("value is null"));
        }
        let type_value = object.get::<_, String>(env, "type")?;
        if &type_value != "U8Vec" {
            return Err(format!("expecting 'U8Vec'; given: '{}'", type_value));
        }
        from_external::<Self>(env, js_ptr)
            .map(|x| U8Vec(x.0.clone()))
    }
}

