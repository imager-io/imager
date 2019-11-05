// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::collections::*;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use image::{
    GenericImage,
    GenericImageView,
    DynamicImage,
    FilterType,
    Pixel,
    ColorType,
    ImageBuffer,
    ImageFormat,
    GrayImage,
    RgbImage,
    Luma,
    Rgb,
    ConvertBuffer,
};
use imageproc::corners::Fast;
use imageproc::definitions::HasBlack;
use imageproc::region_labelling::{
    connected_components,
    Connectivity
};
use imageproc::distance_transform::Norm;


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Class {
    L0,
    L1,
    L2,
    M1,
    H1,
    H2,
}

impl Class {
    pub fn from_str(x: &str) -> Option<Self> {
        match x.to_lowercase().as_str() {
            "l0" => Some(Class::L0),
            "l1" => Some(Class::L1),
            "l2" => Some(Class::L2),
            "m1" => Some(Class::M1),
            "h1" => Some(Class::H1),
            "h2" => Some(Class::H2),
            _ => None
        }
    }
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Class::L0 => write!(f, "l0"),
            Class::L1 => write!(f, "l1"),
            Class::L2 => write!(f, "l2"),
            Class::M1 => write!(f, "m1"),
            Class::H1 => write!(f, "h1"),
            Class::H2 => write!(f, "h2"),
        }
    }
}



impl std::str::FromStr for Class {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match Class::from_str(input) {
            Some(x) => Ok(x),
            None => Err(String::from("parser failed (invalid)"))
        }
    }
}



///////////////////////////////////////////////////////////////////////////////
// UTILS
///////////////////////////////////////////////////////////////////////////////

pub fn random_color_map(keys: HashSet<u32>) -> HashMap<u32, image::Rgb<u8>> {
    use colourado::{Color, ColorPalette, PaletteType};
    let palette = ColorPalette::new(keys.len() as u32, PaletteType::Random, false);
    let mut output: HashMap<u32, image::Rgb<u8>> = HashMap::new();
    for (key, ix) in keys.iter().zip(0 .. keys.len()) {
        let key = key.clone();
        if key == 0 {
            output.insert(key, image::Rgb([0, 0, 0]));
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
}

///////////////////////////////////////////////////////////////////////////////
// WHITE COUNT
///////////////////////////////////////////////////////////////////////////////

pub fn is_white_dominant(media: &DynamicImage) -> bool {
    // LOAD
    let media = media.resize_exact(700, 700, FilterType::Gaussian);
    let mut gray_media = media.to_luma();
    let mut media = image::imageops::colorops::contrast(&gray_media, 5.0);
    // SETUP
    let mut components = connected_components(&media, Connectivity::Four, Luma::black());
    let color_threshold = 240;
    let mut regions_reg: HashMap<u32, usize> = HashMap::new();
    // INIT
    for (x, y, px) in components.enumerate_pixels_mut() {
        let key = px.0[0];
        let val = media.get_pixel(x, y).0[0];
        if (val < color_threshold) {
            px.0[0] = 0;
        }
        // LOG REGION SIZE
        match regions_reg.get_mut(&key) {
            None => {
                regions_reg.insert(key, 1);
            }
            Some(x) => {
                *x += 1;
            }
        }
    }
    // FILTER
    for (x, y, px) in components.enumerate_pixels_mut() {
        let key = px.0[0];
        if let Some(val) = regions_reg.get(&key).map(|x| x.clone()) {
            if val < 15_000 {
                px.0[0] = 0;
            }
        }
    }
    // SUM
    let mut sum = 0;
    for (_, _, px) in components.enumerate_pixels() {
        let val = px.0[0];
        if val > 0 {
            sum += 1;
        }
    }
    // DEBUG IMAGE
    if false {
        let debug_colors = random_color_map(components.pixels().map(|p| p[0]).map(|x| x).collect());
        let debug_media = ImageBuffer::from_fn(media.width(), media.height(), |x, y| {
            let px_key = components.get_pixel(x, y).channels()[0];
            let color = debug_colors.get(&px_key).expect("missing color entry");
            color.clone()
        });
    }
    // DONE
    sum > 130_100
}


///////////////////////////////////////////////////////////////////////////////
// PROCESS
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct DebugImages {
    pub grayscale: GrayImage,
    pub edges: GrayImage,
    pub regions: RgbImage,
}

#[derive(Clone)]
pub struct Meta {
    pub edges_sum: usize,
    pub regions_sum: usize,
    pub component_count: usize,
    pub white_count: usize,
}



#[derive(Clone)]
pub struct Report {
    pub debug_images: DebugImages,
    pub meta: Meta,
    pub class: Class,
    pub white_backdrop: bool,
}



fn calcuate_class(meta: &Meta) -> Class {
    let mut output = Class::L1;
    if meta.edges_sum >= 110_000 && meta.regions_sum <= 12_000 && meta.component_count < 90 {
        output = Class::H2;
    }
    else if meta.edges_sum >= 70_000 && meta.regions_sum <= 12_000 && meta.component_count < 90 {
        output = Class::H1;
    }
    else if meta.edges_sum >= 60_000 && meta.regions_sum <= 90_000 {
        output = Class::M1;
    }
    else if meta.edges_sum >= 20_000 && meta.regions_sum <= 200_000 {
        output = Class::L2;
    }
    else if meta.component_count > 20 {
        output = Class::L2;
    }
    else if meta.component_count <= 6 {
        output = Class::L0;
    }
    output
}

pub fn grayscale_segmentation(media: &DynamicImage) -> GrayImage {
    let mut grayscale_media = media.to_luma();
    unimplemented!()
}

pub fn report(media: &DynamicImage) -> Report {
    // MISC
    let white_dominant = is_white_dominant(&media);
    // PRE-PROCESS IMAGE
    let media = media.resize_exact(700, 700, FilterType::Gaussian);
    // DOMINANT COLORS
    let mut grayscale_media = media.to_luma();
    for (_, _, px) in grayscale_media.enumerate_pixels_mut() {
        // px.0[0] = 200;
    }
    let white_count = grayscale_media
        .pixels()
        .fold(0,|acc, x| {
            let value = x.0[0];
            if value >= 220 {
                acc + 1
            } else {
                acc
            }
        });
    // EDGES
    let edges_media: GrayImage = imageproc::edges::canny(&media.to_luma(), 10.0, 20.0);
    let edges_sum: usize = edges_media
        .pixels()
        .fold(0, |acc, px| {
            match px.channels()[0] {
                255 => {acc + 1}
                _ => {acc}
            }
        });
    // REGIONS
    let mut regions_media = imageproc::morphology::close(&edges_media, Norm::LInf, 6);
    for (_, _, px) in regions_media.enumerate_pixels_mut() {
        let new_px = match px.0[0] {
            255 => 0,
            _ => 255,
        };
        *px = Luma([new_px]);
    }
    let components_background_color = Luma::black();
    let mut components = connected_components(&regions_media, Connectivity::Eight, components_background_color);
    let component_count = components.pixels().map(|p| p[0]).max().expect("components");
    let mut region_sums: HashMap<u32, usize> = HashMap::new();
    for (_, _, px) in components.enumerate_pixels() {
        let px = px.0[0];
        if px != 0 {
            match region_sums.get_mut(&px) {
                None => {
                    region_sums.insert(px, 0);
                }
                Some(v) => {
                    *v = v.clone() + 1;
                }
            }   
        }
    }
    let regions_sum = region_sums.values().map(|x| x).max().map(|x| x.clone()).unwrap_or(0);
    let debug_colors = random_color_map(components.pixels().map(|p| p[0]).map(|x| x).collect());
    let regions_media = ImageBuffer::from_fn(regions_media.width(), regions_media.height(), |x, y| {
        let px_key = components.get_pixel(x, y).channels()[0];
        let color = debug_colors.get(&px_key).expect("missing color entry");
        color.clone()
    });
    // DEBUG IMAGES
    let debug_images = DebugImages {
        grayscale: grayscale_media,
        edges: edges_media,
        regions: regions_media,
    };
    // META
    let meta = Meta {
        edges_sum: edges_sum,
        regions_sum,
        component_count: component_count as usize,
        white_count: white_count,
    };
    // CLASS
    let class = calcuate_class(&meta);
    // DONE
    Report {
        meta,
        class,
        debug_images,
        white_backdrop: white_dominant,
    }
}
