// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::rc::Rc;
use std::convert::AsRef;
use std::path::{PathBuf, Path};
use std::collections::VecDeque;
use itertools::Itertools;
use crate::yuv420p::Yuv420P;

pub trait Stream {
    fn restart(&mut self);
    fn next(&mut self) -> Option<Yuv420P>;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn dimensions(&self) -> (u32, u32) {
        (self.width(), self.height())
    }
}

///////////////////////////////////////////////////////////////////////////////
// SINGLE IMAGE STREAM
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct SingleImage {
    pub yuv: Yuv420P,
    pub done: bool,
}

impl SingleImage {
    pub fn new(width: u32, height: u32) -> Self {
        let luma_size = width * height;
        let chroma_size = luma_size / 4;
        let yuv = Yuv420P {
            y: vec![0; luma_size as usize],
            u: vec![0; chroma_size as usize],
            v: vec![0; chroma_size as usize],
            width,
            height,
        };
        SingleImage {yuv, done: false}
    }
    pub fn empty(width: u32, height: u32) -> Self {
        let yuv = Yuv420P {
            y: Vec::new(),
            u: Vec::new(),
            v: Vec::new(),
            width,
            height,
        };
        SingleImage {yuv, done: false}
    }
    pub fn fill_from_yuv_file<P: AsRef<Path>>(&mut self, path: P) {
        let mut data = std::fs::read(path).expect("missing yuv source file for stream");
        let luma_size = self.yuv.width * self.yuv.height;
        let chroma_size = luma_size / 4;
        self.yuv.y = data.drain(0 .. luma_size as usize).collect::<Vec<u8>>();
        self.yuv.u = data.drain(0 .. chroma_size as usize).collect::<Vec<u8>>();
        self.yuv.v = data.drain(0 .. chroma_size as usize).collect::<Vec<u8>>();
        assert!(data.is_empty());
    }
}

impl Stream for SingleImage {
    fn restart(&mut self) {
        self.done = false;
    }
    fn next(&mut self) -> Option<Yuv420P> {
        let result = if self.done == true {
            None
        } else {
            Some(self.yuv.clone())
        };
        self.done = true;
        result
    }
    fn width(&self) -> u32 {
        self.yuv.width
    }
    fn height(&self) -> u32 {
        self.yuv.height
    }
}



///////////////////////////////////////////////////////////////////////////////
// FILE STREAM
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct FileStream {
    pub width: u32,
    pub height: u32,
    pub list: Vec<(usize, PathBuf)>,
    pub cursor: usize,
}

impl FileStream {
    pub fn new<P: AsRef<Path>>(path: P, width: u32, height: u32) -> Self {
        let files = std::fs::read_dir(path)
            .expect("read source dir")
            .filter_map(Result::ok)
            .filter(|x| x.file_type().expect("source filetype").is_file())
            .map(|x| x.path())
            .filter_map(|x| {
                let file_name = x
                    .file_name()
                    .expect("missing file name")
                    .to_str()
                    .expect("file name to str")
                    .chars()
                    .take_while(|x| x.is_ascii_digit())
                    .collect::<String>();
                let index = file_name.parse::<usize>().ok()?;
                Some((index, x))
            })
            .sorted_by(|(i, _), (j, _)| {
                i.cmp(j)
            })
            .collect::<Vec<_>>();
        FileStream {
            width,
            height,
            list: files,
            cursor: 0,
        }
    }
}

impl Stream for FileStream {
    fn restart(&mut self) {
        self.cursor = 0;
    }
    fn next(&mut self) -> Option<Yuv420P> {
        let (index, path) = self.list.get(self.cursor)?;
        let yuv = Yuv420P::open(&path);
        self.cursor = self.cursor + 1;
        Some(yuv)
    }
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
}
