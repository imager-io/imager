// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::collections::LinkedList;
use std::convert::AsRef;
use std::path::{PathBuf, Path};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float, c_void};
use ffmpeg_dev::extra::defs;
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
        for i in 0 .. self.bufsize as usize {
            let val = ptr.add(i);
            let val = *val;
            output.push(val);
        }
        output
    }
    pub unsafe fn save(&self, path: &str) {
        println!(
            "ffplay -video_size {}x{} -pixel_format yuv420p {}",
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

/////////////////////////////////////////////////////////////////////
// HIGHER-LEVEL TYPES
/////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct Yuv420P {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Yuv420P {
    pub fn luma_size(&self) -> u32 {
        self.width * self.height
    }
    pub fn chroma_size(&self) -> u32 {
        self.width * self.height / 4
    }
    pub unsafe fn from_raw(raw: &RawYuv420p) -> Self {
        let data = std::slice::from_raw_parts(raw.data[0], raw.bufsize as usize);
        Yuv420P {
            width: raw.width,
            height: raw.height,
            data: data.to_vec(),
        }
    }
    pub fn save(&self, path: &str) {
        println!(
            "ffplay -video_size {}x{} -pixel_format yuv420p {}",
            self.width,
            self.height,
            path,
        );
        std::fs::write(path, &self.data);
    }
}

#[derive(Debug, Clone)]
pub struct VideoBuffer {
    width: u32,
    height: u32,
    frames: Vec<Yuv420P>,
}

impl VideoBuffer {
    pub fn load_from_memory(source: &[u8]) -> Result<Self, ()> {
        let result = unsafe {
            crate::format::decode::demux_decode(source.to_vec())
        };
        assert!(!result.is_empty());
        let width = result[0].width;
        let height = result[0].height;
        Ok(VideoBuffer {
            width,
            height,
            frames: result,
        })
    }
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, ()> {
        assert!(path.as_ref().exists());
        let source = std::fs::read(path).expect("VideoBuffer::open - read source file");
        VideoBuffer::load_from_memory(&source)
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub fn as_frames(&self) -> &[Yuv420P] {
        self.frames.as_ref()
    }
}