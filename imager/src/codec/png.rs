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

use crate::data::{VideoBuffer, Yuv420P};
use crate::vmaf;


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
    let dev = false;
    let colorspace = SimpleColorSpace::default();
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

pub fn basic_optimize(source: &DynamicImage) -> Vec<u8> {
    let vmaf_source = VideoBuffer::from_image(&source).expect("to VideoBuffer");
    let run = |num_colors: usize| {
        let mode = ImageMode::Text;
        let compressed = compress(&source, mode, num_colors).expect("compress png source");
        let report = {
            let vmaf_derivative = VideoBuffer::from_png(&compressed).expect("to VideoBuffer");
            vmaf::get_report(&vmaf_source, &vmaf_derivative)
        };
        // println!("vmaf: {}", report);
        (compressed, report)
    };
    let fallback = || {
        let num_colors = 255;
        let mode = ImageMode::Text;
        compress(&source, mode, num_colors).expect("compress png source")
    };
    // RUN
    for num_colors in 1..256 {
        // println!("num_colors: {}", num_colors);
        let (compressed, report) = run(num_colors);
        if report >= 90.0 || num_colors <= 5 {
            return compressed;
        }
    }
    // OR RUN FALLBACK
    fallback()
}


///////////////////////////////////////////////////////////////////////////////
// DEV
///////////////////////////////////////////////////////////////////////////////

pub fn run() {
    // DEV
    let input_path = "assets/samples/code.png";
    let output_path = "assets/output/test.png";
    // LOAD & DECODE
    let img = ::image::open(input_path).expect("load input png");
    let out = basic_optimize(&img);
    std::fs::write(output_path, &out);
}