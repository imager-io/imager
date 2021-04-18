// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use crate::data::{VideoBuffer, Yuv420P};
use ffmpeg_dev::extra::defs;
use ffmpeg_dev::sys::{
    self, AVCodec, AVCodecContext, AVCodecID_AV_CODEC_ID_H264 as AV_CODEC_ID_H264,
    AVCodecParameters, AVCodecParserContext, AVDictionary, AVFormatContext, AVFrame, AVMediaType,
    AVMediaType_AVMEDIA_TYPE_ATTACHMENT as AVMEDIA_TYPE_ATTACHMENT,
    AVMediaType_AVMEDIA_TYPE_AUDIO as AVMEDIA_TYPE_AUDIO,
    AVMediaType_AVMEDIA_TYPE_DATA as AVMEDIA_TYPE_DATA,
    AVMediaType_AVMEDIA_TYPE_NB as AVMEDIA_TYPE_NB,
    AVMediaType_AVMEDIA_TYPE_SUBTITLE as AVMEDIA_TYPE_SUBTITLE,
    AVMediaType_AVMEDIA_TYPE_UNKNOWN as AVMEDIA_TYPE_UNKNOWN,
    AVMediaType_AVMEDIA_TYPE_VIDEO as AVMEDIA_TYPE_VIDEO, AVOutputFormat, AVPacket,
    AVPixelFormat_AV_PIX_FMT_YUV420P as AV_PIX_FMT_YUV420P,
    AVRounding_AV_ROUND_NEAR_INF as AV_ROUND_NEAR_INF,
    AVRounding_AV_ROUND_PASS_MINMAX as AV_ROUND_PASS_MINMAX, AVStream, AVFMT_NOFILE,
    AVIO_FLAG_WRITE, AV_INPUT_BUFFER_PADDING_SIZE,
};
use libc::{c_float, c_void, size_t};
use std::collections::LinkedList;
use std::convert::AsRef;
use std::ffi::{CStr, CString};
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};

fn c_str(s: &str) -> CString {
    CString::new(s).expect("str to c str")
}

static REFCOUNT: i32 = 0;

pub struct RawYuv420p {
    pub width: u32,
    pub height: u32,
    pub bufsize: i32,
    pub linesize: [i32; 4],
    pub data: [*mut u8; 4],
}

impl Drop for RawYuv420p {
    fn drop(&mut self) {
        assert!(!self.data[0].is_null());
        unsafe {
            sys::av_free(self.data[0] as *mut c_void);
            self.data = [
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ];
        };
    }
}

impl RawYuv420p {
    pub fn luma_size(&self) -> u32 {
        self.width * self.height
    }
    pub fn chroma_size(&self) -> u32 {
        self.width * self.height / 4
    }
    pub unsafe fn to_vec(&self) -> Vec<u8> {
        let mut output = Vec::<u8>::new();
        let ptr = self.data[0];
        for i in 0..self.bufsize as usize {
            let val = ptr.add(i);
            let val = *val;
            output.push(val);
        }
        output
    }
    pub unsafe fn save(&self, path: &str) {
        println!(
            "ffplay -video_size {}x{} -pixel_format yuv420p {}",
            self.width, self.height, path,
        );
        std::fs::write(path, self.to_vec());
    }
    pub unsafe fn new(width: u32, height: u32) -> Self {
        use sys::AVPixelFormat_AV_PIX_FMT_YUV420P as AV_PIX_FMT_YUV420P;
        let pix_fmt: sys::AVPixelFormat = AV_PIX_FMT_YUV420P;
        let mut linesize = [0i32; 4];
        let mut data = [
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ];
        let bufsize = sys::av_image_alloc(
            data.as_mut_ptr(),
            linesize.as_mut_ptr(),
            width as i32,
            height as i32,
            pix_fmt,
            1,
        );
        RawYuv420p {
            width,
            height,
            bufsize,
            linesize,
            data,
        }
    }
    pub unsafe fn fill_from_frame(&mut self, frame: *mut sys::AVFrame) {
        use sys::AVPixelFormat_AV_PIX_FMT_YUV420P as AV_PIX_FMT_YUV420P;
        assert!(!frame.is_null());
        assert!((*frame).format == AV_PIX_FMT_YUV420P);
        sys::av_image_copy(
            self.data.as_mut_ptr(),
            self.linesize.as_mut_ptr(),
            (*frame).data.as_mut_ptr() as *mut *const u8,
            (*frame).linesize.as_ptr(),
            (*frame).format,
            (*frame).width,
            (*frame).height,
        );
    }
    pub unsafe fn to_higher(&self) -> Yuv420P {
        let output = std::slice::from_raw_parts(self.data[0], self.bufsize as usize);
        let expected_size = {
            let l = self.luma_size();
            let c = self.chroma_size();
            l + c + c
        };
        assert!(output.len() == (expected_size as usize));
        Yuv420P {
            width: self.width,
            height: self.height,
            data: output.to_vec(),
        }
    }
}

struct Decoder {
    fmt_ctx: *mut sys::AVFormatContext,
    video_dec_ctx: *mut sys::AVCodecContext,
    audio_dec_ctx: *mut sys::AVCodecContext,
    width: i32,
    height: i32,
    pix_fmt: sys::AVPixelFormat,
    video_stream: *mut sys::AVStream,
    audio_stream: *mut sys::AVStream,

    demux_ops: *mut sys::AVDictionary,

    decoded_video: LinkedList<RawYuv420p>,
    decoded_audio: LinkedList<u8>,

    video_dst_data: [*mut u8; 4],
    video_dst_linesize: [i32; 4],
    video_dst_bufsize: i32,

    video_stream_idx: i32,
    audio_stream_idx: i32,
    frame: *mut AVFrame,
    pkt: AVPacket,
    video_frame_count: u32,
    audio_frame_count: u32,
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe {
            if !self.demux_ops.is_null() {
                sys::av_dict_free(&mut self.demux_ops);
                self.demux_ops = std::ptr::null_mut();
            }
            if !self.video_dec_ctx.is_null() {
                sys::avcodec_free_context(&mut self.video_dec_ctx);
                self.video_dec_ctx = std::ptr::null_mut();
            }
            if !self.audio_dec_ctx.is_null() {
                sys::avcodec_free_context(&mut self.audio_dec_ctx);
                self.audio_dec_ctx = std::ptr::null_mut();
            }
            if !self.fmt_ctx.is_null() {
                sys::avformat_close_input(&mut self.fmt_ctx);
                self.fmt_ctx = std::ptr::null_mut();
            }
            if !self.frame.is_null() {
                sys::av_frame_free(&mut self.frame);
                self.frame = std::ptr::null_mut();
            }
            if !self.video_dst_data[0].is_null() {
                sys::av_free(self.video_dst_data[0] as *mut c_void);
                self.video_dst_data = [
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ];
            }
        };
    }
}

impl Decoder {
    pub unsafe fn new() -> Self {
        Decoder {
            fmt_ctx: sys::avformat_alloc_context(),
            video_dec_ctx: std::ptr::null_mut(),
            audio_dec_ctx: std::ptr::null_mut(),
            width: std::mem::zeroed(),
            height: std::mem::zeroed(),
            pix_fmt: std::mem::zeroed(),
            video_stream: std::ptr::null_mut(),
            audio_stream: std::ptr::null_mut(),

            demux_ops: std::ptr::null_mut(),

            decoded_audio: Default::default(),
            decoded_video: Default::default(),

            video_dst_data: std::mem::zeroed(),
            video_dst_linesize: std::mem::zeroed(),
            video_dst_bufsize: std::mem::zeroed(),

            video_stream_idx: -1,
            audio_stream_idx: -1,
            frame: std::mem::zeroed(),
            pkt: std::mem::zeroed(),
            video_frame_count: 0,
            audio_frame_count: 0,
        }
    }
}

unsafe fn decode_packet(got_frame: &mut i32, cached: i32, decoder: &mut Decoder) -> i32 {
    let mut ret: i32 = 0;
    let mut decoded: i32 = decoder.pkt.size;

    *got_frame = 0;

    if (decoder.pkt.stream_index == decoder.video_stream_idx) {
        // DECODE VIDEO FRAME
        ret = sys::avcodec_decode_video2(
            decoder.video_dec_ctx,
            decoder.frame,
            got_frame,
            &mut decoder.pkt,
        );
        if (ret < 0) {
            eprintln!("Error decoding video frame: {}", ret);
            return ret;
        }

        if (*got_frame > 0) {
            if ((*decoder.frame).width != decoder.width
                || (*decoder.frame).height != decoder.height
                || (*decoder.frame).format != decoder.pix_fmt)
            {
                // To handle this change, one could call av_image_alloc again and
                // decode the following frames into another rawvideo file.
                eprintln!("invalid width/height/pixel-format");
                return -1;
            }

            // copy decoded frame to destination buffer:
            // this is required since rawvideo expects non aligned data
            sys::av_image_copy(
                decoder.video_dst_data.as_mut_ptr(),
                decoder.video_dst_linesize.as_mut_ptr(),
                (*decoder.frame).data.as_ptr() as *mut *const u8,
                (*decoder.frame).linesize.as_ptr(),
                decoder.pix_fmt,
                decoder.width,
                decoder.height,
            );

            // WRITE TO RAWVIDEO FILE
            // fwrite(video_dst_data[0], 1, video_dst_bufsize, video_dst_file);
            {
                let mut output_picture: RawYuv420p = RawYuv420p::new(
                    (*decoder.frame).width as u32,
                    (*decoder.frame).height as u32,
                );

                // COPY DECODED FRAME TO DESTINATION BUFFER;
                // THIS IS REQUIRED SINCE RAWVIDEO EXPECTS NON ALIGNED DATA
                // sys::av_image_copy(
                //     output_picture.data.as_mut_ptr(),
                //     output_picture.linesize.as_mut_ptr(),
                //     decoder.video_dst_data.as_mut_ptr() as *mut *const u8,
                //     decoder.video_dst_linesize.as_mut_ptr(),
                //     AV_PIX_FMT_YUV420P,
                //     decoder.width,
                //     decoder.height,
                // );
                output_picture.fill_from_frame(decoder.frame);
                decoder.decoded_video.push_back(output_picture);
            }
        }
    } else if (decoder.pkt.stream_index == decoder.audio_stream_idx) {
        // DECODE AUDIO FRAME
        ret = sys::avcodec_decode_audio4(
            decoder.audio_dec_ctx,
            decoder.frame,
            got_frame,
            &decoder.pkt,
        );
        if (ret < 0) {
            panic!("Error decoding audio frame");
        }

        // Some audio decoders decode only part of the packet, and have to be
        // called again with the remainder of the packet data.
        // Sample: fate-suite/lossless-audio/luckynight-partial.shn
        // Also, some decoders might over-read the packet.
        decoded = {
            let x = ffmpeg_dev::extra::defs::ffmin(ret as usize, decoder.pkt.size as usize);
            x as i32
        };

        if (*got_frame > 0) {
            let mut unpadded_linesize: i32 = {
                (*decoder.frame).nb_samples * sys::av_get_bytes_per_sample((*decoder.frame).format)
            };

            // Write the raw audio data samples of the first plane. This works
            // fine for packed formats (e.g. AV_SAMPLE_FMT_S16). However,
            // most audio decoders output planar audio, which uses a separate
            // plane of audio samples for each channel (e.g. AV_SAMPLE_FMT_S16P).
            // In other words, this code will write only the first audio channel
            // in these cases.
            // You should use libswresample or libavfilter to convert the frame
            // to packed data.
            // fwrite(frame->extended_data[0], 1, unpadded_linesize, audio_dst_file);
            // unimplemented!();
        }
    }

    // If we use frame reference counting, we own the data and need
    // to de-reference it when we don't use it anymore
    if ((*got_frame > 0) && (REFCOUNT > 0)) {
        sys::av_frame_unref(decoder.frame);
    }

    return decoded;
}

unsafe fn open_codec_context(
    stream_idx: &mut i32,
    dec_ctx: &mut *mut AVCodecContext,
    fmt_ctx: *mut AVFormatContext,
    media_type: sys::AVMediaType,
) -> i32 {
    let mut ret: i32;
    let mut stream_index: i32;
    let mut st: *mut AVStream;
    let mut dec: *mut AVCodec = std::ptr::null_mut();
    let mut opts: *mut AVDictionary = std::ptr::null_mut();

    ret = sys::av_find_best_stream(fmt_ctx, media_type, -1, -1, std::ptr::null_mut(), 0);
    if (ret < 0) {
        println!("Could not find %s stream in input file");
        return ret;
    } else {
        stream_index = ret;
        st = *(*fmt_ctx).streams.offset(stream_index as isize);
        assert!(!st.is_null());
        assert!(!(*st).codecpar.is_null());

        // FIND DECODER FOR THE STREAM
        dec = sys::avcodec_find_decoder((*(*st).codecpar).codec_id);
        if (dec.is_null()) {
            unimplemented!("Failed to find codec");
            return unimplemented!();
        }

        // ALLOCATE A CODEC CONTEXT FOR THE DECODER
        *dec_ctx = sys::avcodec_alloc_context3(dec);
        if (dec_ctx.is_null()) {
            panic!("Failed to allocate the codec context");
            return unimplemented!();
        }

        // COPY CODEC PARAMETERS FROM INPUT STREAM TO OUTPUT CODEC CONTEXT
        ret = sys::avcodec_parameters_to_context(*dec_ctx, (*st).codecpar);
        if (ret < 0) {
            panic!("Failed to copy codec parameters to decoder context");
            return unimplemented!();
        }

        // INIT THE DECODERS
        ret = sys::avcodec_open2(*dec_ctx, dec, &mut opts);
        if (ret < 0) {
            panic!("Failed to open %s codec");
            return unimplemented!();
        }
        *stream_idx = stream_index;
    }

    return 0;
}

unsafe fn get_format_from_sample_fmt(fmt: &mut CString, sample_fmt: sys::AVSampleFormat) -> i32 {
    use sys::{
        AVSampleFormat, AVSampleFormat_AV_SAMPLE_FMT_DBL as AV_SAMPLE_FMT_DBL,
        AVSampleFormat_AV_SAMPLE_FMT_FLT as AV_SAMPLE_FMT_FLT,
        AVSampleFormat_AV_SAMPLE_FMT_S16 as AV_SAMPLE_FMT_S16,
        AVSampleFormat_AV_SAMPLE_FMT_S32 as AV_SAMPLE_FMT_S32,
        AVSampleFormat_AV_SAMPLE_FMT_U8 as AV_SAMPLE_FMT_U8,
    };
    let mut i: i32;

    #[repr(C)]
    struct SampleFmtEntry {
        sample_fmt: sys::AVSampleFormat,
        fmt_be: CString,
        fmt_le: CString,
    }

    let mut sample_fmt_entries: [SampleFmtEntry; 5] = [
        SampleFmtEntry {
            sample_fmt: AV_SAMPLE_FMT_U8,
            fmt_be: c_str("u8"),
            fmt_le: c_str("u8"),
        },
        SampleFmtEntry {
            sample_fmt: AV_SAMPLE_FMT_S16,
            fmt_be: c_str("s16be"),
            fmt_le: c_str("s16le"),
        },
        SampleFmtEntry {
            sample_fmt: AV_SAMPLE_FMT_S32,
            fmt_be: c_str("s32be"),
            fmt_le: c_str("s32le"),
        },
        SampleFmtEntry {
            sample_fmt: AV_SAMPLE_FMT_FLT,
            fmt_be: c_str("f32be"),
            fmt_le: c_str("f32le"),
        },
        SampleFmtEntry {
            sample_fmt: AV_SAMPLE_FMT_DBL,
            fmt_be: c_str("f64be"),
            fmt_le: c_str("f64le"),
        },
    ];
    *fmt = c_str("");
    let mut i = 0;
    while i < sample_fmt_entries.len() {
        i = i + 1;
        let mut entry: &mut SampleFmtEntry = &mut sample_fmt_entries[i];
        if sample_fmt == entry.sample_fmt {
            if sys::AV_HAVE_BIGENDIAN > 0 {
                *fmt = entry.fmt_be.clone();
            } else {
                *fmt = entry.fmt_le.clone();
            }
            return 0;
        }
    }

    println!("sample format is not supported as output format");
    return -1;
}

pub unsafe fn demux_decode(source: Vec<u8>) -> Vec<Yuv420P> {
    // DEBUG
    let suppress_log = true;
    sys::av_log_set_level(16);
    // SETUP
    let mut got_frame: i32 = std::mem::zeroed();
    let mut ret = 0;
    let mut decoder = Decoder::new();

    // CUSTOM AV-IO-CONTEXT HELPERS
    struct CallbackContext {
        buffer: Cursor<Vec<u8>>,
    }

    unsafe extern "C" fn read_packet(ctx: *mut c_void, buf: *mut u8, buf_size: i32) -> i32 {
        // INIT
        let ctx = (ctx as *mut CallbackContext).as_mut().expect("not null");
        assert!(buf_size >= 0);
        let buf_size = buf_size as usize;
        // CHECK
        let mut chunk = {
            let xs = ctx
                .buffer
                .get_ref()
                .split_at(ctx.buffer.position() as usize)
                .1
                .chunks(buf_size)
                .nth(0);
            if xs.is_none() {
                return ffmpeg_dev::extra::defs::averror_eof();
            }
            xs.expect("not empty").to_vec()
        };
        ctx.buffer.seek(SeekFrom::Current(chunk.len() as i64));
        assert!(chunk.len() <= buf_size);
        let chunk_size = chunk.len();
        for (ix, x) in chunk.into_iter().enumerate() {
            (*buf.add(ix)) = x;
        }
        // DONE
        chunk_size as i32
    }

    unsafe extern "C" fn seek_packet(
        opaque: *mut ::std::os::raw::c_void,
        offset: i64,
        whence: ::std::os::raw::c_int,
    ) -> i64 {
        use std::io::SeekFrom;
        // INIT
        let ctx = (opaque as *mut CallbackContext).as_mut().expect("not null");
        // CHECK
        assert!(whence >= 0);
        // MODES
        const SEEK_SET: i32 = 0;
        const SEEK_CUR: i32 = 1;
        const SEEK_END: i32 = 2;
        const AVSEEK_SIZE: i32 = sys::AVSEEK_SIZE as i32;
        // GO
        let seek_from = match whence {
            SEEK_SET => SeekFrom::Start(offset as u64),
            SEEK_CUR => SeekFrom::Current(offset),
            SEEK_END => SeekFrom::End(offset),
            AVSEEK_SIZE => {
                return ctx.buffer.get_ref().len() as i64;
            }
            _ => panic!("what to do here?"),
        };
        ctx.buffer.seek(seek_from).expect("seek to position") as i64
    }

    // INIT CUSTOM AV-IO-CONTEXT
    let avio_ctx_buffer_size = 4096;
    // let avio_ctx_buffer_size = source.len();
    let mut avio_ctx_buffer: *mut u8 = sys::av_malloc(avio_ctx_buffer_size) as *mut u8;
    let mut bd: CallbackContext = CallbackContext {
        buffer: Cursor::new(source),
    };
    let mut avio_ctx: *mut sys::AVIOContext = sys::avio_alloc_context(
        avio_ctx_buffer,
        avio_ctx_buffer_size as i32,
        0,
        (&mut bd as *mut CallbackContext) as *mut c_void,
        Some(read_packet),
        None,
        Some(seek_packet),
    );
    assert!(!avio_ctx.is_null());

    // OPEN INPUT FILE, AND ALLOCATE FORMAT CONTEXT
    assert!(!decoder.fmt_ctx.is_null());
    assert!(!decoder.fmt_ctx.is_null());
    assert!((*decoder.fmt_ctx).pb.is_null());
    (*decoder.fmt_ctx).pb = avio_ctx;
    (*decoder.fmt_ctx).flags = sys::AVFMT_FLAG_CUSTOM_IO as i32;
    (*decoder.fmt_ctx).probesize = 1200000;
    {
        let status = sys::avformat_open_input(
            &mut decoder.fmt_ctx,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut decoder.demux_ops,
        );
        if status < 0 {
            panic!("Could not open source file: {}", status);
        }
    }

    // RETRIEVE STREAM INFORMATION
    if (sys::avformat_find_stream_info(decoder.fmt_ctx, std::ptr::null_mut()) < 0) {
        panic!("Could not find stream information");
    }

    if (open_codec_context(
        &mut decoder.video_stream_idx,
        &mut decoder.video_dec_ctx,
        decoder.fmt_ctx,
        AVMEDIA_TYPE_VIDEO,
    ) >= 0)
    {
        decoder.video_stream = (*(*decoder.fmt_ctx).streams).add(decoder.video_stream_idx as usize);

        // ALLOCATE IMAGE WHERE THE DECODED IMAGE WILL BE PUT
        decoder.width = (*decoder.video_dec_ctx).width;
        decoder.height = (*decoder.video_dec_ctx).height;
        decoder.pix_fmt = (*decoder.video_dec_ctx).pix_fmt;
        ret = sys::av_image_alloc(
            decoder.video_dst_data.as_mut_ptr(),
            decoder.video_dst_linesize.as_mut_ptr(),
            decoder.width,
            decoder.height,
            decoder.pix_fmt,
            1,
        );
        if (ret < 0) {
            panic!("Could not allocate raw video buffer");
        }
        decoder.video_dst_bufsize = ret;
    }

    let get_audio_stream = false;
    if get_audio_stream {
        if (open_codec_context(
            &mut decoder.audio_stream_idx,
            &mut decoder.audio_dec_ctx,
            decoder.fmt_ctx,
            AVMEDIA_TYPE_AUDIO,
        ) >= 0)
        {
            decoder.audio_stream =
                (*(*decoder.fmt_ctx).streams).add(decoder.audio_stream_idx as usize);
        }
    }

    // DUMP INPUT INFORMATION TO STDERR
    if !suppress_log {
        sys::av_dump_format(decoder.fmt_ctx, 0, std::ptr::null(), 0);
    }

    if (decoder.audio_stream.is_null() && decoder.video_stream.is_null()) {
        panic!("Could not find audio or video stream in the input, aborting");
    }

    decoder.frame = sys::av_frame_alloc();
    if (decoder.frame.is_null()) {
        panic!("Could not allocate frame");
    }

    // INITIALIZE PACKET - SET DATA TO NULL - LET THE DEMUXER FILL IT
    sys::av_init_packet(&mut decoder.pkt);
    decoder.pkt.data = std::ptr::null_mut();
    decoder.pkt.size = 0;

    if (!decoder.video_stream.is_null()) {
        if !suppress_log {
            println!("Demuxing video from file");
        }
    }
    if (!decoder.audio_stream.is_null()) {
        if !suppress_log {
            println!("Demuxing audio from file");
        }
    }

    // READ FRAMES FROM THE FILE
    while sys::av_read_frame(decoder.fmt_ctx, &mut decoder.pkt) >= 0 {
        let mut orig_pkt: AVPacket = decoder.pkt.clone();
        {
            ret = decode_packet(&mut got_frame, 0, &mut decoder);
            if (ret < 0) {
                break;
            }
            decoder.pkt.data = decoder.pkt.data.add(ret as usize);
            decoder.pkt.size = decoder.pkt.size - ret;
        }
        while decoder.pkt.size > 0 {
            ret = decode_packet(&mut got_frame, 0, &mut decoder);
            if (ret < 0) {
                break;
            }
            decoder.pkt.data = decoder.pkt.data.add(ret as usize);
            decoder.pkt.size = decoder.pkt.size - ret;
        }
        sys::av_packet_unref(&mut orig_pkt);
    }

    // FLUSH CACHED FRAMES
    decoder.pkt.data = std::ptr::null_mut();
    decoder.pkt.size = 0;
    decode_packet(&mut got_frame, 1, &mut decoder);
    while got_frame > 0 {
        decode_packet(&mut got_frame, 1, &mut decoder);
    }

    if !suppress_log {
        println!("Demuxing succeeded");
    }

    // RETURN VALUES
    let decoded_frames = decoder
        .decoded_video
        .iter()
        .map(|x| x.to_higher())
        .collect::<Vec<_>>();

    // CLEANUP
    std::mem::drop(decoder);
    sys::avio_context_free(&mut avio_ctx);

    // DONE
    decoded_frames
}
