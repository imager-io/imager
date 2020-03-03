use std::convert::AsRef;
use std::path::{PathBuf, Path};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use rand::prelude::*;
use image::imageops::FilterType;
use image::{GenericImage, GenericImageView, ImageBuffer, DynamicImage};
use image::{Luma, Rgb, Pixel};
use imageproc::region_labelling::{
    connected_components,
    Connectivity
};
use imageproc::definitions::Image;
use imageproc::distance_transform::Norm;
use imageproc::definitions::HasBlack;
use image::GrayImage;
use rayon::prelude::*;
use serde::{Serialize, Deserialize};

use crate::color::palette::{self, ToPrettyRgbPalette};

///////////////////////////////////////////////////////////////////////////////
// NOISY LAYER
///////////////////////////////////////////////////////////////////////////////

pub struct NoisyLayer(DynamicImage);

impl NoisyLayer {
    pub fn new(input: DynamicImage) -> Self {
        unimplemented!()
    }
}


///////////////////////////////////////////////////////////////////////////////
// PASSES
///////////////////////////////////////////////////////////////////////////////


pub fn quantizer(image: &DynamicImage) -> DynamicImage {
    let image = image.resize_exact(600, 600, FilterType::Lanczos3);
    let image = image.unsharpen(1.2, 4);
    let image = crate::color::quant::reduce_palette(&image, 64);
    let image = image.to_luma();
    let image = ::imageproc::map::map_pixels(&image, |x, y, mut px| {
        if px.0[0] == 0 {
            px.0[0] = 1;
        }
        px
    });
    let image = imageproc::region_labelling::connected_components(
        &image,
        Connectivity::Eight,
        Luma::black()
    );
    let image = palette::set_region(&image, Luma([std::u32::MAX]), |_, count| count > (120 * 120));
    
    // DONE
    let image = image.to_pretty_rgb_palette();
    DynamicImage::ImageRgb8(image)
}


///////////////////////////////////////////////////////////////////////////////
// CLASSIFY
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Class {
    HiBasic,
    Hi,
    ExLo,
    Lo,
}

impl Class {
    pub fn all_variants() -> Vec<Self> {
        vec![
            Class::HiBasic,
            Class::Hi,
            Class::ExLo,
            Class::Lo,
        ]
    }
    pub fn id(&self) -> u8 {
        match self {
            Class::HiBasic => 0,
            Class::Hi => 1,
            Class::ExLo => 2,
            Class::Lo => 3,
        }
    }
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Class::HiBasic => "hi-basic",
            Class::Hi => "hi",
            Class::ExLo => "ex-lo",
            Class::Lo => "lo",
        };
        write!(f, "{}", value)
    }
}


///////////////////////////////////////////////////////////////////////////////
// MAIN
///////////////////////////////////////////////////////////////////////////////


pub fn run() {
    let output_dir = PathBuf::from("assets/output");
    std::fs::create_dir_all(&output_dir);
    let paths = glob::glob("assets/samples/focus/**/*.jpeg")
        .expect("input glob")
        .filter_map(Result::ok)
        .collect::<Vec<_>>()
        .into_par_iter()
        .for_each(|input_path| {
            let mut output_path = input_path
                .file_name()
                .map(|name| output_dir.join(name))
                .expect("init output file name");
            output_path.set_extension("jpeg");
            let src_image = ::image::open(input_path).expect("open source image");
            let out_image = quantizer(&src_image);
            out_image.save(output_path);
        });
}

