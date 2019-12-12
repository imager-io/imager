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
    AVFrame,
    AVDictionary,
    AVCodec,
    AVCodecContext,
    AVStream,
    AVPacket,
    AVFormatContext,
    AVOutputFormat,
    AVCodecParameters,
    AVCodecParserContext,
    AVMediaType,
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
    AV_INPUT_BUFFER_PADDING_SIZE,
    AVPixelFormat_AV_PIX_FMT_YUV420P as AV_PIX_FMT_YUV420P,
};

fn c_str(s: &str) -> CString {
    CString::new(s).expect("str to c str")
}

pub struct RawYuv420p {
    pub width: u32,
    pub height: u32,
    pub bufsize: i32,
    pub linesize: [i32; 4],
    pub data: [*mut u8; 4],
}

impl Drop for RawYuv420p {
    fn drop(&mut self) {
        unsafe {
            if !self.data[0].is_null() {
                sys::av_free(self.data[0] as *mut c_void);
            }
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
        for i in 0 .. self.bufsize as usize {
            let val = ptr.add(i);
            let val = *val;
            output.push(val);
        }
        output
    }
    pub unsafe fn save(&self, path: &str) {
        println!(
            "ffplay -s {}x{} -pix_fmt yuv420p {}",
            self.width,
            self.height,
            path,
        );
        std::fs::write(path, self.to_vec());
    }
    pub unsafe fn new(width: u32, height: u32) -> Self {
        use sys::{
            AVPixelFormat_AV_PIX_FMT_YUV420P as AV_PIX_FMT_YUV420P
        };
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
    pub unsafe fn fill_from_frame(&mut self, frame: *mut AVFrame) {
        use sys::{
            AVPixelFormat_AV_PIX_FMT_YUV420P as AV_PIX_FMT_YUV420P
        };
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
}


struct Decoder {
    demux_ops: *mut sys::AVDictionary,
    fmt_ctx: *mut sys::AVFormatContext,
    video_dec_ctx: *mut sys::AVCodecContext,
    video_stream: *mut AVStream,
    video_stream_idx: i32,
    frame: *mut AVFrame,
    pkt: *mut AVPacket,
    video_frame_count: i32,
    refcount: i32,
}

struct CallbackContext {
    buffer: Vec<u8>,
}

unsafe extern "C" fn read_packet(
    ctx: *mut c_void,
    buf: *mut u8,
    buf_size: i32,
) -> i32 {
    // INIT
    let ctx = ctx as *mut CallbackContext;
    let ctx = ctx.as_mut().expect("not null");
    let buf_size = ffmpeg_dev::extra::defs::sys_ffmin(
        buf_size as usize,
        ctx.buffer.len(),
    );
    // CHECK
    if buf_size <= 0 {
        return ffmpeg_dev::extra::defs::averror_eof();
    }
    // COPY & UPDATE
    let buf_size = buf_size;
    let chunk = ctx.buffer
        .drain(0 .. buf_size)
        .collect::<Vec<_>>();
    for i in 0 .. chunk.len() {
        *buf.add(i) = chunk[i];
    }
    // DONE
    buf_size as i32
}

unsafe fn decode_packet(
    decoder: &mut Decoder,
    output_buffer: &mut Vec<RawYuv420p>,
    got_frame: &mut bool,
) -> i32 {
    let mut decoded: i32 = (*decoder.pkt).size;
    *got_frame = false;
    if ((*decoder.pkt).stream_index == decoder.video_stream_idx) {
        let mut got_frame_c: i32 = 0;
        let res = sys::avcodec_decode_video2(
            decoder.video_dec_ctx,
            decoder.frame,
            &mut got_frame_c,
            decoder.pkt,
        );
        if got_frame_c > 0 {
            *got_frame = true;
        } else {
            *got_frame = false;
        }

        assert!(res >= 0);

        if *got_frame {
            let mut output_picture: RawYuv420p = RawYuv420p::new(
                (*decoder.frame).width as u32,
                (*decoder.frame).height as u32,
            );

            // COPY DECODED FRAME TO DESTINATION BUFFER;
            // THIS IS REQUIRED SINCE RAWVIDEO EXPECTS NON ALIGNED DATA
            sys::av_image_copy(
                output_picture.data.as_mut_ptr(),
                output_picture.linesize.as_mut_ptr(),
                (*decoder.frame).data.as_mut_ptr() as *mut *const u8,
                (*decoder.frame).linesize.as_mut_ptr(),
                AV_PIX_FMT_YUV420P,
                (*decoder.frame).width,
                (*decoder.frame).height,
            );
            output_buffer.push(output_picture);
        }
    }

    // IF WE USE FRAME REFERENCE COUNTING;
    // WE OWN THE DATA AND NEED TO DE-REFERENCE
    // IT WHEN WE DON'T USE IT ANYMORE
    if (*got_frame && (decoder.refcount > 0)) {
        sys::av_frame_unref(decoder.frame);
    }
    
    decoded
}

pub unsafe fn demux_decode() {
    // I/O
    let input_path = "assets/samples/3183183.jpg";
    // let input_path = "assets/samples/pexels-photo-1153655.jpeg";
    // let input_path = "assets/samples/test.h264";
    // let output_path = "assets/output/test.mp4";
    assert!(PathBuf::from(input_path).exists());
    let source = std::fs::read(input_path).expect("source file");
    
    // INIT DECODER CONTEXT
    let mut decoder: Decoder = Decoder {
        demux_ops: std::ptr::null_mut(),
        fmt_ctx: sys::avformat_alloc_context(),
        video_dec_ctx: std::ptr::null_mut(),
        video_stream: std::ptr::null_mut(),
        video_stream_idx: -1,
        frame: std::ptr::null_mut(),
        pkt: sys::av_packet_alloc(),
        video_frame_count: 0,
        refcount: 0,
    };

    assert!(!decoder.fmt_ctx.is_null());
    assert!(!decoder.pkt.is_null());

    // SOURCE META
    let is_raw_video = false;
    if is_raw_video {
        let resolution: &str = unimplemented!();
        assert!(sys::av_dict_set(
            &mut decoder.demux_ops,
            c_str("pixel_format").as_ptr(),
            c_str("yuv420p").as_ptr(),
            0,
        ) >= 0);
        assert!(sys::av_dict_set(
            &mut decoder.demux_ops,
            c_str("video_size").as_ptr(),
            c_str(resolution).as_ptr(),
            0,
        ) >= 0);
        (*decoder.fmt_ctx).iformat = sys::av_find_input_format(c_str("rawvideo").as_ptr());
    } else {
        (*decoder.fmt_ctx).iformat = sys::av_find_input_format(c_str("mp4").as_ptr());
        // (*decoder.fmt_ctx).iformat = sys::av_find_input_format(c_str("image2pipe").as_ptr());
    }
    assert!(!(*decoder.fmt_ctx).iformat.is_null());

    // INIT CUSTOM AV-IO-CONTEXT
    let avio_ctx_buffer_size = 4096;
    let mut avio_ctx_buffer: *mut u8 = sys::av_malloc(avio_ctx_buffer_size) as *mut u8;
    let mut bd: CallbackContext = CallbackContext{
        buffer: source,
    };
    let mut avio_ctx: *mut sys::AVIOContext = sys::avio_alloc_context(
        avio_ctx_buffer,
        avio_ctx_buffer_size as i32,
        0,
        (&mut bd as *mut CallbackContext) as *mut c_void,
        Some(read_packet),
        None,
        None,
    );
    assert!(!avio_ctx.is_null());

    // OPEN INPUT
    (*decoder.fmt_ctx).pb = avio_ctx;
    assert!(sys::avformat_open_input(
        &mut decoder.fmt_ctx,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &mut decoder.demux_ops,
    ) >= 0);

    // RETRIEVE STREAM INFORMATION
    assert!(sys::avformat_find_stream_info(
        decoder.fmt_ctx,
        std::ptr::null_mut(),
    ) >= 0);

    // INIT CODEC-DECODER
    unsafe fn open_codec_context(
        decoder: &mut Decoder,
    ) {
        let stream_type: AVMediaType = AVMEDIA_TYPE_VIDEO;
        let mut stream_index: i32;
        let mut st: *mut AVStream = std::ptr::null_mut();
        let mut dec: *mut AVCodec = std::ptr::null_mut();
        let mut opts: *mut AVDictionary = std::ptr::null_mut();

        stream_index = sys::av_find_best_stream(
            decoder.fmt_ctx,
            stream_type,
            -1,
            -1,
            std::ptr::null_mut(),
            0,
        );
        assert!(stream_index >= 0);
        {
            st = (*(*decoder.fmt_ctx).streams).add(stream_index as usize);

            // FIND DECODER FOR THE STREAM
            dec = sys::avcodec_find_decoder((*(*st).codecpar).codec_id);
            assert!(!dec.is_null());

            // ALLOCATE A CODEC CONTEXT FOR THE 'ACTUAL' DECODER
            decoder.video_dec_ctx = sys::avcodec_alloc_context3(dec);
            assert!(!decoder.video_dec_ctx.is_null());

            // COPY CODEC PARAMETERS FROM INPUT STREAM TO OUTPUT CODEC CONTEXT
            assert!(sys::avcodec_parameters_to_context(
                decoder.video_dec_ctx,
                (*st).codecpar,
            ) >= 0);

            // INIT THE DECODERS, WITH OR WITHOUT REFERENCE COUNTING
            assert!(
                sys::av_dict_set(
                    &mut opts,
                    c_str("refcounted_frames").as_ptr(),
                    {
                        // decoder.refcount ? "1" : "0"
                        if decoder.refcount > 0 {
                            c_str("1")
                        } else {
                            c_str("0")
                        }
                    }.as_ptr(),
                    0,
                ) >= 0,
            );
            assert!(sys::avcodec_open2(decoder.video_dec_ctx, dec, &mut opts ) == 0);
            decoder.video_stream_idx = stream_index;
        }
    }
    open_codec_context(&mut decoder);

    // SET CODEC-DECODER STREAM
    decoder.video_stream = (*(*decoder.fmt_ctx).streams).add(decoder.video_stream_idx as usize);

    // DUMP INPUT INFORMATION TO STDERR
    sys::av_dump_format(
        decoder.fmt_ctx,
        0,
        c_str(input_path).as_ptr(),
        0,
    );
    assert!(!decoder.video_stream.is_null());
    decoder.frame = sys::av_frame_alloc();
    assert!(!decoder.frame.is_null());

    // INITIALIZE PACKET, SET DATA TO NULL, LET THE DEMUXER FILL IT
    sys::av_init_packet(decoder.pkt);
    (*decoder.pkt).data = std::ptr::null_mut();
    (*decoder.pkt).size = 0;
    let mut got_frame = std::mem::zeroed();
    let mut output: Vec<RawYuv420p> = Vec::new();
    while sys::av_read_frame(decoder.fmt_ctx, decoder.pkt) >= 0 {
        let mut orig_pkt: &mut AVPacket = decoder.pkt
            .as_mut()
            .expect("not null");
        let run = |decoder: &mut Decoder, output: &mut Vec<RawYuv420p>, got_frame: &mut bool| -> Result<(), ()> {
            let ret = decode_packet(
                decoder,
                output,
                got_frame,
            );
            if ret < 0 {
                return Err(());
            }
            (*decoder.pkt).data = (*decoder.pkt).data.add(1);
            (*decoder.pkt).size = (*decoder.pkt).size - 1;
            Ok(())
        };
        if run(&mut decoder, &mut output, &mut got_frame).is_err() {
            break;
        }
        while (*decoder.pkt).size > 0 {
            if run(&mut decoder, &mut output, &mut got_frame).is_err() {
                break;
            }
        }
        sys::av_packet_unref(orig_pkt);
    }

    // // FLUSH CACHED FRAMES
    // (*decoder.pkt).data = std::ptr::null_mut();
    // (*decoder.pkt).size = 0;
    // decode_packet(
    //     &mut decoder,
    //     &mut output,
    //     &mut got_frame,
    // );
    // while (got_frame) {
    //     decode_packet(
    //         &mut decoder,
    //         &mut output,
    //         &mut got_frame,
    //     );
    // }

    // // CLEANUP
    // sys::av_dict_free(&mut decoder.demux_ops);
    // sys::avcodec_free_context(&mut decoder.video_dec_ctx);
    // sys::avformat_close_input(&mut decoder.fmt_ctx);
    // sys::av_frame_free(&mut decoder.frame);
    // // NOTE: THE INTERNAL BUFFER COULD HAVE CHANGED, AND BE != AVIO_CTX_BUFFER
    // if !avio_ctx.is_null() {
    //     sys::av_freep((*avio_ctx).buffer as *mut c_void);
    // }
    // sys::avio_context_free(&mut avio_ctx);

    // // DONE
    // println!("output frames: {}", output.len());
}

pub fn run() {
    unsafe {
        demux_decode();
    };
}