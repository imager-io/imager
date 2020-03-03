use std::convert::AsRef;
use std::path::{PathBuf, Path};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use rand::prelude::*;
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


pub fn quantizer(image: &DynamicImage) -> DynamicImage {
    let image = image.resize_exact(600, 600, ::image::FilterType::Lanczos3);
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

pub fn preprocess(image: &DynamicImage, class: Class) -> DynamicImage {
    let image = quantizer(&image);
    image
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



pub fn train() {
    let load_group = |pattern: &str| -> Vec<DynamicImage> {
        glob::glob(pattern)
            .expect("input glob")
            .filter_map(Result::ok)
            .map(|input_path| {
                let image = ::image::open(input_path).expect("load input image");
                let image = quantizer(&image);
                image
            })
            .collect::<Vec<_>>()
    };
    let dataset = vec![
        (
            load_group("assets/samples/high/**/*.jpeg"),
            Class::Hi,
        ),
        (
            load_group("assets/samples/low/**/*.jpeg"),
            Class::Lo,
        ),
        (
            load_group("assets/samples/extra-low/**/*.jpeg"),
            Class::ExLo,
        ),
        (
            load_group("assets/samples/high-basic/**/*.jpeg"),
            Class::HiBasic,
        ),
    ];
    for (media, class) in dataset {

    }
}


///////////////////////////////////////////////////////////////////////////////
// MAIN
///////////////////////////////////////////////////////////////////////////////


// pub fn process(input_path: &str, output_path: &str, class: Class) {
//     // RUN
//     let image = ::image::open(input_path).expect("open source image");
//     let debug_image = preprocess(&image, class);

//     // FILE PATHS
//     let base_path = PathBuf::from(output_path)
//         .parent()
//         .expect("parent path")
//         .to_owned();
//     std::fs::create_dir_all(&base_path);
    
//     let debug_output_path = {
//         let mut path = PathBuf::from(output_path)
//             .file_name()
//             .map(|x| PathBuf::from(x.clone()))
//             .expect("file name");
//         path.set_extension("debug.jpeg");
//         base_path.join(path)
//     };

//     // SAVE
//     // image.save(output_path);
//     debug_image.save(debug_output_path);
//     // let compressed = unsafe {
//     //     crate::codec::jpeg::encode(&image, 4)
//     // };
//     // std::fs::write(debug_output_path, compressed);
// }

pub fn run() {
    // let output_path = PathBuf::from("assets/output");
    // let paths = glob::glob("assets/samples/focus/**/*.jpeg")
    //     .expect("input glob")
    //     .filter_map(Result::ok)
    //     .map(|input_path| {
    //         let output_path = input_path
    //             .file_name()
    //             .map(|name| {
    //                 output_path.join(name)
    //             })
    //             .expect("init output file name");
    //         println!("path: {:?}", output_path);
    //     });
}

