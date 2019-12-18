use image::{DynamicImage, GenericImage, GenericImageView, ImageFormat};
use either::{Either, Either::*};

use crate::codec::jpeg;
use crate::codec::png;
use crate::codec::webp;

pub struct Opt {
    source: DynamicImage,
    source_format: ImageFormat,
    output_format: ImageFormat,
}

impl Opt {
    pub fn new(source: &[u8]) -> Result<Self, ()> {
        let source_format = ::image::guess_format(source).map_err(drop)?;
        match source_format {
            ImageFormat::WEBP => {
                let source = webp::decode::decode(source);
                Ok(Opt {
                    output_format: source_format,
                    source,
                    source_format,
                })
            }
            _ => {
                let source = ::image::load_from_memory_with_format(
                    source,
                    source_format,
                )
                .map_err(drop)?;
                Ok(Opt {
                    output_format: source_format,
                    source,
                    source_format,
                })
            }
        }
    }
    pub fn set_output_format(&mut self, output_format: ImageFormat) {
        self.output_format = output_format;
    }
    pub fn run(self) -> Result<Vec<u8>, ()> {
        match self.output_format {
            ImageFormat::WEBP => {
                Ok(webp::opt::opt(&self.source).0)
            }
            ImageFormat::JPEG => {
                Ok(jpeg::OptContext::from_image(self.source.clone()).run_search().0)
            }
            ImageFormat::PNG => {
                Ok(png::basic_optimize(&self.source))
            }
            _ => unimplemented!()
        }
    }
}

