// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float, c_void};

///////////////////////////////////////////////////////////////////////////////
// RGBA
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Rgba {
    pub fn from_image(source: &::image::DynamicImage) -> Self {
        use image::{DynamicImage, GenericImage, GenericImageView};
        let (width, height) = source.dimensions();
        let data = source
            .to_rgba()
            .pixels()
            .flat_map(|x| x.0.to_vec())
            .collect::<Vec<_>>();
        Rgba {width, height, data}
    }
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn to_yuv420p(&self) -> Yuv420p {
        let (width, height) = (self.width, self.height);
        let data = rgb2yuv420::convert_rgb_to_yuv420p(&self.data, width, height, 1);
        Yuv420p {width: width, height: height, data}
    }
    pub fn decode_with_format(data: &Vec<u8>, format: ::image::ImageFormat) -> Self {
        use image::{DynamicImage, GenericImage, GenericImageView};
        let data = ::image::load_from_memory_with_format(data, format).expect("load image from memory");
        Rgba::from_image(&data)
    }
}


///////////////////////////////////////////////////////////////////////////////
// YUV420P
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct Yuv420p {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Yuv420p {
    pub fn decode_with_format(data: &Vec<u8>, format: ::image::ImageFormat) -> Self {
        Rgba::decode_with_format(data, format).to_yuv420p()
    }
    pub fn from_image(data: &::image::DynamicImage) -> Self {
        Rgba::from_image(data).to_yuv420p()
    }
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
}


