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
use image::imageops::{FilterType};
use image::{DynamicImage, GenericImage, GenericImageView};

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

pub fn compress(source: &DynamicImage, num_colors: usize) -> Result<Vec<u8>, String> {
    // CHECKS
    assert!(num_colors <= 256);
    // SETUP
    let (ditherer, optimizer) = {
        // VALUES
        let d = ditherer::None;
        let o = optimizer::WeightedKMeans;
        // DONE
        let d: Box<dyn ditherer::Ditherer> = Box::new(d);
        let o: Box<dyn Optimizer> = Box::new(o);
        (d, o)
    };
    let input_pixels = source
        .pixels()
        .map(|(_, _, px)| Color::new(px.0[0], px.0[1], px.0[2], px.0[3]))
        .collect::<Vec<Color>>();
    let histogram = Histogram::from_iter(input_pixels.clone());
    let colorspace = SimpleColorSpace::default();
    let mut quantizer = Quantizer::new(&histogram, &colorspace);
    let kmeans_step = (num_colors as f64).sqrt().round() as usize;
    for _ in 0..num_colors {
        quantizer.step();
        if quantizer.num_colors() % kmeans_step == 0 {
            quantizer = quantizer.optimize(&*optimizer, 2);
        }
        // quantizer = quantizer.optimize(&*optimizer, 16);
    }
    // PALETTE DATA
    let palette = quantizer.colors(&colorspace);
    let palette = optimizer.optimize_palette(&colorspace, &palette, &histogram, 8);
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


pub fn reduce_palette(source: &DynamicImage, num_colors: usize) -> DynamicImage {
    let result = compress(source, num_colors).expect("failed to reduce color palette");
    let result = ::image::load_from_memory_with_format(&result, ::image::ImageFormat::Png).expect("decode png");
    result
}