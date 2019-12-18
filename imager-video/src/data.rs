// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::rc::Rc;
use std::collections::LinkedList;
use std::convert::{AsRef, TryFrom};
use std::path::{PathBuf, Path};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use libc::{size_t, c_float, c_void};
use itertools::Itertools;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer};
use rayon::prelude::*;
use webp_dev::sys::webp::{
    self as webp_sys,
    WebPConfig,
    WebPPicture,
    WebPMemoryWriter,
};


///////////////////////////////////////////////////////////////////////////////
// INTERNAL HELPERS
///////////////////////////////////////////////////////////////////////////////

pub fn open_dir_sorted_paths<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
    std::fs::read_dir(path)
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
        .map(|(_, x)| x)
        .collect::<Vec<_>>()
}

unsafe fn convert_to_yuv_using_webp(source: &DynamicImage) -> Yuv420P {
    let (width, height) = source.dimensions();
    assert!(width < webp_sys::WEBP_MAX_DIMENSION);
    assert!(height < webp_sys::WEBP_MAX_DIMENSION);
    let mut picture: WebPPicture = unsafe {std::mem::zeroed()};
    unsafe {
        assert!(webp_sys::webp_picture_init(&mut picture) != 0);
    };
    let argb_stride = width;
    picture.use_argb = 1;
    picture.width = width as i32;
    picture.height = height as i32;
    picture.argb_stride = argb_stride as i32;
    // FILL PIXEL BUFFERS
    unsafe {
        let mut pixel_data = source
            .to_rgb()
            .pixels()
            .flat_map(|px: &::image::Rgb<u8>| px.0.to_vec())
            .collect::<Vec<_>>();
        let full_stride = argb_stride * 3;
        let status = webp_sys::webp_picture_import_rgb(
            &mut picture,
            pixel_data.as_mut_ptr(),
            full_stride as i32,
        );
        // CHECKS
        let expected_size = argb_stride * height * 3;
        assert!(pixel_data.len() as u32 == expected_size);
        assert!(status != 0);
        // CLEANUP
        std::mem::drop(pixel_data);
    };
    // CHECKS
    assert!(picture.use_argb == 1);
    assert!(picture.y.is_null());
    assert!(!picture.argb.is_null());
    // CONVERT
    unsafe {
        assert!(webp_sys::webp_picture_sharp_argb_to_yuva(&mut picture) != 0);
        assert!(picture.use_argb == 0);
        assert!(!picture.y.is_null());
    };
    let data = unsafe {
        assert!(picture.y_stride as u32 == width);
        assert!(picture.uv_stride as u32 == width / 2);
        let y_size = width * height;
        let uv_size = width * height / 4;
        let y = std::slice::from_raw_parts_mut(picture.y, y_size as usize).to_vec();
        let u = std::slice::from_raw_parts_mut(picture.u, uv_size as usize).to_vec();
        let v = std::slice::from_raw_parts_mut(picture.v, uv_size as usize).to_vec();
        [y, u, v].concat()
    };
    // CLEANUP
    unsafe {
        webp_sys::webp_picture_free(&mut picture);
    };
    std::mem::drop(picture);
    // DONE
    let result = Yuv420P {data, width, height};
    assert!(result.expected_yuv420p_size());
    result
}

unsafe fn convert_to_rgba_using_webp(source: &Yuv420P) -> DynamicImage {
    let (width, height) = source.dimensions();
    assert!(width < webp_sys::WEBP_MAX_DIMENSION);
    assert!(height < webp_sys::WEBP_MAX_DIMENSION);
    let mut picture: WebPPicture = unsafe {std::mem::zeroed()};
    assert!(webp_sys::webp_picture_init(&mut picture) != 0);
    let argb_stride = width;
    picture.use_argb = 0;
    picture.width = width as i32;
    picture.height = height as i32;
    picture.argb_stride = argb_stride as i32;
    picture.colorspace = webp_sys::WEBP_YUV420;
    // ALLOCATE
    assert!(webp_sys::webp_picture_alloc(&mut picture) != 0);
    // FILL SOURCE PIXEL BUFFERS
    {
        // CHECKS
        assert!(!picture.y.is_null());
        assert!(!picture.u.is_null());
        assert!(!picture.v.is_null());
        // GO
        let y_size = source.luma_size();
        let uv_size = source.chroma_size();
        let mut y = std::slice::from_raw_parts_mut(picture.y, y_size as usize);
        let mut u = std::slice::from_raw_parts_mut(picture.u, uv_size as usize);
        let mut v = std::slice::from_raw_parts_mut(picture.v, uv_size as usize);
        y.copy_from_slice(source.y());
        u.copy_from_slice(source.u());
        v.copy_from_slice(source.v());
    };
    // CONVERT
    assert!(picture.argb.is_null());
    assert!(webp_sys::webp_picture_has_transparency(&picture) == 0);
    assert!(webp_sys::webp_picture_yuva_to_argb(
        &mut picture,
    ) != 0);
    // CHECKS
    assert!(picture.use_argb == 1);
    assert!(!picture.argb.is_null());
    assert!(webp_sys::webp_picture_has_transparency(&picture) == 0);
    // GET RESULT DATA
    assert!(picture.argb_stride as u32 == width);
    let rgba_output = ::image::RgbaImage::from_fn(width, height, |x_pos, y_pos| {
        let ptr_ix = (y_pos * width) + x_pos;
        let px = *picture.argb.add(ptr_ix as usize);
        let [a, r, g, b]: [u8; 4] = std::mem::transmute(px.to_be());
        ::image::Rgba([r, g, b, a])
    });
    let rgba_output = DynamicImage::ImageRgba8(rgba_output);
    // CLEANUP
    unsafe {
        webp_sys::webp_picture_free(&mut picture);
    };
    std::mem::drop(picture);
    // DONE
    rgba_output
}


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
    pub fn open_image<P: AsRef<Path>>(path: P) -> Result<Self, ()> {
        let source = ::image::open(path).expect("Yuv420P::open_image - load image");
        Yuv420P::from_image(&source)
    }
    pub fn from_image(source: &DynamicImage) -> Result<Self, ()> {
        Ok(unsafe{ convert_to_yuv_using_webp(source) })
    }
    pub fn open_yuv<P: AsRef<Path>>(path: P, width: u32, height: u32) -> Result<Self, ()> {
        let source = std::fs::read(path).expect("read raw yuv file");
        let result = Yuv420P {
            width,
            height,
            data: source,
        };
        assert!(result.expected_yuv420p_size());
        Ok(result)
    }
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
    pub fn to_rgba_image(&self) -> DynamicImage {
        unsafe {convert_to_rgba_using_webp(self)}
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
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}


///////////////////////////////////////////////////////////////////////////////
// VIDEO FRAME BUFFERS
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct VideoBuffer {
    width: u32,
    height: u32,
    frames: Rc<Vec<Yuv420P>>,
    cursor: usize,
}

impl VideoBuffer {
    pub fn singleton(frame: Yuv420P) -> Self {
        VideoBuffer {
            width: frame.width,
            height: frame.height,
            frames: Rc::new(vec![frame]),
            cursor: 0,
        }
    }
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
            frames: Rc::new(result),
            cursor: 0,
        })
    }
    pub fn open_video<P: AsRef<Path>>(path: P) -> Result<Self, ()> {
        assert!(path.as_ref().exists());
        let source = std::fs::read(path).expect("VideoBuffer::open - read source file");
        VideoBuffer::load_from_memory(&source)
    }
    pub fn open_image_dir<P: AsRef<Path>>(dir_path: P) -> Result<Self, ()> {
        assert!(dir_path.as_ref().exists());
        let frames = open_dir_sorted_paths(dir_path)
            .into_par_iter()
            .map(|path| Yuv420P::open_image(&path).expect("open and decode image"))
            .collect::<Vec<_>>();
        assert!(!frames.is_empty());
        let (width, height) = {
            let w = frames[0].width;
            let h = frames[0].height;
            (w, h)
        };
        Ok(VideoBuffer {
            width,
            height,
            frames: Rc::new(frames),
            cursor: 0,
        })
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
    pub fn into_frames(self) -> Vec<Yuv420P> {
        let refs = Rc::strong_count(&self.frames);
        if refs == 0 {
            Rc::try_unwrap(self.frames).expect("shuld have no other refs")
        } else {
            self.frames.as_ref().clone()
        }
    }
    pub fn next(&mut self) -> Option<&Yuv420P> {
        let frame = self.frames.get(self.cursor)?;
        self.cursor = self.cursor + 1;
        Some(frame)
    }
    pub fn set_cursor(&mut self, cursor_pos: usize) {
        self.cursor = cursor_pos;
    }
    pub fn position(&self) -> usize {
        self.cursor
    }
    pub fn as_fresh_cursor(&self) -> VideoBuffer {
        VideoBuffer {
            width: self.width,
            height: self.height,
            frames: self.frames.clone(),
            cursor: self.cursor,
        }
    }
}
