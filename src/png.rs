use std::io::Read;
use std::io::Write;
use std::process::exit;
use exoquant::optimizer::Optimizer;
use exoquant::*;
use lodepng::Bitmap;
use lodepng::RGBA;


fn load_img(path: &str) -> Result<Bitmap<RGBA<u8>>, String> {
    match lodepng::decode32_file(path) {
        Ok(img) => Ok(img),
        Err(_) => Err(format!("Error: Failed to load PNG '{}'.", path)),
    }
}

pub fn compress() {
    let input_path = "assets/samples/code.png";
    let output_path = "assets/output/code.png";
    let img = load_img(input_path).expect("load input png");
    let ditherer: Box<dyn ditherer::Ditherer> = Box::new(ditherer::None);
    let num_colors: usize = 200;
    let (optimizer, opt_level): (Box<dyn Optimizer>, u32) = (Box::new(optimizer::WeightedKMeans), 3);
    let histogram = img
        .buffer
        .as_ref()
        .iter()
        .map(|c| Color::new(c.r, c.g, c.b, c.a))
        .collect();
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
    let palette = quantizer.colors(&colorspace);
    let palette = optimizer.optimize_palette(&colorspace, &palette, &histogram, 8);
    let mut state = lodepng::State::new();
    for color in &palette {
        unsafe {
            lodepng::ffi::lodepng_palette_add(
                &mut state.info_png().color,
                color.r,
                color.g,
                color.b,
                color.a,
            );
            lodepng::ffi::lodepng_palette_add(
                &mut state.info_raw(),
                color.r,
                color.g,
                color.b,
                color.a,
            );
        }
    }
    state.info_png().color.bitdepth = 8;
    state.info_png().color.colortype = lodepng::ColorType::LCT_PALETTE;
    state.info_raw().bitdepth = 8;
    state.info_raw().colortype = lodepng::ColorType::LCT_PALETTE;
    let remapper = Remapper::new(&palette, &colorspace, &*ditherer);
    let out_data: Vec<_> = remapper
        .remap_iter(
            Box::new(
                img.buffer
                    .as_ref()
                    .iter()
                    .map(|c| Color::new(c.r, c.g, c.b, c.a)),
            ),
            img.width,
        )
        .collect();
    match state.encode_file(output_path, &out_data, img.width, img.height) {
        Ok(_) => (),
        Err(_) => {
            eprintln!("Error: Failed to write PNG '{}'.", output_path);
            exit(1)
        }
    };
}
