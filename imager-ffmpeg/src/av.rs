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
};


pub const NOPTS_VALUE: i64 = -9223372036854775808;
pub const AVERROR_EAGAIN: i32 = 35;

fn c_str(s: &str) -> CString {
    CString::new(s).expect("str to c str")
}


// static void pgm_save(unsigned char *buf, int wrap, int xsize, int ysize,
//     char *filename)
// {
//     FILE *f;
//     int i;

//     f = fopen(filename,"w");
//     fprintf(f, "P5\n%d %d\n%d\n", xsize, ysize, 255);
//     for (i = 0; i < ysize; i++)
//          fwrite(buf + i * wrap, 1, xsize, f);
//     fclose(f);
// }

pub struct RawYuv420p {
    width: u32,
    height: u32,
    bufsize: i32,
    linesize: [i32; 4],
    data: [*mut u8; 4],
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

unsafe fn decode_video() {
    unsafe fn decode(
        dec_ctx: *mut AVCodecContext,
        frame: *mut AVFrame,
        pkt: *mut AVPacket,
        output: &mut Vec<RawYuv420p>,
    ) {
        let mut buf = vec![0u8; 1024];
        let mut ret = sys::avcodec_send_packet(dec_ctx, pkt);
        assert!(ret >= 0);
        while ret >= 0 {
            ret = sys::avcodec_receive_frame(dec_ctx, frame);
            if (ret < 0) {
                return;
            }
            let done = {
                ret == ffmpeg_dev::extra::defs::averror(ffmpeg_dev::extra::defs::eagain()) ||
                ret == ffmpeg_dev::extra::defs::averror_eof()
            };
            if done {
                return;
            }
            assert!(ret >= 0);
            // WRITE DECODED FRAME
            let mut decoded = RawYuv420p::new((*frame).width as u32, (*frame).height as u32);
            decoded.fill_from_frame(frame);
            output.push(decoded);
        }
    }
    // I/O
    let input_path = "assets/samples/test.h264";
    let output_path = "assets/output/test.yuv";
    assert!(PathBuf::from(input_path).exists());
    let input_path_cstr = c_str(input_path);
    let output_path_cstr = c_str(output_path);
    let mut f = std::fs::read(input_path).expect("source file");
    // MISC
    const INBUF_SIZE: u32 = 4096;
    // SETUP AV STATE
    let mut codec: *mut AVCodec = sys::avcodec_find_decoder(AV_CODEC_ID_H264);
    assert!(!codec.is_null());
    let mut parser: *mut AVCodecParserContext = sys::av_parser_init((*codec).id as i32);
    let mut c: *mut AVCodecContext = sys::avcodec_alloc_context3(codec);
    assert!(!c.is_null());
    let mut frame: *mut AVFrame = sys::av_frame_alloc();
    let mut inbuf: Vec<u8> = vec![0; (INBUF_SIZE + AV_INPUT_BUFFER_PADDING_SIZE) as usize];
    // let mut data: Vec<u8> = Vec::new();
    let mut pkt: *mut AVPacket = sys::av_packet_alloc();
    assert!(!pkt.is_null());
    // OPEN
    assert!(sys::avcodec_open2(c, codec, std::ptr::null_mut()) >= 0);
    let mut output = Vec::<RawYuv420p>::new();
    let mut eof = false;
    while eof == false {
        let inbuf_size = {
            if f.len() < INBUF_SIZE as usize {
                f.len()
            } else {
                INBUF_SIZE as usize
            }
        };
        let inbuf = f
            .drain(0..inbuf_size)
            .collect::<Vec<u8>>();
        let mut inbuf_size = inbuf_size as isize;
        if inbuf.is_empty() {
            eof = true;
            break;
        }
        while inbuf_size > 0 {
            let ret = sys::av_parser_parse2(
                parser,
                c,
                &mut (*pkt).data,
                &mut (*pkt).size,
                inbuf.as_ptr(),
                inbuf.len() as i32,
                NOPTS_VALUE,
                NOPTS_VALUE,
                0,
            );
            assert!(ret >= 0);
            inbuf_size = inbuf_size - (ret as isize);
            if (*pkt).size > 0 {
                decode(c, frame, pkt, &mut output);
            }
        }
    }
    // FLUSH THE DECODER
    decode(c, frame, std::ptr::null_mut(), &mut output);
    // CLEANUP
    sys::av_parser_close(parser);
    sys::avcodec_free_context(&mut c);
    sys::av_frame_free(&mut frame);
    sys::av_packet_free(&mut pkt);
    // DONE
    println!("frames: {}", output.len());
    output[50].save(output_path);
}


///////////////////////////////////////////////////////////////////////////////
// DEV
///////////////////////////////////////////////////////////////////////////////

pub fn run() {
    unsafe {
        decode_video();
    };
}

