// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::convert::AsRef;
use std::path::{PathBuf, Path};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::collections::VecDeque;
use libc::{size_t, c_float, c_void, fread};
use x264_dev::{raw, sys};
use itertools::Itertools;
use crate::yuv420p::Yuv420P;
use crate::stream::{Stream, FileStream, SingleImage};


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


///////////////////////////////////////////////////////////////////////////////
// GLOBAL SETTINGS
///////////////////////////////////////////////////////////////////////////////

pub const SYSTEM_MODE: Mode = Mode::Speed;

///////////////////////////////////////////////////////////////////////////////
// HELPERS
///////////////////////////////////////////////////////////////////////////////

unsafe fn new_param(width: u32, height: u32) -> sys::X264ParamT {
    let mut param: sys::X264ParamT = unsafe {std::mem::zeroed()};
    let preset = CString::new("ultrafast").expect("CString failed");
    let tune = CString::new("ssim").expect("CString failed");
    let profile = CString::new("high").expect("CString failed");
    let preset = match SYSTEM_MODE {
        Mode::Quality => CString::new("placebo").expect("CString failed"),
        Mode::Speed => CString::new("ultrafast").expect("CString failed"),
    };
    
    {
        let status = sys::x264_param_default_preset(
            &mut param,
            preset.as_ptr(),
            tune.as_ptr(),
        );
        assert!(status == 0);
    };

    param.rc.i_rc_method = 1;
    param.i_bitdepth = 8;
    param.i_csp = raw::X264_CSP_I420 as i32;
    param.i_width  = width as i32;
    param.i_height = height as i32;
    param.b_vfr_input = 0;
    param.b_repeat_headers = 1;
    param.b_annexb = 1;

    {
        let status = sys::x264_param_apply_profile(&mut param, profile.as_ptr());
        assert!(status == 0);
    };

    param
}




///////////////////////////////////////////////////////////////////////////////
// LOW-LEVEL ENCODER
///////////////////////////////////////////////////////////////////////////////

pub unsafe fn encode(mut stream: impl Stream) -> Result<Vec<u8>, String> {
    ///////////////////////////////////////////////////////////////////////////
    // SETUP
    ///////////////////////////////////////////////////////////////////////////
    let (width, height) = stream.dimensions();
    let luma_size = width * height;
    let chroma_size = luma_size / 4;
    ///////////////////////////////////////////////////////////////////////////
    // INIT PARAM
    ///////////////////////////////////////////////////////////////////////////
    let mut param = new_param(width, height);
    param.rc.i_qp_constant = 10;
    ///////////////////////////////////////////////////////////////////////////
    // INIT PICTURE
    ///////////////////////////////////////////////////////////////////////////
    let mut picture_param = new_param(width, height);
    let mut picture: sys::X264PictureT = std::mem::zeroed();
    let mut picture_output: sys::X264PictureT = std::mem::zeroed();
    {
        let status = sys::x264_picture_alloc(
            &mut picture,
            param.i_csp,
            param.i_width,
            param.i_height
        );
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
    // ???
    let mut p_nal: *mut sys::X264NalT = std::ptr::null_mut();
    let mut i_nal: i32 = std::mem::zeroed();
    ///////////////////////////////////////////////////////////////////////////
    // ENCODED OUTPUT
    ///////////////////////////////////////////////////////////////////////////
    let mut output = Vec::<u8>::new();
    ///////////////////////////////////////////////////////////////////////////
    // GO!
    ///////////////////////////////////////////////////////////////////////////
    let mut frame_index = 0;
    while let Some(source) = stream.next() {
        frame_index = frame_index + 1;
        let (mut y_ptr, mut u_ptr, mut v_ptr) = unsafe {(
            std::slice::from_raw_parts_mut(picture.img.plane[0], luma_size as usize),
            std::slice::from_raw_parts_mut(picture.img.plane[1], chroma_size as usize),
            std::slice::from_raw_parts_mut(picture.img.plane[2], chroma_size as usize),
        )};
        y_ptr.copy_from_slice(&source.y);
        u_ptr.copy_from_slice(&source.u);
        v_ptr.copy_from_slice(&source.v);
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
            let encoded = std::slice::from_raw_parts(
                (*p_nal).p_payload,
                i_frame_size as usize,
            );
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
            let encoded = std::slice::from_raw_parts(
                (*p_nal).p_payload,
                i_frame_size as usize,
            );
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
    let source_path = "assets/samples/sintel-trailer-gop1";
    // let source_path = "assets/samples/gop-test-single";
    let mut stream = FileStream::new(source_path, 1920, 818);
    let output = unsafe {encode(stream).expect("encode faild")};
    std::fs::write("assets/output/test.h264", &output);
}