// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::convert::TryFrom;
use std::sync::{Mutex, MutexGuard};
use std::rc::Rc;
use std::cell::RefCell;
use std::io::prelude::*;
use std::path::PathBuf;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::os::raw::c_int;
use libc::{size_t, c_float};
use lazy_static::lazy_static;
use crate::tool::mozjpeg::DecodedImage;


#[repr(C)]
struct VmafReportContext {
    source1: DecodedImage,
    source2: DecodedImage,
    frame_set: bool,
}

unsafe extern "C" fn read_frame(
    mut source1_out: *mut c_float,
    mut source2_out: *mut c_float,
    temp_data: *mut c_float,
    out_stride: c_int,
    raw_ctx: *mut libc::c_void,
) -> c_int {
    // CONTEXT
    let mut vmaf_ctx = Box::from_raw(raw_ctx as *mut VmafReportContext);
    let mut vmaf_ctx = Box::leak(vmaf_ctx);

    // RESOLUTION
    let width = vmaf_ctx.source1.width;
    let height = vmaf_ctx.source1.height;

    // Y PLANE DATA
    let mut source1_in: *mut u8 = vmaf_ctx.source1.buffer.as_mut_ptr();
    let mut source2_in: *mut u8 = vmaf_ctx.source2.buffer.as_mut_ptr();

    // Y PLANE LINESIZE
    let source1_linesize: usize = vmaf_ctx.source1.linesize as usize;
    let source2_linesize: usize = vmaf_ctx.source2.linesize as usize;
    
    // OUTPUT LINESIZE
    let out_stride = out_stride as usize;

    // DONE
    if vmaf_ctx.frame_set {
        return 2;
    }
    
    // FILL BUFFERS (THE EXTREMELY UNSAFE, DANGEROUS AND CONFUSING PART)
    for y in 0..height {
        for x in 0..width {
            // GET - SOURCE 1 & 2
            let s1_px: u8 = *(source1_in.offset(x as isize));
            let s2_px: u8 = *(source2_in.offset(x as isize));

            // CONVERT - SOURCE 1 & 2
            let s1_px: c_float = s1_px as c_float;
            let s2_px: c_float = s2_px as c_float;

            // SET - OUTPUT 1 & 2
            *(source1_out.offset(x as isize)) = s1_px;
            *(source2_out.offset(x as isize)) = s2_px;
        }
        // UPDATE - SOURCE 1
        source1_in = source1_in.add(source1_linesize / std::mem::size_of_val(&*source1_in));
        source1_out = source1_out.add(out_stride / std::mem::size_of_val(&*source1_out));

        // UPDATE - SOURCE 2
        source2_in = source2_in.add(source2_linesize / std::mem::size_of_val(&*source2_in));
        source2_out = source2_out.add(out_stride / std::mem::size_of_val(&*source2_out));
    }
    vmaf_ctx.frame_set = true;
    return 0;
}


unsafe fn vmaf_controller(source1: DecodedImage, source2: DecodedImage) -> f64 {
    // RESOLUTION REQUIREMENTS
    assert!(source1.width == source2.width);
    assert!(source1.height == source2.height);

    // INIT VMAF CONTEXT
    let mut vmaf_ctx = Box::new(VmafReportContext {
        source1: source1.clone(),
        source2: source2.clone(),
        frame_set: false
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
    let width = source1.width;
    let height = source1.height;
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
    let compute_vmaf_res = vmaf_sys::compute_vmaf(
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
    assert!(compute_vmaf_res == 0);

    // CLEANUP
    let mut vmaf_ctx = Box::from_raw(vmaf_ctx);
    std::mem::drop(vmaf_ctx);

    // DONE
    vmaf_score
}


lazy_static! {
    static ref VMAF_LOCK: Mutex<()> = {
        Mutex::new(())
    };
}

pub fn report(source1: &DecodedImage, source2: &DecodedImage) -> f64 {
    // LOCK
    let lock = VMAF_LOCK.lock().expect("failed to lock vmaf work");
    // GO!
    let score = unsafe {
        vmaf_controller(source1.clone(), source2.clone())
    };
    // UNLOCK
    std::mem::drop(lock);
    // DONE
    score
}
