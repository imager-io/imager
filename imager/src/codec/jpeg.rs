// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::path::PathBuf;
use std::convert::From;
use image::{
    GenericImage,
    DynamicImage,
    GenericImageView,
    FilterType,
    ColorType,
    Pixel,
};
// use rayon::prelude::*;
// use either::{Either, Either::*};


///////////////////////////////////////////////////////////////////////////////
// MOZJPEG (INTERNAL) FFI HELPERS
///////////////////////////////////////////////////////////////////////////////

#[allow(non_snake_case)]
const TRUE: mozjpeg_sys::boolean = true as mozjpeg_sys::boolean;
#[allow(non_snake_case)]
const FALSE: mozjpeg_sys::boolean = false as mozjpeg_sys::boolean;

const COLOR_SPACE: mozjpeg_sys::J_COLOR_SPACE = mozjpeg_sys::J_COLOR_SPACE::JCS_RGB;
const COLOR_SPACE_COMPONENTS: libc::c_int = 3 as libc::c_int;


///////////////////////////////////////////////////////////////////////////////
// MOZJPEG  DECODER
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub linesize: u32,
    pub buffer: Vec<u8>,
}

impl DecodedImage {
    pub fn load_from_memory(buffer: &[u8]) -> Result<Self, String> {
        let format = ::image::guess_format(buffer)
            .map_err(|x| format!("{}", x))?;
        if format == ::image::ImageFormat::JPEG {
            Ok(DecodedImage::mozjpeg_decode(&buffer))
        } else {
            let image = ::image::load_from_memory(buffer)
                .map_err(|x| format!("{}", x))?;
            DecodedImage::from_image(&image)
        }
    }
    pub fn from_image(source: &DynamicImage) -> Result<Self, String> {
        let source = crate::codec::jpeg_utils::native_encode_as_jpeg(source, 100);
        Ok(DecodedImage::mozjpeg_decode(&source))
    }
    pub fn open(path: &PathBuf) -> Result<Self, String> {
        let buffer = std::fs::read(path)
            .map_err(|x| format!("{}", x))?;
        DecodedImage::load_from_memory(&buffer)
    }
    fn mozjpeg_decode(file: &[u8]) -> DecodedImage {
        use libc;
        use mozjpeg_sys::*;
        use std::mem;
        use std::ffi::CString;
        
        // SETUP DECODER CONTEXT
        let (mut err, mut cinfo) = unsafe {
            let mut err: jpeg_error_mgr = mem::zeroed();
            let mut cinfo: jpeg_decompress_struct = mem::zeroed();
            cinfo.common.err = jpeg_std_error(&mut err);
            jpeg_create_decompress(&mut cinfo);
            (err, cinfo)
        };

        // INIT
        unsafe {
            jpeg_mem_src(&mut cinfo, file.as_ptr(), file.len() as c_ulong);
            jpeg_read_header(&mut cinfo, TRUE);
        };

        // SOURCE INFO
        let width = cinfo.image_width;
        let height = cinfo.image_height;
        cinfo.out_color_space = COLOR_SPACE;
        // cinfo.do_block_smoothing = TRUE;
        unsafe {
            jpeg_start_decompress(&mut cinfo);
        };

        // DECODED OUTPUT MEMORY
        let row_stride = cinfo.image_width as usize * cinfo.output_components as usize;
        let buffer_size = row_stride * cinfo.image_height as usize;
        let mut buffer = vec![0u8; buffer_size];

        // GO!
        while cinfo.output_scanline < cinfo.output_height {
            let offset = cinfo.output_scanline as usize * row_stride;
            let mut jsamparray = [buffer[offset..].as_mut_ptr()];
            unsafe {
                jpeg_read_scanlines(&mut cinfo, jsamparray.as_mut_ptr(), 1);
            };
        }

        // CLEANUP 
        unsafe {
            jpeg_finish_decompress(&mut cinfo);
            jpeg_destroy_decompress(&mut cinfo);
        };
        // DONE
        DecodedImage {
            buffer,
            width,
            height,
            linesize: row_stride as u32,
        }
    }
}



///////////////////////////////////////////////////////////////////////////////
// MOZJPEG (INTERNAL) ENCODER
///////////////////////////////////////////////////////////////////////////////

fn mozjpeg_encode(buffer: &[u8], width: u32, height: u32, quality: u8) -> Vec<u8> {
    use libc;
    use std::mem;
    use std::ffi::CString;

    // SETUP ENCODER CONTEXT
    let (mut err, mut cinfo, mut outbuffer) = unsafe {
        let mut err = unsafe {mem::zeroed()};
        let mut cinfo: mozjpeg_sys::jpeg_compress_struct = unsafe {mem::zeroed()};
        let mut outbuffer: *mut libc::c_uchar = unsafe {std::ptr::null_mut()};
        (err, cinfo, outbuffer)
    };
    let mut outsize: libc::c_ulong = 0;
    unsafe {
        cinfo.common.err = unsafe {mozjpeg_sys::jpeg_std_error(&mut err)};
        mozjpeg_sys::jpeg_create_compress(&mut cinfo);
        mozjpeg_sys::jpeg_mem_dest(&mut cinfo, &mut outbuffer, &mut outsize);
    };

    // ENCODER CONFIG - PRE-INIT (REQUIRED)
    cinfo.image_width = width;
    cinfo.image_height = height;
    cinfo.input_components = COLOR_SPACE_COMPONENTS;
    let row_stride = cinfo.image_width as usize * cinfo.input_components as usize;
    cinfo.in_color_space = COLOR_SPACE;
    // ENCODER CONFIG - SET DEFAULTS (REQUIRED)
    unsafe {
        mozjpeg_sys::jpeg_set_defaults(&mut cinfo)
    }
    // ENCODER CONFIG - MISC
    cinfo.dct_method = mozjpeg_sys::J_DCT_METHOD::JDCT_ISLOW;
    cinfo.write_JFIF_header = FALSE;
    cinfo.optimize_coding = TRUE;
    unsafe {
        mozjpeg_sys::jpeg_simple_progression(&mut cinfo);
    };
    unsafe {
        mozjpeg_sys::jpeg_c_set_bool_param(&mut cinfo, mozjpeg_sys::JBOOLEAN_USE_SCANS_IN_TRELLIS, TRUE);
        mozjpeg_sys::jpeg_c_set_bool_param(&mut cinfo, mozjpeg_sys::JBOOLEAN_USE_LAMBDA_WEIGHT_TBL, TRUE);
        // mozjpeg_sys::jpeg_c_set_bool_param(&mut cinfo, mozjpeg_sys::JBOOLEAN_TRELLIS_EOB_OPT, TRUE);
    };
    // ENCODER CONFIG - QUALITY
    unsafe {
        mozjpeg_sys::jpeg_set_quality(&mut cinfo, quality as i32, TRUE)
    }
    // GO!
    unsafe {
        mozjpeg_sys::jpeg_start_compress(&mut cinfo, TRUE)
    };
    while cinfo.next_scanline < cinfo.image_height {
        let offset = cinfo.next_scanline as usize * row_stride;
        let jsamparray = [buffer[offset..].as_ptr()];
        unsafe {
            mozjpeg_sys::jpeg_write_scanlines(&mut cinfo, jsamparray.as_ptr(), 1);
        };
    }
    unsafe {
        mozjpeg_sys::jpeg_finish_compress(&mut cinfo);
        mozjpeg_sys::jpeg_destroy_compress(&mut cinfo);
    };
    let output_data = {
        let res = unsafe {
            std::slice::from_raw_parts(outbuffer, outsize as usize).to_vec()
        };
        // FREE MEMORY DEST
        if !outbuffer.is_null() {
            unsafe {
                libc::free(outbuffer as *mut mozjpeg_sys::c_void);
            };
            outbuffer = std::ptr::null_mut();
            outsize = 0;
        }
        res
    };
    output_data
}



///////////////////////////////////////////////////////////////////////////////
// EXTERNAL ENCODER API
///////////////////////////////////////////////////////////////////////////////

pub fn encode(source: &DecodedImage, quality: u8) -> Vec<u8> {
    let encoded = mozjpeg_encode(&source.buffer, source.width, source.height, quality);
    encoded
}

