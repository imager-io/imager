use std::ffi::{CString, c_void};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float};
use image::{DynamicImage, GenericImage, GenericImageView};
use webp_dev::sys::webp::{
    self as webp_sys,
    WebPConfig,
    WebPPicture,
    WebPMemoryWriter,
};

pub fn init_config(q: f32) -> WebPConfig {
    let mut config: WebPConfig = unsafe {std::mem::zeroed()};
    unsafe {
        webp_sys::webp_config_init(&mut config);
        webp_sys::webp_validate_config(&mut config);
    };
    config.quality = q;
    config.lossless = 0;
    config.method = 6;
    config
}

pub fn init_picture(source: &DynamicImage) -> (WebPPicture, *mut WebPMemoryWriter) {
    // SETUP
    let (mut picture, writer) = crate::encode::lossless::init_picture(source);
    // CONVERT
    unsafe {
        assert!(webp_sys::webp_picture_sharp_argb_to_yuva(&mut picture) != 0);
        assert!(picture.use_argb == 0);
        assert!(!picture.y.is_null());
    };
    // DONE
    (picture, writer)
}

pub fn encode(source: &DynamicImage, q: f32) -> Vec<u8> {
    let config = init_config(q);
    let (mut picture, writer_ptr) = init_picture(&source);
    unsafe {
        assert!(webp_sys::webp_encode(&config, &mut picture) != 0);
    };
    // COPY OUTPUT
    let mut writer = unsafe { Box::from_raw(writer_ptr) };
    let mut output: Vec<u8> = unsafe {
        std::slice::from_raw_parts_mut(writer.mem, writer.size).to_vec()
    };
    // CLEANUP PICTURE & WRITER
    unsafe {
        webp_sys::webp_picture_free(&mut picture);
        webp_sys::webp_memory_writer_clear(writer_ptr);
        std::mem::drop(picture);
        std::mem::drop(writer_ptr);
        std::mem::drop(writer);
    };
    // DONE
    output
}

