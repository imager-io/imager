mod colourado;

use image::{GenericImage, GenericImageView, ImageBuffer};
use image::{Luma, Pixel, Rgb};
use rand::prelude::*;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;

#[doc(hidden)]
macro_rules! random_color_map {
    ($image:expr, $max_value:expr) => {{
        use colourado::{ColorPalette, PaletteType};
        let keys: Vec<_> = $image.pixels().map(|p| p[0]).collect();
        let palette = ColorPalette::new(keys.len() as u32, PaletteType::Random, false);
        let mut output: HashMap<_, image::Rgb<u8>> = HashMap::new();
        for (key, ix) in keys.iter().zip(0..keys.len()) {
            let key = key.clone();
            if key == 0 {
                output.insert(key, image::Rgb([0, 0, 0]));
            } else if key == $max_value {
                output.insert(key, image::Rgb([std::u8::MAX, std::u8::MAX, std::u8::MAX]));
            } else {
                fn convert(x: f32) -> u8 {
                    (x * 255.0) as u8
                }
                let red = convert(palette.colors[ix].red);
                let green = convert(palette.colors[ix].green);
                let blue = convert(palette.colors[ix].blue);

                output.insert(key, image::Rgb([red, green, blue]));
            }
        }
        output
    }};
}

#[doc(hidden)]
macro_rules! to_pretty_rgb_palette {
    ($image:expr, $max_value:expr) => {{
        let colors = random_color_map!($image, $max_value);
        let media = ImageBuffer::from_fn($image.width(), $image.height(), |x, y| {
            let px_key = $image.get_pixel(x, y).channels()[0];
            let color = colors.get(&px_key).expect("missing color entry");
            color.clone()
        });
        media
    }};
}

pub trait ToPrettyRgbPalette {
    fn to_pretty_rgb_palette(&self) -> image::RgbImage;
}

impl ToPrettyRgbPalette for imageproc::definitions::Image<Luma<usize>> {
    fn to_pretty_rgb_palette(&self) -> image::RgbImage {
        to_pretty_rgb_palette!(self, std::usize::MAX)
    }
}
impl ToPrettyRgbPalette for imageproc::definitions::Image<Luma<u8>> {
    fn to_pretty_rgb_palette(&self) -> image::RgbImage {
        to_pretty_rgb_palette!(self, std::u8::MAX)
    }
}
impl ToPrettyRgbPalette for imageproc::definitions::Image<Luma<u16>> {
    fn to_pretty_rgb_palette(&self) -> image::RgbImage {
        to_pretty_rgb_palette!(self, std::u16::MAX)
    }
}
impl ToPrettyRgbPalette for imageproc::definitions::Image<Luma<u32>> {
    fn to_pretty_rgb_palette(&self) -> image::RgbImage {
        to_pretty_rgb_palette!(self, std::u32::MAX)
    }
}
impl ToPrettyRgbPalette for imageproc::definitions::Image<Luma<u64>> {
    fn to_pretty_rgb_palette(&self) -> image::RgbImage {
        to_pretty_rgb_palette!(self, std::u64::MAX)
    }
}
impl ToPrettyRgbPalette for imageproc::definitions::Image<Luma<isize>> {
    fn to_pretty_rgb_palette(&self) -> image::RgbImage {
        to_pretty_rgb_palette!(self, std::isize::MAX)
    }
}
impl ToPrettyRgbPalette for imageproc::definitions::Image<Luma<i8>> {
    fn to_pretty_rgb_palette(&self) -> image::RgbImage {
        to_pretty_rgb_palette!(self, std::i8::MAX)
    }
}
impl ToPrettyRgbPalette for imageproc::definitions::Image<Luma<i16>> {
    fn to_pretty_rgb_palette(&self) -> image::RgbImage {
        to_pretty_rgb_palette!(self, std::i16::MAX)
    }
}
impl ToPrettyRgbPalette for imageproc::definitions::Image<Luma<i32>> {
    fn to_pretty_rgb_palette(&self) -> image::RgbImage {
        to_pretty_rgb_palette!(self, std::i32::MAX)
    }
}
impl ToPrettyRgbPalette for imageproc::definitions::Image<Luma<i64>> {
    fn to_pretty_rgb_palette(&self) -> image::RgbImage {
        to_pretty_rgb_palette!(self, std::i64::MAX)
    }
}

///////////////////////////////////////////////////////////////////////////////
// MISC UTILS
///////////////////////////////////////////////////////////////////////////////

pub fn filter_rgb_regions(image: &::image::RgbImage, min_occurrence: usize) -> ::image::RgbImage {
    let mut image = image.clone();
    filter_rgb_regions_mut(&mut image, min_occurrence);
    image
}

pub fn filter_rgb_regions_mut(image: &mut ::image::RgbImage, min_occurrence: usize) {
    use imageproc::definitions::HasBlack;

    // INIT COUNTER
    let mut counter: HashMap<Rgb<u8>, usize> = HashMap::new();
    for px in image.pixels() {
        match counter.get_mut(&px) {
            Some(mut x) => {
                *x = *x + 1;
            }
            None => {
                counter.insert(px.clone(), 0);
            }
        }
    }

    // FILTER PIXELS
    for px in image.pixels_mut() {
        if let Some(count) = counter.get_mut(px) {
            if *count < min_occurrence {
                *px = Rgb::black();
            }
        }
    }
}

pub fn filter_luma_u32_regions(
    image: &::imageproc::definitions::Image<Luma<u32>>,
    min_occurrence: usize,
) -> ::imageproc::definitions::Image<Luma<u32>> {
    let mut image = image.clone();
    filter_luma_u32_regions_mut(&mut image, min_occurrence);
    image
}

pub fn filter_luma_u32_regions_mut(
    image: &mut ::imageproc::definitions::Image<Luma<u32>>,
    min_occurrence: usize,
) {
    use imageproc::definitions::HasBlack;

    // INIT COUNTER
    let mut counter: HashMap<Luma<u32>, usize> = HashMap::new();
    for px in image.pixels() {
        match counter.get_mut(&px) {
            Some(mut x) => {
                *x = *x + 1;
            }
            None => {
                counter.insert(px.clone(), 0);
            }
        }
    }

    // FILTER PIXELS
    for px in image.pixels_mut() {
        if let Some(count) = counter.get_mut(px) {
            if *count < min_occurrence {
                *px = Luma([0]);
            }
        }
    }
}

pub fn remove_larger_luma_u32_regions(
    image: &::imageproc::definitions::Image<Luma<u32>>,
    max_occurrence: usize,
) -> ::imageproc::definitions::Image<Luma<u32>> {
    let mut image = image.clone();
    remove_larger_luma_u32_regions_mut(&mut image, max_occurrence);
    image
}

pub fn remove_larger_luma_u32_regions_mut(
    image: &mut ::imageproc::definitions::Image<Luma<u32>>,
    max_occurrence: usize,
) {
    use imageproc::definitions::HasBlack;

    // INIT COUNTER
    let mut counter: HashMap<Luma<u32>, usize> = HashMap::new();
    for px in image.pixels() {
        match counter.get_mut(&px) {
            Some(mut x) => {
                *x = *x + 1;
            }
            None => {
                counter.insert(px.clone(), 0);
            }
        }
    }

    // FILTER PIXELS
    for px in image.pixels_mut() {
        if let Some(count) = counter.get_mut(px) {
            if *count > max_occurrence {
                *px = Luma([0]);
            }
        }
    }
}

pub fn set_region(
    image: &::imageproc::definitions::Image<Luma<u32>>,
    new_value: Luma<u32>,
    pred: impl Fn(&Luma<u32>, usize) -> bool,
) -> ::imageproc::definitions::Image<Luma<u32>> {
    let mut image = image.clone();

    // INIT COUNTER
    let mut counter: HashMap<Luma<u32>, usize> = HashMap::new();
    for px in image.pixels() {
        match counter.get_mut(&px) {
            Some(mut x) => {
                *x = *x + 1;
            }
            None => {
                counter.insert(px.clone(), 0);
            }
        }
    }

    // FILTER PIXELS
    for px in image.pixels_mut() {
        if let Some(count) = counter.get_mut(px) {
            if pred(px, *count) {
                *px = new_value.clone();
            }
        }
    }

    // DONE
    image
}
