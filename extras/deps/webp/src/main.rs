#![allow(unused)]
use std::ffi::{CString, c_void};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float};

use webp_dev::{
    WebPConfig,
    // WebPConfigInit,
    webp_config_init,
    webp_config_preset,
};

pub fn init_config(q: u8) -> WebPConfig {
    let mut config: WebPConfig = unsafe {
        std::mem::zeroed()
    };
    unsafe {
        webp_config_init(&mut config);
    };
    config.quality = q as f32;
    config.method = 6;
    config
}

fn main() {
    println!("Hello, world!");
}
