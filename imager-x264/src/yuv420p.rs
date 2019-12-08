// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::convert::AsRef;
use std::path::{PathBuf, Path};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float, c_void};
use image::{DynamicImage, GenericImage, GenericImageView};
use webp_dev::sys::webp::{
    self as webp_sys,
    WebPConfig,
    WebPPicture,
    WebPMemoryWriter,
};

#[derive(Debug, Clone)]
pub struct Yuv420P {
    pub y: Vec<u8>,
    pub u: Vec<u8>,
    pub v: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Yuv420P {
    pub fn open<P: AsRef<Path>>(path: P) -> Self {
        let media = ::image::open(path).expect("open image failed");
        Yuv420P::from_image(&media)
    }
    pub fn from_image(source: &DynamicImage) -> Self {
        image_convert_pixels_using_webp(source)
    }
}

fn image_convert_pixels_using_webp(source: &DynamicImage) -> Yuv420P {
    let (width, height) = source.dimensions();
    assert!(width < webp_sys::WEBP_MAX_DIMENSION);
    assert!(height < webp_sys::WEBP_MAX_DIMENSION);
    let mut picture: WebPPicture = unsafe {std::mem::zeroed()};
    unsafe {
        assert!(webp_sys::webp_picture_init(&mut picture) != 0);
    };
    let argb_stride = width;
    picture.use_argb = 1;
    picture.width = width as i32;
    picture.height = height as i32;
    picture.argb_stride = argb_stride as i32;
    // FILL PIXEL BUFFERS
    unsafe {
        let mut pixel_data = source
            .to_rgb()
            .pixels()
            .flat_map(|px: &::image::Rgb<u8>| px.0.to_vec())
            .collect::<Vec<_>>();
        let full_stride = argb_stride * 3;
        let status = webp_sys::webp_picture_import_rgb(
            &mut picture,
            pixel_data.as_mut_ptr(),
            full_stride as i32,
        );
        // CHECKS
        let expected_size = argb_stride * height * 3;
        assert!(pixel_data.len() as u32 == expected_size);
        assert!(status != 0);
        // CLEANUP
        std::mem::drop(pixel_data);
    };
    // CHECKS
    assert!(picture.use_argb == 1);
    assert!(picture.y.is_null());
    assert!(!picture.argb.is_null());
    // CONVERT
    unsafe {
        assert!(webp_sys::webp_picture_sharp_argb_to_yuva(&mut picture) != 0);
        assert!(picture.use_argb == 0);
        assert!(!picture.y.is_null());
    };
    let (y, u, v) = unsafe {
        assert!(picture.y_stride as u32 == width);
        assert!(picture.uv_stride as u32 == width / 2);
        let y_size = width * height;
        let uv_size = width * height / 4;
        let y = std::slice::from_raw_parts_mut(picture.y, y_size as usize).to_vec();
        let u = std::slice::from_raw_parts_mut(picture.u, uv_size as usize).to_vec();
        let v = std::slice::from_raw_parts_mut(picture.v, uv_size as usize).to_vec();
        (y, u, v)
    };
    // CLEANUP
    unsafe {
        webp_sys::webp_picture_free(&mut picture);
    };
    std::mem::drop(picture);
    // DONE
    Yuv420P {y, u, v, width, height}
}

