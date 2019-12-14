// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use rayon::prelude::*;
use image::{
    DynamicImage,
    GenericImageView,
    FilterType,
    ColorType
};


///////////////////////////////////////////////////////////////////////////////
// MISC
///////////////////////////////////////////////////////////////////////////////

pub static RESOLUTION_1080P: (u32, u32) = (1920, 1080);


///////////////////////////////////////////////////////////////////////////////
// NATIVE ENCODER
///////////////////////////////////////////////////////////////////////////////

pub fn native_encode_as_jpeg(input: &DynamicImage, quality: u8) -> Vec<u8> {
    let width = input.width();
    let height = input.height();
    let (raw_input, colorspace): (Vec<u8>, ColorType) = match &input {
        DynamicImage::ImageLuma8(gray_image) => {
            (gray_image.to_vec(), ColorType::Gray(8))
        }
        DynamicImage::ImageRgb8(rgb_image) => {
            (rgb_image.to_vec(), ColorType::RGB(8))
        }
        DynamicImage::ImageBgr8(bgr_image) => {
            (bgr_image.to_vec(), ColorType::BGR(8))
        }
        DynamicImage::ImageLumaA8(_) | DynamicImage::ImageBgra8(_) | DynamicImage::ImageRgba8(_) | DynamicImage::ImageBgra8(_) => {
            let rgb = input.to_rgb().to_vec();
            (rgb, ColorType::RGB(8))
        }
    };
    let mut output = Vec::new();
    let mut encoder = image::jpeg::JPEGEncoder::new_with_quality(&mut output, quality);
    let r = encoder.encode(
        raw_input.as_ref(),
        width,
        height,
        colorspace
    );
    r.expect(&format!("encode as 8-bit {:?} jpeg", colorspace));
    assert!(!output.is_empty());
    output
}


///////////////////////////////////////////////////////////////////////////////
// RESIZE BY MAX RESOLUTION
///////////////////////////////////////////////////////////////////////////////

pub fn downsize(source: DynamicImage, max_width: u32, max_height: u32) -> DynamicImage {
    if source.dimensions() > (max_width, max_height) {
        let source = source.resize(max_width, max_height, FilterType::Lanczos3);
        source
    } else {
        source
    }
}



///////////////////////////////////////////////////////////////////////////////
// RESIZE BY PIXEL COUNT
///////////////////////////////////////////////////////////////////////////////

pub static PIXEL_COUNT_8K: u32 = 8192 * 4320;
pub static PIXEL_COUNT_4K: u32 = 3840 * 2160;
pub static PIXEL_COUNT_2K: u32 = 2048 * 1080;
pub static PIXEL_COUNT_1080P: u32 = 1920 * 1080;

pub fn clamp_max_resolution_to_1080p(source: DynamicImage) -> DynamicImage {
    clamp_max_resolution_to_pixel_count(source, PIXEL_COUNT_1080P)
}

pub fn clamp_max_resolution_to_2k(source: DynamicImage) -> DynamicImage {
    clamp_max_resolution_to_pixel_count(source, PIXEL_COUNT_2K)
}

pub fn clamp_max_resolution_to_4k(source: DynamicImage) -> DynamicImage {
    clamp_max_resolution_to_pixel_count(source, PIXEL_COUNT_4K)
}

pub fn clamp_max_resolution_to_8k(source: DynamicImage) -> DynamicImage {
    clamp_max_resolution_to_pixel_count(source, PIXEL_COUNT_8K)
}

pub fn clamp_max_resolution_to_pixel_count(source: DynamicImage, max_pixel_count: u32) -> DynamicImage {
    let compute = |scale: f32| {
        assert!(scale > 0.0);
        let scaled_width = source.width() as f32 * scale;
        let scaled_height = source.height() as f32 * scale;
        assert!(scaled_width > 0.0);
        assert!(scaled_height > 0.0);
        resize_dimensions(
            source.width(),
            source.height(),
            scaled_width as u32,
            scaled_height as u32,
            true
        )
    };

    if (source.width() * source.height() < max_pixel_count) {
        source
    } else {
        let mut done = false;
        let mut rate = 1.0;
        let mut new_resolution: Option<(u32, u32)> = None;
        while (new_resolution.is_none()) {
            rate = rate - 0.01;
            let (width, height) = compute(rate);
            if ((width * height) < max_pixel_count) {
                new_resolution = Some((width, height));
            }
        }
        let (new_width, new_height) = new_resolution.unwrap_or_else(|| panic!());
        let output = source.resize_exact(new_width, new_height, FilterType::Lanczos3);
        output
    }
}


/// Calculates the width and height an image should be resized to.
/// This preserves aspect ratio, and based on the `fill` parameter
/// will either fill the dimensions to fit inside the smaller constraint
/// (will overflow the specified bounds on one axis to preserve
/// aspect ratio), or will shrink so that both dimensions are
/// completely contained with in the given `width` and `height`,
/// with empty space on one axis.
fn resize_dimensions(width: u32, height: u32, nwidth: u32, nheight: u32, fill: bool) -> (u32, u32) {
    let ratio = u64::from(width) * u64::from(nheight);
    let nratio = u64::from(nwidth) * u64::from(height);

    let use_width = if fill {
        nratio > ratio
    } else {
        nratio <= ratio
    };
    let intermediate = if use_width {
        u64::from(height) * u64::from(nwidth) / u64::from(width)
    } else {
        u64::from(width) * u64::from(nheight) / u64::from(height)
    };
    if use_width {
        if intermediate <= u64::from(::std::u32::MAX) {
            (nwidth, intermediate as u32)
        } else {
            (
                (u64::from(nwidth) * u64::from(::std::u32::MAX) / intermediate) as u32,
                ::std::u32::MAX,
            )
        }
    } else if intermediate <= u64::from(::std::u32::MAX) {
        (intermediate as u32, nheight)
    } else {
        (
            ::std::u32::MAX,
            (u64::from(nheight) * u64::from(::std::u32::MAX) / intermediate) as u32,
        )
    }
}
