use std::convert::{AsRef, From};
use std::io::{Read, Write, BufWriter};
use std::process::exit;
use std::iter::FromIterator;
use std::iter::Extend;
use std::path::{Path, PathBuf};
use exoquant::optimizer::Optimizer;
use exoquant::*;
use lodepng::Bitmap;
use lodepng::RGBA;
use image::{DynamicImage, GenericImage, GenericImageView};

use crate::vmaf::Yuv420pImage;


///////////////////////////////////////////////////////////////////////////////
// DATA TYPES
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
pub enum ImageMode {
    Text,
}

///////////////////////////////////////////////////////////////////////////////
// ENCODER
///////////////////////////////////////////////////////////////////////////////

fn encode_indexed(palette: &[Color], image: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut state = lodepng::State::new();
    for color in palette {
        unsafe {
            lodepng::ffi::lodepng_palette_add(
                &mut state.info_png_mut().color,
                color.r,
                color.g,
                color.b,
                color.a,
            );
            lodepng::ffi::lodepng_palette_add(
                &mut state.info_raw_mut(),
                color.r,
                color.g,
                color.b,
                color.a,
            );
        }
    }
    state.info_png_mut().color.set_bitdepth(8);
    state.info_png_mut().color.colortype = lodepng::ColorType::PALETTE;
    state.info_raw_mut().set_bitdepth(8);
    state.info_raw_mut().colortype = lodepng::ColorType::PALETTE;
    state.encode(image, width as usize, height as usize).expect("encode png data")
}

pub fn compress(source: &DynamicImage, mode: ImageMode, num_colors: usize) -> Result<Vec<u8>, String> {
    // CHECKS
    assert!(num_colors <= 256);
    // SETUP
    let (ditherer, optimizer) = match mode {
        ImageMode::Text => {
            // VALUES
            let d = ditherer::None;
            let o = optimizer::WeightedKMeans;
            // DONE
            let d: Box<dyn ditherer::Ditherer> = Box::new(d);
            let o: Box<dyn Optimizer> = Box::new(o);
            (d, o)
        }
    };
    let input_pixels = source
        .pixels()
        .map(|(_, _, px)| Color::new(px.0[0], px.0[1], px.0[2], px.0[3]))
        .collect::<Vec<Color>>();
    let histogram = Histogram::from_iter(input_pixels.clone());
    let colorspace = SimpleColorSpace::default();
    let colorspace = SimpleColorSpace {
        gamma: 0.85,
        dither_gamma: 2.2,
        transparency_scale: 0.01,
        scale: Colorf {
            r: 0.6,
            g: 0.6,
            b: 0.6,
            a: 0.5,
        },
    };
    let mut quantizer = Quantizer::new(&histogram, &colorspace);
    for _ in 0..num_colors {
        quantizer.step();
        quantizer = quantizer.optimize(&*optimizer, 16);
    }
    // PALETTE DATA
    let palette = quantizer.colors(&colorspace);
    let palette = optimizer.optimize_palette(&colorspace, &palette, &histogram, 16);
    let remapper = Remapper::new(&palette, &colorspace, &*ditherer);
    // PIXEL DATA
    let out_data: Vec<u8> = remapper
        .remap_iter(Box::new(input_pixels.into_iter()), source.width() as usize)
        .collect();
    // ENCODE
    let out_file = encode_indexed(
        &palette,
        &out_data,
        source.width(),
        source.height(),
    );
    // DONE
    Ok(out_file)
}


///////////////////////////////////////////////////////////////////////////////
// DEV
///////////////////////////////////////////////////////////////////////////////

pub fn run() {
    // DEV
    let input_path = "assets/samples/code.png";
    let output_path = "assets/output/main.png";
    // LOAD & DECODE
    let img = ::image::open(input_path).expect("load input png");
    // RUN
    let mode = ImageMode::Text;
    let num_colors = 6;
    let out = compress(&img, mode, num_colors).expect("compress png source");
    std::fs::write("assets/output/test.png", &out);
    // VMAF REPORT
    let source1 = Yuv420pImage::from_image(&img);
    let source2 = Yuv420pImage::from_png_image(&out);
    let report = crate::vmaf::report(&source1, &source2);
    println!("vmaf: {}", report);
}