use std::ffi::{CString, c_void};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float};
use crate::ffi;

#[link(name = "cbits")]
extern {
    pub fn webp_picture_from(
        data: *const u8,
        data_size: size_t,
        picture: *mut ffi::WebPPicture,
    );
}