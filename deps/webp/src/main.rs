#![allow(unused)]
use std::ffi::{CString, c_void};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float};

use webp_dev::{
    WebPConfig,
    // WebPConfigInit,
    webp_config_init
};

fn main() {
    let mut picture: *mut WebPConfig = std::ptr::null_mut();
    // WebPConfigInit(picture);
    println!("Hello, world!");
}
