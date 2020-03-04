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

#[derive(Clone)]
pub struct NoisyLayer(DynamicImage);

impl NoisyLayer {
    pub fn new(source: Image<Luma<u32>>) -> Self {
        // INIT PIXEL-TABLE
        let mut pixel_table = HashMap::<u32, Vec<(u32, u32)>>::new();
        for (x, y, px) in source.enumerate_pixels() {
            if let Some(xs) = pixel_table.get_mut(&px.0[0]) {
                xs.push((x, y));
            } else {
                pixel_table.insert(px.0[0], vec![(x, y)]);
            }
        }
        // COMPUTE CENTER POINTS
        let centers = pixel_table
            .into_iter()
            .map(|(px, cs)| {
                let cs_len = cs.len() as u32;
                let sum = cs
                    .into_iter()
                    .fold((0u32, 0u32), |(xs, ys), (x, y)| {
                        (xs + x, ys + y)
                    });
                (sum.0 / cs_len, sum.1 / cs_len)
            })
            .collect::<Vec<_>>();
        let mut output = ::image::GrayImage::from_pixel(
            source.width(),
            source.height(),
            Luma([0])
        );
        for (cx, cy) in centers.into_iter() {
            let px = output.get_pixel_mut(cx, cy);
            px[0] = std::u8::MAX;
        }
        // // INVERT & FILTER
        // let empty_pixel = Luma([std::u8::MAX]);
        // let feature_pixel = Luma([0]);
        // let mut output = ::imageproc::map::map_pixels(&output, |x, y, mut px| {
        //     // NOTHING TO DO
        //     if px.0[0] == 0 {
        //         return empty_pixel;
        //     }
        //     // FILTER
        //     let lookup = |cx: u32, cy: u32| -> Option<Luma<u8>> {
        //         if output.in_bounds(cx, cy) {
        //             Some(*output.get_pixel(cx, cy))
        //         } else {
        //             None
        //         }
        //     };
        //     let north = lookup(x, y + 1);
        //     let south = lookup(x, y - 1);
        //     let east = lookup(x + 1, y);
        //     let west = lookup(x - 1, y);
        //     let neighbords = vec![north, south, east, west];
        //     let connected = neighbords
        //         .into_iter()
        //         .filter_map(|x| x)
        //         .any(|x| x.0[0] != 0);
        //     if connected {
        //         feature_pixel
        //     } else {
        //         empty_pixel
        //     }
        // });
        // imageproc::morphology::open_mut(&mut output, Norm::L1, 2);
        // // INVERT
        // for (x, y, px) in output.enumerate_pixels_mut() {
        //     if px.0[0] == 0 {
        //         *px = Luma([std::u8::MAX]);
        //     } else {
        //         *px = Luma([0]);
        //     }
        // }
        // // MISC
        // imageproc::morphology::dilate_mut(&mut output, Norm::L1, 2);
        // DONE
        NoisyLayer(DynamicImage::ImageLuma8(output))
    }
}

///////////////////////////////////////////////////////////////////////////////
// DENSE LAYER
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct DenseLayer(DynamicImage);

impl DenseLayer {
    pub fn new(source: Image<Luma<u32>>) -> Self {
        // INIT PIXEL-TABLE
        let mut pixel_table = HashMap::<u32, Vec<(u32, u32)>>::new();
        for (x, y, px) in source.enumerate_pixels() {
            if let Some(xs) = pixel_table.get_mut(&px.0[0]) {
                xs.push((x, y));
            } else {
                pixel_table.insert(px.0[0], vec![(x, y)]);
            }
        }
        // FILTER
        let mut output = ::image::GrayImage::from_fn(source.width(), source.height(), |x, y| {
            let px = source.get_pixel(x, y);
            let val = pixel_table.get(&px.0[0]).expect("missing pixel");
            if val.len() <= (2 * 2) {
                Luma([std::u8::MAX])
            } else {
                Luma([0])
            }
        });
        // INVERT
        for (x, y, px) in output.enumerate_pixels_mut() {
            if px.0[0] == 0 {
                *px = Luma([std::u8::MAX]);
            } else {
                *px = Luma([0]);
            }
        }
        // DONE
        DenseLayer(DynamicImage::ImageLuma8(output))
    }
}


///////////////////////////////////////////////////////////////////////////////
// GRADIENT LAYER
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct GradientLayer(DynamicImage);

impl GradientLayer {
    pub fn new(source: &Image<Luma<u8>>) -> Self {
        let output = ::imageproc::edges::canny(source, 10.0, 20.0);
        GradientLayer(DynamicImage::ImageLuma8(output))
    }
}


///////////////////////////////////////////////////////////////////////////////
// EVAL
///////////////////////////////////////////////////////////////////////////////

// pub fn eval(image: &DynamicImage) -> DynamicImage {
//     let image = image.resize_exact(600, 600, FilterType::Lanczos3);
//     let image = image.unsharpen(1.2, 4);
//     let image = crate::color::quant::reduce_palette(&image, 64);
//     let image = image.to_luma();
//     let image = ::imageproc::map::map_pixels(&image, |x, y, mut px| {
//         if px.0[0] == 0 {
//             px.0[0] = 1;
//         }
//         px
//     });
//     let image = imageproc::region_labelling::connected_components(
//         &image,
//         Connectivity::Eight,
//         Luma::black()
//     );
    
//     // let image = palette::set_region(&image, Luma([std::u32::MAX]), |_, count| count > (120 * 120));
    
//     let image = DenseLayer::new(image);
//     image.0
    
//     // DONE
//     // let image = image.to_pretty_rgb_palette();
//     // DynamicImage::ImageRgb8(image)
// }

pub fn eval(source: &DynamicImage) -> DynamicImage {
    let output = source.resize_exact(600, 600, FilterType::Lanczos3);
    // let output = output.unsharpen(1.2, 4);
    // let output = crate::color::quant::reduce_palette(&output, 12);
    // REGION-LABEL
    let output = output.to_luma();
    // let output = ::imageproc::map::map_pixels(&output, |x, y, mut px| {
    //     if px.0[0] == 0 {
    //         px.0[0] = 1;
    //     }
    //     px
    // });
    // let output = imageproc::region_labelling::connected_components(
    //     &output,
    //     Connectivity::Eight,
    //     Luma::black()
    // );
    
    // // LAYERS
    // let output = NoisyLayer::new(output);
    // output.0
    // DONE

    // MAIN-FILTERS
    // let output = palette::set_region(&output, Luma([0]), |_, count| count < (60 * 60));
    // let output = palette::set_region(&output, Luma([0]), |pixel, count| {
    //     if pixel.0[0] == 0 {
    //         false
    //     } else {
    //         count > (200 * 200)
    //     }
    // });
    
    // // GRADIENT
    // let output = GradientLayer::new(&output.to_luma());

    // DONE
    DynamicImage::ImageLuma8(output)
    // let output = output.to_pretty_rgb_palette();
    // DynamicImage::ImageRgb8(output)
}


///////////////////////////////////////////////////////////////////////////////
// MAIN
///////////////////////////////////////////////////////////////////////////////


pub fn run() {
    let output_dir = PathBuf::from("assets/output");
    std::fs::create_dir_all(&output_dir);
    let paths = vec![
        glob::glob("assets/samples/focus/**/*.jpeg"),
        glob::glob("assets/samples/focus/**/*.png"),
    ];
    paths
        .into_iter()
        .flat_map(|x| x.expect("input glob").filter_map(Result::ok))
        .collect::<Vec<_>>()
        .into_par_iter()
        .for_each(|input_path| {
            let mut output_path = input_path
                .file_name()
                .map(|name| output_dir.join(name))
                .expect("init output file name");
            output_path.set_extension("png");
            let src_image = ::image::open(input_path).expect("open source image");
            let out_image = eval(&src_image);
            out_image.save(output_path);
        });
}

