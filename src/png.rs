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
    // let ditherer: Box<dyn ditherer::Ditherer> = unimplemented!();
    // let num_colors: usize = unimplemented!();
    // let (optimizer, opt_level): (Box<dyn Optimizer>, u32) = unimplemented!();
    let img = load_img(input_path).expect("load input png");
}
