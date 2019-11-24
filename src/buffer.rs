use std::convert::AsRef;
use std::path::{Path, PathBuf};
use image::DynamicImage;

#[derive(Clone, Debug)]
pub struct Rgba {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

// impl Rgba {
//     pub fn from_image(media: &DynamicImage) {
//         // let media = 
//     }
// }