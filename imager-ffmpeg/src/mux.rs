// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::convert::AsRef;
use std::path::{PathBuf, Path};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float, c_void};
use ffmpeg_dev::sys::{
    self,
    AVMediaType_AVMEDIA_TYPE_UNKNOWN as AVMEDIA_TYPE_UNKNOWN,
    AVMediaType_AVMEDIA_TYPE_VIDEO as AVMEDIA_TYPE_VIDEO,
    AVMediaType_AVMEDIA_TYPE_AUDIO as AVMEDIA_TYPE_AUDIO,
    AVMediaType_AVMEDIA_TYPE_DATA as AVMEDIA_TYPE_DATA,
    AVMediaType_AVMEDIA_TYPE_SUBTITLE as AVMEDIA_TYPE_SUBTITLE,
    AVMediaType_AVMEDIA_TYPE_ATTACHMENT as AVMEDIA_TYPE_ATTACHMENT,
    AVMediaType_AVMEDIA_TYPE_NB as AVMEDIA_TYPE_NB,
    AVFMT_NOFILE,
    AVIO_FLAG_WRITE,
    AVRounding_AV_ROUND_NEAR_INF as AV_ROUND_NEAR_INF,
    AVRounding_AV_ROUND_PASS_MINMAX as AV_ROUND_PASS_MINMAX,
    AVCodecID_AV_CODEC_ID_H264 as AV_CODEC_ID_H264,
};


fn c_str(s: &str) -> CString {
    CString::new(s).expect("str to c str")
}

unsafe fn remux(
    input_path: &str,
    output_path: &str,
) {
    // I/O
    assert!(PathBuf::from(input_path).exists());
    let input_path_cstr = CString::new(input_path).expect("to c str");
    let output_path_cstr = CString::new(output_path).expect("to c str");
    // SETUP AV CONTEXT
    let mut ifmt_ctx: *mut sys::AVFormatContext = std::ptr::null_mut();
    let mut ofmt_ctx: *mut sys::AVFormatContext = std::ptr::null_mut();
    let mut pkt: sys::AVPacket = std::mem::zeroed();
    // OPEN SOURCE
    assert_eq!(
        sys::avformat_open_input(
            &mut ifmt_ctx,
            input_path_cstr.as_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ),
        0
    );
    assert!(sys::avformat_find_stream_info(ifmt_ctx, std::ptr::null_mut()) >= 0);
    sys::av_dump_format(
        ifmt_ctx,
        0,
        input_path_cstr.as_ptr(),
        0,
    );
    // OUTPUT CONTEXT
    assert!(sys::avformat_alloc_output_context2(
        &mut ofmt_ctx,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        input_path_cstr.as_ptr(),
    ) >= 0);
    // OUTPUT META
    let mut ofmt: *mut sys::AVOutputFormat = (*ofmt_ctx).oformat;

    // STREAM TRACKER
    let mut stream_mapping_size: u32 = (*ifmt_ctx).nb_streams;
    let mut stream_mapping: Vec<i32> = vec![0; stream_mapping_size as usize];

    // SOURCE TO DEST STREAMS
    let input_streams = {
        let len = (*ifmt_ctx).nb_streams as usize;
        std::slice::from_raw_parts((*ifmt_ctx).streams, len)
            .iter()
            .map(|x| (*x).as_ref().expect("not null"))
            .collect::<Vec<&sys::AVStream>>()
    };
    for (index, in_stream) in input_streams.iter().enumerate() {
        assert!(!in_stream.codecpar.is_null());
        let mut out_stream: *mut sys::AVStream = std::ptr::null_mut();
        let skip = {
            (*in_stream.codecpar).codec_type != AVMEDIA_TYPE_VIDEO
        };
        if skip {
            stream_mapping[index] = -1;
        } else {
            out_stream = sys::avformat_new_stream(ofmt_ctx, std::ptr::null());
            assert!(!out_stream.is_null());
            let status = sys::avcodec_parameters_copy(
                (*out_stream).codecpar,
                in_stream.codecpar,
            );
            assert!(status >= 0);
            (*(*out_stream).codecpar).codec_tag = 0;
        }
    }

    sys::av_dump_format(ofmt_ctx, 0, output_path_cstr.as_ptr(), 1);

    // OPEN OUTPUT STREAM
    if ((*ofmt).flags & (AVFMT_NOFILE as i32)) == 0 {
        let status = sys::avio_open(
            &mut (*ofmt_ctx).pb,
            output_path_cstr.as_ptr(),
            AVIO_FLAG_WRITE as i32,
        );
        assert!(status >= 0);
    }
    // WITE OUTPUT
    assert!(sys::avformat_write_header(ofmt_ctx, std::ptr::null_mut()) >= 0);
    let mut status = 0;
    loop {
        if sys::av_read_frame(ifmt_ctx, &mut pkt) != 0 {
            break;
        }
        // SOURCE
        let in_stream: *mut sys::AVStream = (*(*ifmt_ctx).streams).offset(pkt.stream_index as isize);
        assert!(!in_stream.is_null());
        // DEST
        let mut out_stream: *mut sys::AVStream = std::ptr::null_mut();
        // ???
        let skip = {
            pkt.stream_index >= stream_mapping.len() as i32 ||
            stream_mapping[pkt.stream_index as usize] < 0
        };
        if skip {
            sys::av_packet_unref(&mut pkt);
            continue;
        }
        pkt.stream_index = stream_mapping[pkt.stream_index as usize];
        out_stream = (*(*ofmt_ctx).streams).offset(pkt.stream_index as isize);
        // COPY PACKET
        pkt.pts = sys::av_rescale_q_rnd(
            pkt.pts,
            (*in_stream).time_base,
            (*out_stream).time_base,
            AV_ROUND_NEAR_INF|AV_ROUND_PASS_MINMAX,
        );
        pkt.dts = sys::av_rescale_q_rnd(
            pkt.dts,
            (*in_stream).time_base,
            (*out_stream).time_base,
            AV_ROUND_NEAR_INF|AV_ROUND_PASS_MINMAX
        );
        pkt.duration = sys::av_rescale_q(
            pkt.duration,
            (*in_stream).time_base,
            (*out_stream).time_base,
        );
        pkt.pos = -1;
        // WRITE
        
        // RESCALE OUTPUT PACKET TIMESTAMP VALUES FROM CODEC TO STREAM TIMEBASE
        // av_packet_rescale_ts(pkt, *time_base, st->time_base);
        // pkt->stream_index = st->index;
        
        // WRITE THE COMPRESSED FRAME TO THE MEDIA FILE
        assert!(sys::av_interleaved_write_frame(ofmt_ctx, &mut pkt) >= 0);
        // assert!(av_write_frame(ofmt_ctx, &mut pkt) >= 0);
        sys::av_packet_unref(&mut pkt);
    }

    sys::av_write_trailer(ofmt_ctx);

    (*ifmt_ctx);

    // CLOSE OUTPUT
    if !ofmt_ctx.is_null() && ((*ofmt).flags & (AVFMT_NOFILE as i32) <= 0) {
        sys::avio_closep(&mut (*ofmt_ctx).pb);
    }
    sys::avformat_free_context(ofmt_ctx); 
    assert!(status != sys::EOF);
}

pub fn run() {
    // let input_path = "assets/samples/test.h264";
    let input_path = "assets/samples/sintel_trailer.1080p.mp4";
    let output_path = "assets/output/test.mp4";
    unsafe {
        remux(input_path, output_path);
    };
}
