#![allow(unused)]
pub mod ffi;

use std::convert::AsRef;
use std::path::{PathBuf, Path};
use std::ffi::{CString, c_void};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float};
use crate::ffi::{
    WebPConfig,
    WebPPicture,
};


pub fn encoder_config(q: f32) -> WebPConfig {
    let mut config: WebPConfig = unsafe {
        std::mem::zeroed()
    };
    unsafe {
        ffi::cbits::encoder::webp_config_init(&mut config);
        ffi::cbits::encoder::webp_validate_config(&mut config);
    };
    config.quality = q;
    config.lossless = 0;
    config.method = 6;
    config
}

pub fn load_image(data: Vec<u8>) -> Result<WebPPicture, String> {
    let mut picture: WebPPicture = unsafe {
        std::mem::zeroed()
    };
    let format = ::image::guess_format(&data)
        .map_err(|x| format!("{:?}", x))?;
    match format {
        ::image::ImageFormat::JPEG => {
            unsafe {
                ffi::cbits::utils::webp_picture_from_jpeg(
                    data.as_ptr(),
                    data.len() as libc::size_t,
                    &mut picture
                );
            };
        }
        ::image::ImageFormat::PNG => {
            unsafe {
                ffi::cbits::utils::webp_picture_from_png(
                    data.as_ptr(),
                    data.len() as libc::size_t,
                    &mut picture
                );
            };
        }
        _ => {
            return Err(String::from("unknown format"))
        }
    }
    Ok(picture)
}

pub fn open_image<P: AsRef<Path>>(path: P) -> Result<WebPPicture, String> {
    let data = std::fs::read(path).expect("open image failed");
    load_image(data)
}

pub fn main() {
    let input_path = PathBuf::from("assets/samples/small/low/2yV-pyOxnPw300.jpeg");
    assert!(input_path.exists());
    let config = encoder_config(75.0);
    let picture = open_image(&input_path).expect("open_image failed");
}