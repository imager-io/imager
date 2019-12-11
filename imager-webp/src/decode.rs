use std::ffi::{CString, c_void};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float};
use image::{
    DynamicImage,
    GenericImage,
    GenericImageView,
    RgbaImage,
    ImageBuffer,
};
use webp_dev::sys::webp::{
    self as webp_sys,
};


pub fn decode(source: &Vec<u8>) -> DynamicImage {
    let mut width: i32 = 0;
    let mut height: i32 = 0;
    let decoded = unsafe {
        webp_sys::webp_decode_rgba(
            source.as_ptr(),
            source.len(),
            &mut width,
            &mut height,
        )
    };
    assert!(!decoded.is_null());
    assert!(width != 0 && height != 0);
    let (width, height) = (width as u32, height as u32);
    let size = (width * height * 4) as usize;
    let output = unsafe {
        std::slice::from_raw_parts_mut(decoded, size).to_vec()
    };
    let media: RgbaImage = ImageBuffer::from_vec(width, height, output).expect("to ImageBuffer");
    let media = DynamicImage::ImageRgba8(media);
    media
}

