// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::collections::LinkedList;
use std::convert::AsRef;
use std::path::{PathBuf, Path};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float, c_void};


///////////////////////////////////////////////////////////////////////////////
// PICTURE BUFFERS
///////////////////////////////////////////////////////////////////////////////

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
    pub fn expected_yuv420p_size(&self) -> bool {
        let expected_size = {
            let l = self.luma_size();
            let c = self.chroma_size();
            l + c + c
        };
        self.data.len() == (expected_size as usize)
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
    pub fn y(&self) -> &[u8] {
        assert!(self.expected_yuv420p_size());
        let end = self.luma_size();
        self.data.get(0 .. end as usize).expect("bad (Y) plane size")
    }
    pub fn u(&self) -> &[u8] {
        assert!(self.expected_yuv420p_size());
        let plane = self.data
            .as_slice()
            .split_at(self.luma_size() as usize).1
            .chunks(self.chroma_size() as usize)
            .nth(0)
            .expect("bad (U) plane chunk size");
        assert!(plane.len() == self.chroma_size() as usize);
        plane
    }
    pub fn v(&self) -> &[u8] {
        assert!(self.expected_yuv420p_size());
        let plane = self.data
            .as_slice()
            .split_at(self.luma_size() as usize).1
            .chunks(self.chroma_size() as usize)
            .nth(1)
            .expect("bad (V) plane chunk size");
        assert!(plane.len() == self.chroma_size() as usize);
        plane
    }
}


///////////////////////////////////////////////////////////////////////////////
// VIDEO FRAME BUFFERS
///////////////////////////////////////////////////////////////////////////////

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
