// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float, c_void};
use lazy_static::lazy_static;

use crate::data::{Yuv420P, VideoBuffer};


///////////////////////////////////////////////////////////////////////////////
// VMAF CONTEXT
///////////////////////////////////////////////////////////////////////////////

struct Context<'a> {
    stream1: &'a mut VideoBuffer,
    stream2: &'a mut VideoBuffer,
    frames_set: bool,
}

lazy_static! {
    static ref VMAF_LOCK: Mutex<()> = {
        Mutex::new(())
    };
}

///////////////////////////////////////////////////////////////////////////////
// VMAF CALLBACK
///////////////////////////////////////////////////////////////////////////////


unsafe fn fill_vmaf_buffer(
    mut output: *mut c_float,
    output_stride: c_int,
    source: &Yuv420P,
) {
    let (width, height) = source.dimensions();
    let src_linesize = source.width as usize;
    let dest_stride = output_stride as usize;
    let mut source_ptr: *const u8 = source.y().as_ptr();
    for y in 0..height {
        for x in 0..width {
            let s1_px: u8 = *(source_ptr.offset(x as isize));
            let s1_px: c_float = s1_px as c_float;
            *(output.offset(x as isize)) = s1_px
        }
        source_ptr = source_ptr.add(src_linesize / std::mem::size_of_val(&*source_ptr));
        output = output.add(dest_stride / std::mem::size_of_val(&*output));
    }
}

unsafe extern "C" fn read_frame(
    mut source1_out: *mut c_float,
    mut source2_out: *mut c_float,
    temp_data: *mut c_float,
    out_stride: c_int,
    raw_ctx: *mut libc::c_void,
) -> c_int {
    // CONTEXT
    let mut vmaf_ctx = Box::from_raw(raw_ctx as *mut Context);
    let mut vmaf_ctx = Box::leak(vmaf_ctx);

    // DONE
    if vmaf_ctx.frames_set {
        return 2;
    }

    // NEXT FRAME OR DONE
    match (vmaf_ctx.stream1.next(), vmaf_ctx.stream2.next()) {
        (Some(frame1), Some(frame2)) => {
            fill_vmaf_buffer(source1_out, out_stride, &frame1);
            fill_vmaf_buffer(source2_out, out_stride, &frame2);
        }
        (None, None) => {
            vmaf_ctx.frames_set = true;
        }
        _ => panic!()
    }
    if vmaf_ctx.frames_set {
        2
    } else {
        0
    }
}


///////////////////////////////////////////////////////////////////////////////
// VMAF PIPELINE
///////////////////////////////////////////////////////////////////////////////

pub unsafe fn vmaf_controller<'a>(stream1: &'a mut VideoBuffer, stream2: &'a mut VideoBuffer) -> f64 {
    // CHECKS
    assert!(stream1.dimensions() == stream2.dimensions());

    // INIT VMAF CONTEXT
    let (width, height) = stream1.dimensions();
    let vmaf_ctx = Box::new(Context {
        stream1: stream1,
        stream2: stream2,
        frames_set: false
    });
    let vmaf_ctx = Box::into_raw(vmaf_ctx);

    // SETTINGS
    let mut vmaf_score = 0.0;
    let model_path = vmaf_sys::extras::get_4k_model_path()
        .to_str()
        .expect("PathBuf to str failed")
        .to_owned();
    let model_path = CString::new(model_path).expect("CString::new failed");
    let mut fmt = CString::new(String::from("yuv420p")).expect("CString::new failed");
    let log_path: *mut c_char = std::ptr::null_mut();
    let log_fmt: *mut c_char = std::ptr::null_mut();
    let disable_clip = 0;
    let disable_avx = 0;
    let enable_transform = 0;
    let phone_model = 0;
    let do_psnr = 0;
    let do_ssim = 0;
    let do_ms_ssim = 0;
    let pool_method: *mut c_char = std::ptr::null_mut();
    let n_thread = 1;
    let n_subsample = 1;
    let enable_conf_interval = 0;

    // GO!
    let status = vmaf_sys::compute_vmaf(
        &mut vmaf_score,
        fmt.as_ptr() as *mut c_char,
        width as c_int,
        height as c_int,
        Some(read_frame),
        vmaf_ctx as *mut libc::c_void,
        model_path.as_ptr() as *mut c_char,
        log_path,
        log_fmt,
        disable_clip,
        disable_avx,
        enable_transform,
        phone_model,
        do_psnr,
        do_ssim,
        do_ms_ssim,
        pool_method,
        n_thread,
        n_subsample,
        enable_conf_interval
    );

    // CHECK
    assert!(status == 0);

    // CLEANUP
    let mut vmaf_ctx = Box::from_raw(vmaf_ctx);
    std::mem::drop(vmaf_ctx);

    // DONE
    vmaf_score
}

pub fn get_report(stream1: &VideoBuffer, stream2: &VideoBuffer) -> f64 {
    // SETUP
    let mut stream1 = stream1.as_fresh_cursor();
    let mut stream2 = stream2.as_fresh_cursor();
    assert!(stream1.as_frames().len() == stream2.as_frames().len());
    // LOCK
    let lock = VMAF_LOCK.lock().expect("failed to lock vmaf work");
    // GO!
    let score = unsafe {vmaf_controller(&mut stream1, &mut stream2)};
    // UNLOCK
    std::mem::drop(lock);
    // DONE
    score
}