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

use crate::data::{VideoBuffer, Yuv420P};
use crate::classifier::{self, Class};
use crate::vmaf;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptReport {
    pub start_q: u8,
    pub end_q: u8,
    pub passed: bool,
    pub class: Class,
    pub vmaf_score: Option<f64>,
}

pub struct OptContext {
    source: DynamicImage,
    vmaf_source: VideoBuffer,
    class_report: classifier::Report,
    extreme_mode: bool,
}

impl OptContext {
    pub fn from_image(source: DynamicImage) -> Self {
        OptContext {
            vmaf_source: VideoBuffer::from_image(&source).expect("to VideoBuffer"),
            class_report: classifier::report(&source),
            source: source,
            extreme_mode: false,
        }
    }
    fn terminate(&self, score: f64) -> bool {
        let mut threshold;
        let (width, height) = self.source.dimensions();
        let is_small = {
            (width * height) <= (500 * 500)
        };
        let is_big = {
            (width * height) >= (1200 * 1000)
        };
        match self.class_report.class {
            Class::L0 if self.class_report.white_backdrop => {
                threshold = 96.0;
            }
            Class::L1 if self.class_report.white_backdrop => {
                threshold = 94.0;
            }
            Class::L2 if self.class_report.white_backdrop => {
                threshold = 93.0;
            }
            Class::L0 | Class::L1 | Class::L2 if self.extreme_mode && is_big => {
                threshold = 95.0;
            }
            Class::L0 => {
                threshold = 99.0;
            }
            Class::L1 => {
                threshold = 98.0;
            }
            Class::L2 => {
                threshold = 96.0;
            }
            Class::M1 => {
                threshold = 92.0;
            }
            Class::H1 if !is_small => {
                threshold = 84.0;
            }
            Class::H2 if !is_small => {
                threshold = 76.0;
            }
            Class::H1 | Class::H2 => {
                assert!(is_small);
                threshold = 88.0;
            }
        }
        if (score >= threshold) {
            true
        } else {
            false
        }
    }
    fn find_starting_position(&self) -> Option<u8> {
        let reduce_starting_values = |qs: Vec<u8>| -> Option<u8> {
            let xs = qs
                .into_par_iter()
                .map(|q| -> (u8, bool) {
                    (q, self.run_instance(q).1)
                })
                .collect::<Vec<_>>()
                .into_iter()
                .sorted_by(|a, b| u8::cmp(&a.0, &b.0))
                .collect::<Vec<_>>();
            let mut result: Option<u8> = None;
            for (q, passed) in xs.into_iter().rev() {
                if !passed {
                    result = Some(q);
                    break;
                }
            }
            result
        };
        let bad_fallback = || reduce_starting_values(vec![
            90,
            80,
            70,
            60,
            50,
            40,
        ]);
        match self.class_report.class {
            Class::H2 => {
                reduce_starting_values(vec![
                    30,
                    20,
                    10,
                ])
            }
            Class::H1 => {
                reduce_starting_values(vec![
                    60,
                    50,
                    40,
                    30,
                    20,
                ])
            }
            Class::M1 => {
                reduce_starting_values(vec![
                    80,
                    70,
                    60,
                    50,
                    40,
                    30,
                ])
            }
            _ => bad_fallback()
        }
    }
    fn run_instance(&self, q: u8) -> (Vec<u8>, bool, f64) {
        let compressed = unsafe {
            encode(&self.source, q)
        };
        // TODO - CLEANUP
        let report: f64 = {
            let vmaf_derivative = VideoBuffer::from_jpeg(&compressed).expect("load jpeg image");
            vmaf::get_report(&self.vmaf_source, &vmaf_derivative)
        };
        if self.terminate(report) {
            (compressed, true, report)
        } else {
            (compressed, false, report)
        }
    }
    pub fn run_search(&mut self, extreme_mode: bool) -> (Vec<u8>, OptReport) {
        self.extreme_mode = extreme_mode;
        let mut passed_output: Option<(Vec<u8>, OptReport)> = None;
        let starting_q = self.find_starting_position().unwrap_or(0);
        for q in starting_q..=98 {
            let (compressed, done, score) = self.run_instance(q);
            if done {
                let out_meta = OptReport {
                    start_q: starting_q,
                    end_q: q,
                    passed: true,
                    class: self.class_report.class.clone(),
                    vmaf_score: Some(score),
                };
                passed_output = Some((compressed, out_meta));
                break;
            }
        }
        match passed_output {
            // BAD
            None => {
                let fallback_q = 98;
                let payload = unsafe {
                    encode(&self.source, fallback_q)
                };
                let out_meta = OptReport {
                    start_q: starting_q,
                    end_q: fallback_q,
                    passed: false,
                    class: self.class_report.class.clone(),
                    vmaf_score: None,
                };
                (payload, out_meta)
            }
            // MAYBE
            Some((payload, meta)) => {
                // BAD
                if meta.start_q == 0 && meta.end_q == 0 {
                    let fallback_q = 75;
                    let payload = unsafe {
                        encode(&self.source, fallback_q)
                    };
                    let out_meta = OptReport {
                        start_q: starting_q,
                        end_q: fallback_q,
                        passed: false,
                        class: self.class_report.class.clone(),
                        vmaf_score: None,
                    };
                    (payload, out_meta)
                }
                // GOOD
                else {
                    (payload, meta)
                }
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DEV
///////////////////////////////////////////////////////////////////////////////

pub fn run() {
    let input_path = "assets/samples/ceiling.jpeg";
    let source = ::image::open(input_path).expect("source image");
    let (encoded, report) = OptContext::from_image(source).run_search(false);
    println!("results: {:#?}", report);
    std::fs::write("assets/output/test.jpeg", encoded);
}

