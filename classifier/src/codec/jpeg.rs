// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::path::PathBuf;
use std::convert::From;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float, c_void};
use serde::{Serialize, Deserialize};
use image::{
    GenericImage,
    DynamicImage,
    GenericImageView,
    FilterType,
    ColorType,
    Pixel,
};
use rayon::prelude::*;
use itertools::Itertools;

///////////////////////////////////////////////////////////////////////////////
// MOZJPEG FFI HELPERS
///////////////////////////////////////////////////////////////////////////////

#[allow(non_snake_case)]
const TRUE: mozjpeg_sys::boolean = true as mozjpeg_sys::boolean;
#[allow(non_snake_case)]
const FALSE: mozjpeg_sys::boolean = false as mozjpeg_sys::boolean;

const COLOR_SPACE: mozjpeg_sys::J_COLOR_SPACE = mozjpeg_sys::J_COLOR_SPACE::JCS_RGB;
const COLOR_SPACE_COMPONENTS: libc::c_int = 3 as libc::c_int;


///////////////////////////////////////////////////////////////////////////////
// MOZJPEG ENCODER
///////////////////////////////////////////////////////////////////////////////


pub unsafe fn encode(source: &DynamicImage, quality: u8) -> Vec<u8> {
    ///////////////////////////////////////////////////////////////////////////
    // INPUT
    ///////////////////////////////////////////////////////////////////////////
    let rgb_source = source
        .to_rgb()
        .pixels()
        .flat_map(|x| x.0.to_vec())
        .collect::<Vec<_>>();
    let (width, height) = source.dimensions();

    ///////////////////////////////////////////////////////////////////////////
    // INIT ENCODER CONTEXT
    ///////////////////////////////////////////////////////////////////////////
    let mut err = std::mem::zeroed();
    let mut cinfo: mozjpeg_sys::jpeg_compress_struct = std::mem::zeroed();
    let mut outbuffer: *mut libc::c_uchar = std::ptr::null_mut();
    let mut outsize: libc::c_ulong = 0;

    cinfo.common.err = mozjpeg_sys::jpeg_std_error(&mut err);
    mozjpeg_sys::jpeg_create_compress(&mut cinfo);
    mozjpeg_sys::jpeg_mem_dest(&mut cinfo, &mut outbuffer, &mut outsize);

    ///////////////////////////////////////////////////////////////////////////
    // ENCODER CONFIG
    ///////////////////////////////////////////////////////////////////////////
    cinfo.image_width = width;
    cinfo.image_height = height;
    cinfo.input_components = COLOR_SPACE_COMPONENTS;
    let row_stride = cinfo.image_width as usize * cinfo.input_components as usize;
    cinfo.in_color_space = COLOR_SPACE;
    mozjpeg_sys::jpeg_set_defaults(&mut cinfo);
    cinfo.dct_method = mozjpeg_sys::J_DCT_METHOD::JDCT_ISLOW;
    cinfo.write_JFIF_header = FALSE;
    cinfo.optimize_coding = TRUE;
    mozjpeg_sys::jpeg_simple_progression(&mut cinfo);
    mozjpeg_sys::jpeg_c_set_bool_param(&mut cinfo, mozjpeg_sys::JBOOLEAN_USE_SCANS_IN_TRELLIS, TRUE);
    mozjpeg_sys::jpeg_c_set_bool_param(&mut cinfo, mozjpeg_sys::JBOOLEAN_USE_LAMBDA_WEIGHT_TBL, TRUE);
    mozjpeg_sys::jpeg_set_quality(&mut cinfo, quality as i32, TRUE);

    ///////////////////////////////////////////////////////////////////////////
    // GO!
    ///////////////////////////////////////////////////////////////////////////
    mozjpeg_sys::jpeg_start_compress(&mut cinfo, TRUE);
    while cinfo.next_scanline < cinfo.image_height {
        let offset = cinfo.next_scanline as usize * row_stride;
        let jsamparray = [rgb_source[offset..].as_ptr()];
        mozjpeg_sys::jpeg_write_scanlines(&mut cinfo, jsamparray.as_ptr(), 1);
    }
    mozjpeg_sys::jpeg_finish_compress(&mut cinfo);
    mozjpeg_sys::jpeg_destroy_compress(&mut cinfo);

    ///////////////////////////////////////////////////////////////////////////
    // OUTPUT
    ///////////////////////////////////////////////////////////////////////////
    let output_data = std::slice::from_raw_parts(outbuffer, outsize as usize).to_vec();

    ///////////////////////////////////////////////////////////////////////////
    // CLEANUP
    ///////////////////////////////////////////////////////////////////////////
    if !outbuffer.is_null() {
        // FREE MEMORY DEST
        libc::free(outbuffer as *mut mozjpeg_sys::c_void);
        outbuffer = std::ptr::null_mut();
        outsize = 0;
    }

    ///////////////////////////////////////////////////////////////////////////
    // DONE
    ///////////////////////////////////////////////////////////////////////////
    output_data
}

///////////////////////////////////////////////////////////////////////////////
// OPT
///////////////////////////////////////////////////////////////////////////////



///////////////////////////////////////////////////////////////////////////////
// DEV
///////////////////////////////////////////////////////////////////////////////

// pub fn run() {
//     let input_path = "assets/samples/ceiling.jpeg";
//     let source = ::image::open(input_path).expect("source image");
//     let (encoded, report) = OptContext::from_image(source).run_search(false);
//     println!("results: {:#?}", report);
//     std::fs::write("assets/output/test.jpeg", encoded);
// }

