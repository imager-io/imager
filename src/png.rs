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


pub fn encode_indexed(palette: &[Color], image: &[u8], width: u32, height: u32) -> Vec<u8> {
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


pub fn compress() {
    // INPUT ARGUMENTS
    let num_colors: usize = 256;
    assert!(num_colors <= 256);
    let opt_level = 3;
    // DEV
    let input_path = "assets/samples/code.png";
    let output_path = "assets/output/main.png";
    // LOAD & DECODE
    let img = ::image::open(input_path).expect("load input png");
    // SETUP
    let ditherer: Box<dyn ditherer::Ditherer> = Box::new(ditherer::None);
    let optimizer: Box<dyn Optimizer> = Box::new(optimizer::WeightedKMeans);
    let input_pixels = img
        .pixels()
        .map(|(_, _, px)| Color::new(px.0[0], px.0[1], px.0[2], px.0[3]))
        .collect::<Vec<Color>>();
    let histogram = Histogram::from_iter(input_pixels.clone());
    let colorspace = SimpleColorSpace::default();
    let mut quantizer = Quantizer::new(&histogram, &colorspace);
    let kmeans_step = {
        if opt_level < 2 {
            num_colors
        } else if opt_level == 2 {
            (num_colors as f64).sqrt().round() as usize
        } else {
            1
        }
    };
    while quantizer.num_colors() < num_colors {
        quantizer.step();
        if quantizer.num_colors() % kmeans_step == 0 {
            quantizer = quantizer.optimize(&*optimizer, 4);
        }
    }
    // PALETTE DATA
    let palette = quantizer.colors(&colorspace);
    let palette = optimizer.optimize_palette(&colorspace, &palette, &histogram, 8);
    let remapper = Remapper::new(&palette, &colorspace, &*ditherer);
    // PIXEL DATA
    let out_data: Vec<u8> = remapper
        .remap_iter(Box::new(input_pixels.into_iter()), img.width() as usize)
        .collect();
    // ENCODE
    let out_file = encode_indexed(
        &palette,
        &out_data,
        img.width(),
        img.height(),
    );
    // DONE
    std::fs::write("test.png", &out_file);
}

// pub fn encode(data: Vec<u8>, width: u32, height: u32) {
//     // SETUP
//     // let mut buffer = BufWriter::new(Vec::<u8>::new());
//     let path = Path::new("image.png");
//     let file = std::fs::File::create(path).unwrap();
//     let ref mut buffer = BufWriter::new(file);
//     // RUN
//     {
//         let mut encoder = ::png::Encoder::new(buffer, width, height);
//         encoder.set_depth(::png::BitDepth::Eight);
//         // encoder.set_compression(png::Compression::Best);
//         // encoder.set_filter(png::FilterType::Paeth);
//         encoder.set_color(::png::ColorType::Indexed);
//         // ENCODE
//         let mut writer = encoder
//             .write_header()
//             .expect("write png header");
//         writer
//             .write_image_data(&data)
//             .expect("write/encode png data");
//     }
//     // DONE
//     // buffer
//     //     .into_inner()
//     //     .expect("extract encoded png buffer")
// }


///////////////////////////////////////////////////////////////////////////////
// DEV
///////////////////////////////////////////////////////////////////////////////

pub fn run() {
    compress();
}