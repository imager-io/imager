// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use itertools::Itertools;
use libc::{c_float, c_void, fread, size_t};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::convert::AsRef;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};
use x264_dev::{raw, sys};

use crate::data::{VideoBuffer, Yuv420P};
use crate::tool::classifier::{self, Class};

///////////////////////////////////////////////////////////////////////////////
// DATA TYPES
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub enum Mode {
    Speed,
    Quality,
}

///////////////////////////////////////////////////////////////////////////////
// MISCELLANEOUS
///////////////////////////////////////////////////////////////////////////////

pub fn pretty_float(x: f64) -> String {
    let x = format!("{}", x);
    let x = x.split(".").collect::<Vec<_>>();
    match x[..] {
        [x, y] => x.to_owned(),
        _ => panic!(),
    }
}

fn c_str(s: &str) -> CString {
    CString::new(s).expect("str to c str")
}

///////////////////////////////////////////////////////////////////////////////
// GLOBAL SETTINGS
///////////////////////////////////////////////////////////////////////////////

pub const SYSTEM_MODE: Mode = Mode::Quality;

///////////////////////////////////////////////////////////////////////////////
// HELPERS
///////////////////////////////////////////////////////////////////////////////

unsafe fn apply(param: &mut sys::X264ParamT, key: &str, value: &str) {
    let key = c_str(key);
    let value = c_str(value);
    assert!(sys::x264_param_parse(param, key.as_ptr(), value.as_ptr()) == 0);
    std::mem::forget(key);
    std::mem::forget(value);
}

unsafe fn new_param(width: u32, height: u32) -> sys::X264ParamT {
    // INIT PARAM
    let mut param: sys::X264ParamT = unsafe { std::mem::zeroed() };
    let profile = CString::new("high").expect("CString failed");
    let tune = {
        let opt1 = "film";
        let opt2 = "animation";
        let opt3 = "grain";
        let opt4 = "ssim";
        CString::new(opt4).expect("CString failed")
    };
    let preset = match SYSTEM_MODE {
        Mode::Quality => {
            let extreme = false;
            if extreme {
                CString::new("placebo").expect("CString failed")
            } else {
                CString::new("medium").expect("CString failed")
            }
        }
        Mode::Speed => CString::new("ultrafast").expect("CString failed"),
    };
    assert!(sys::x264_param_default_preset(&mut param, preset.as_ptr(), tune.as_ptr(),) == 0);
    param.i_bitdepth = 8;
    param.i_csp = raw::X264_CSP_I420 as i32;
    param.i_width = width as i32;
    param.i_height = height as i32;
    param.b_vfr_input = 0;
    param.b_repeat_headers = 1;
    param.b_annexb = 1;

    // DEBUGGING
    param.i_log_level = 1;

    // CPU FLAGS
    apply(&mut param, "non-deterministic", "1");

    // FRAME-TYPE
    apply(&mut param, "partitions", "all");
    apply(&mut param, "constrained-intra", "1");
    apply(&mut param, "deblock", "0,0");

    // RATECONTROL
    // apply(&mut param, "crf", crf);
    apply(&mut param, "qcomp", "0.5");
    apply(&mut param, "aq-mode", "2");
    apply(&mut param, "cplxblur", "20.0");

    // ANALYSIS
    apply(&mut param, "trellis", "2");
    apply(&mut param, "subme", "11");
    apply(&mut param, "psy-rd", "2.0:0.7");
    apply(&mut param, "direct", "none");
    apply(&mut param, "cqm", "flat");
    apply(&mut param, "no-weightb", "1");
    apply(&mut param, "no-mixed-refs", "1");
    apply(&mut param, "no-dct-decimate", "1");

    // FRAME-TYPE
    apply(&mut param, "partitions", "all");
    apply(&mut param, "constrained-intra", "1");
    apply(&mut param, "deblock", "0,0");

    // FINALIZE
    {
        let status = sys::x264_param_apply_profile(&mut param, profile.as_ptr());
        assert!(status == 0);
    };

    // TODO
    std::mem::forget(preset);
    std::mem::forget(tune);
    std::mem::forget(profile);

    // DONE
    param
}

///////////////////////////////////////////////////////////////////////////////
// REPORTING / METADATA
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameReport {
    index: usize,
    vmaf: f64,
    class: Class,
    start_pos: u8,
    crf: u8,
}

///////////////////////////////////////////////////////////////////////////////
// LOW-LEVEL ENCODER
///////////////////////////////////////////////////////////////////////////////

pub unsafe fn encode(stream: &VideoBuffer, crf: f32) -> Result<Vec<u8>, String> {
    ///////////////////////////////////////////////////////////////////////////
    // SETUP
    ///////////////////////////////////////////////////////////////////////////
    let (width, height) = stream.dimensions();
    let luma_size = width * height;
    let chroma_size = luma_size / 4;
    ///////////////////////////////////////////////////////////////////////////
    // INIT PARAM
    ///////////////////////////////////////////////////////////////////////////
    let mut param: sys::X264ParamT = new_param(width, height);
    apply(&mut param, "crf", &format!("{}", crf));
    ///////////////////////////////////////////////////////////////////////////
    // INIT PICTURE
    ///////////////////////////////////////////////////////////////////////////
    let mut picture_param = new_param(width, height);
    let mut picture: sys::X264PictureT = std::mem::zeroed();
    let mut picture_output: sys::X264PictureT = std::mem::zeroed();
    {
        let status =
            sys::x264_picture_alloc(&mut picture, param.i_csp, param.i_width, param.i_height);
        assert!(status == 0);
    };
    ///////////////////////////////////////////////////////////////////////////
    // ENCODER CONTEXT
    ///////////////////////////////////////////////////////////////////////////
    let mut encoder_ctx: *mut sys::X264T = sys::x264_encoder_open(&mut param);
    assert!(!encoder_ctx.is_null());
    assert!(picture.img.i_plane == 3);
    assert!(picture.img.i_stride[0] == width as i32);
    assert!(picture.img.i_stride[1] == (width / 2) as i32);
    assert!(picture.img.i_stride[2] == (width / 2) as i32);
    assert!(picture.param.is_null());
    // picture.param = &mut picture_param;
    ///////////////////////////////////////////////////////////////////////////
    // ???
    ///////////////////////////////////////////////////////////////////////////
    let mut p_nal: *mut sys::X264NalT = std::ptr::null_mut();
    let mut i_nal: i32 = std::mem::zeroed();
    ///////////////////////////////////////////////////////////////////////////
    // ENCODED OUTPUT
    ///////////////////////////////////////////////////////////////////////////
    let mut output = Vec::<u8>::new();
    ///////////////////////////////////////////////////////////////////////////
    // GO!
    ///////////////////////////////////////////////////////////////////////////
    for source in stream.as_frames() {
        let (mut y_ptr, mut u_ptr, mut v_ptr) = unsafe {
            (
                std::slice::from_raw_parts_mut(picture.img.plane[0], luma_size as usize),
                std::slice::from_raw_parts_mut(picture.img.plane[1], chroma_size as usize),
                std::slice::from_raw_parts_mut(picture.img.plane[2], chroma_size as usize),
            )
        };
        y_ptr.copy_from_slice(&source.y());
        u_ptr.copy_from_slice(&source.u());
        v_ptr.copy_from_slice(&source.v());
        // PICTURE SETTINGS
        // apply(&mut picture_param, "crf", &format!("{}", crf));
        // ENCODE
        let i_frame_size = sys::x264_encoder_encode(
            encoder_ctx,
            &mut p_nal,
            &mut i_nal,
            &mut picture,
            &mut picture_output,
        );
        assert!(i_frame_size >= 0);
        if i_frame_size > 0 {
            let encoded = std::slice::from_raw_parts((*p_nal).p_payload, i_frame_size as usize);
            output.extend_from_slice(encoded);
        }
    }
    ///////////////////////////////////////////////////////////////////////////
    // FLUSH DELAYED FRAMES
    ///////////////////////////////////////////////////////////////////////////
    while sys::x264_encoder_delayed_frames(encoder_ctx) > 0 {
        let i_frame_size = sys::x264_encoder_encode(
            encoder_ctx,
            &mut p_nal,
            &mut i_nal,
            std::ptr::null_mut(),
            &mut picture_output,
        );
        assert!(i_frame_size >= 0);
        if i_frame_size > 0 {
            let encoded = std::slice::from_raw_parts((*p_nal).p_payload, i_frame_size as usize);
            output.extend_from_slice(encoded);
        }
    }
    ///////////////////////////////////////////////////////////////////////////
    // CLEANUP
    ///////////////////////////////////////////////////////////////////////////
    sys::x264_encoder_close(encoder_ctx);
    sys::x264_picture_clean(&mut picture);
    ///////////////////////////////////////////////////////////////////////////
    // DONE
    ///////////////////////////////////////////////////////////////////////////
    Ok(output)
}

///////////////////////////////////////////////////////////////////////////////
// DEV - PICTURE OPT
///////////////////////////////////////////////////////////////////////////////

pub unsafe fn opt_frame(index: usize, source: Yuv420P) -> (Vec<u8>, FrameReport) {
    let class_report = classifier::get_report(&source.to_rgba_image());
    let is_4k = (source.width * source.height) >= (3840 * 2160);
    let is_hd = (source.width * source.height) >= (1280 * 720);
    let source_video = VideoBuffer::singleton(source);
    // GO!
    let term = |vmaf_report: f64| -> bool {
        match class_report.class {
            Class::L0 | Class::L1 | Class::L2 => {
                if is_hd {
                    vmaf_report >= 80.0
                } else {
                    vmaf_report >= 96.0
                }
            }
            Class::M1 => {
                if is_hd {
                    vmaf_report >= 60.0
                } else {
                    vmaf_report >= 70.0
                }
            }
            Class::H1 | Class::H2 => {
                if is_hd {
                    vmaf_report >= 30.0
                } else {
                    vmaf_report >= 60.0
                }
            }
        }
    };
    let starting_pos = || {
        let run = |q| -> bool {
            let encoded = encode(&source_video, q as f32).expect("encode yuv420p");
            let ref_video = VideoBuffer::load_from_memory(&encoded).expect("reconstruct");
            let vmaf_report = crate::tool::vmaf::get_report(&source_video, &ref_video);
            term(vmaf_report)
        };
        let reduce_starting_values = |qs: Vec<u8>| -> Option<u8> {
            let mut result: Option<u8> = None;
            for q in qs.into_iter() {
                let passed = run(q);
                if !passed {
                    assert!(result.is_none());
                    result = Some(q);
                    break;
                }
            }
            result
        };
        let bad_fallback = || reduce_starting_values(vec![10, 30, 50, 60]);
        match class_report.class {
            // TODO
            // ...
            _ => bad_fallback(),
        }
    };
    let start_pos = starting_pos().unwrap_or(0);
    for crf in (0..start_pos).rev().filter(|x| x % 2 == 0) {
        let encoded = encode(&source_video, crf as f32).expect("encode yuv420p");
        let ref_video = VideoBuffer::load_from_memory(&encoded).expect("reconstruct");
        let vmaf_report = crate::tool::vmaf::get_report(&source_video, &ref_video);
        if term(vmaf_report) {
            // return (crf, vmaf_report, class_report.class.clone(), encoded);
            let meta = FrameReport {
                index,
                vmaf: vmaf_report,
                class: class_report.class.clone(),
                start_pos: start_pos,
                crf: crf,
            };
            return (encoded, meta);
        }
    }
    // FALLBACK
    unimplemented!()
}

pub fn opt_frames(stream: &VideoBuffer) -> BTreeMap<usize, FrameReport> {
    let frames_meta = stream
        .as_frames()
        .clone()
        .into_iter()
        .enumerate()
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|(index, source)| {
            let (frame, frame_meta) = unsafe { opt_frame(index, source.clone()) };
            let path = format!(
                "assets/output/debug/ix={ix}--q={qp}--vmaf={vmaf}--cls={cls}.h264",
                ix = frame_meta.index,
                qp = frame_meta.crf,
                cls = frame_meta.class,
                vmaf = pretty_float(frame_meta.vmaf),
            );
            std::fs::write(path, frame);
            println!("opt frame: {}", index);
            (index, frame_meta)
        })
        .collect::<BTreeMap<usize, FrameReport>>();
    {
        let json = serde_json::to_string_pretty(&frames_meta).expect("to json str");
        std::fs::write("assets/output/test.json", json);
    }
    frames_meta
}

pub unsafe fn opt_video(stream: &VideoBuffer) -> Result<Vec<u8>, ()> {
    ///////////////////////////////////////////////////////////////////////////
    // FRAME REPORT
    ///////////////////////////////////////////////////////////////////////////
    println!("total frames: {}", stream.as_frames().len());
    let frames_report = opt_frames(stream);
    ///////////////////////////////////////////////////////////////////////////
    // SETUP
    ///////////////////////////////////////////////////////////////////////////
    let (width, height) = stream.dimensions();
    let luma_size = width * height;
    let chroma_size = luma_size / 4;
    ///////////////////////////////////////////////////////////////////////////
    // INIT PARAM
    ///////////////////////////////////////////////////////////////////////////
    let mut param: sys::X264ParamT = new_param(width, height);
    // apply(&mut param, "crf", &format!("{}", crf));
    ///////////////////////////////////////////////////////////////////////////
    // INIT PICTURE
    ///////////////////////////////////////////////////////////////////////////
    let mut picture_param = new_param(width, height);
    let mut picture: sys::X264PictureT = std::mem::zeroed();
    let mut picture_output: sys::X264PictureT = std::mem::zeroed();
    {
        let status =
            sys::x264_picture_alloc(&mut picture, param.i_csp, param.i_width, param.i_height);
        assert!(status == 0);
    };
    ///////////////////////////////////////////////////////////////////////////
    // ENCODER CONTEXT
    ///////////////////////////////////////////////////////////////////////////
    let mut encoder_ctx: *mut sys::X264T = sys::x264_encoder_open(&mut param);
    assert!(!encoder_ctx.is_null());
    assert!(picture.img.i_plane == 3);
    assert!(picture.img.i_stride[0] == width as i32);
    assert!(picture.img.i_stride[1] == (width / 2) as i32);
    assert!(picture.img.i_stride[2] == (width / 2) as i32);
    assert!(picture.param.is_null());
    picture.param = &mut picture_param;
    ///////////////////////////////////////////////////////////////////////////
    // ???
    ///////////////////////////////////////////////////////////////////////////
    let mut p_nal: *mut sys::X264NalT = std::ptr::null_mut();
    let mut i_nal: i32 = std::mem::zeroed();
    ///////////////////////////////////////////////////////////////////////////
    // ENCODED OUTPUT
    ///////////////////////////////////////////////////////////////////////////
    let mut output = Vec::<u8>::new();
    ///////////////////////////////////////////////////////////////////////////
    // GO!
    ///////////////////////////////////////////////////////////////////////////
    let mut frames_meta = Vec::<FrameReport>::new();
    for (index, source) in stream.as_frames().iter().enumerate() {
        // BUFFER
        let (mut y_ptr, mut u_ptr, mut v_ptr) = unsafe {
            (
                std::slice::from_raw_parts_mut(picture.img.plane[0], luma_size as usize),
                std::slice::from_raw_parts_mut(picture.img.plane[1], chroma_size as usize),
                std::slice::from_raw_parts_mut(picture.img.plane[2], chroma_size as usize),
            )
        };
        y_ptr.copy_from_slice(&source.y());
        u_ptr.copy_from_slice(&source.u());
        v_ptr.copy_from_slice(&source.v());
        // PICTURE SETTINGS
        let crf = {
            // let (frame, frame_meta) = opt_frame(index, source.clone());
            // frames_meta.push(frame_meta.clone());
            // let path = format!(
            //     "assets/output/debug/ix={ix}--q={qp}--vmaf={vmaf}--cls={cls}.h264",
            //     ix=frame_meta.index,
            //     qp=frame_meta.crf,
            //     cls=frame_meta.class,
            //     vmaf=pretty_float(frame_meta.vmaf),
            // );
            // std::fs::write(path, frame);
            // frame_meta.crf
            frames_report.get(&index).expect("missing frame report").crf
        };
        apply(&mut picture_param, "crf", &format!("{}", crf));
        // ENCODE
        let i_frame_size = sys::x264_encoder_encode(
            encoder_ctx,
            &mut p_nal,
            &mut i_nal,
            &mut picture,
            &mut picture_output,
        );
        assert!(i_frame_size >= 0);
        if i_frame_size > 0 {
            let encoded = std::slice::from_raw_parts((*p_nal).p_payload, i_frame_size as usize);
            output.extend_from_slice(encoded);
        }
        // MISC
        println!("frame: {}", index);
    }
    ///////////////////////////////////////////////////////////////////////////
    // DEBUG LOG
    ///////////////////////////////////////////////////////////////////////////
    // {
    //     let json = serde_json::to_string_pretty(&frames_meta).expect("to json str");
    //     std::fs::write("assets/output/test.json", json);
    // }
    ///////////////////////////////////////////////////////////////////////////
    // FLUSH DELAYED FRAMES
    ///////////////////////////////////////////////////////////////////////////
    while sys::x264_encoder_delayed_frames(encoder_ctx) > 0 {
        let i_frame_size = sys::x264_encoder_encode(
            encoder_ctx,
            &mut p_nal,
            &mut i_nal,
            std::ptr::null_mut(),
            &mut picture_output,
        );
        assert!(i_frame_size >= 0);
        if i_frame_size > 0 {
            let encoded = std::slice::from_raw_parts((*p_nal).p_payload, i_frame_size as usize);
            output.extend_from_slice(encoded);
        }
    }
    ///////////////////////////////////////////////////////////////////////////
    // CLEANUP
    ///////////////////////////////////////////////////////////////////////////
    sys::x264_encoder_close(encoder_ctx);
    sys::x264_picture_clean(&mut picture);
    ///////////////////////////////////////////////////////////////////////////
    // DONE
    ///////////////////////////////////////////////////////////////////////////
    Ok(output)
}

///////////////////////////////////////////////////////////////////////////////
// DEV
///////////////////////////////////////////////////////////////////////////////

pub fn run() {
    let source = VideoBuffer::open_video("assets/samples/dump2.h264").expect("decode video file");
    let output = unsafe { opt_video(&source).expect("opt encode faild") };
    std::fs::write("assets/output/test.h264", &output);
}

// pub fn run() {
//     let source = Yuv420P::open_image("assets/samples/ceiling.jpeg").expect("source image");
//     let (_, result) = unsafe {opt_frame(source)};
//     std::fs::write("assets/output/test.h264", &result);
// }
