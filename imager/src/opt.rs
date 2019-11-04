use std::path::PathBuf;
use std::sync::*;
use serde::{Serialize, Deserialize};
use image::{DynamicImage, GenericImage, FilterType, GenericImageView};
use rayon::prelude::*;
use itertools::Itertools;
use either::{Either, Either::*};
use crate::tool::classifier::{self, Class};
use crate::tool::mozjpeg::DecodedImage;
use crate::data::{OutputSize, Resolution};

#[derive(Clone)]
pub struct Source {
    pub class: classifier::Report,
    pub source: DecodedImage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutMeta {
    pub start_q: u8,
    pub end_q: u8,
    pub passed: bool,
    pub class: Class,
}

impl Source {
    pub fn new(buffer: &[u8], resize: OutputSize) -> Result<Self, String> {
        let format = ::image::guess_format(buffer)
            .map_err(|x| format!("{}", x))?;
        let image = ::image::load_from_memory(buffer)
            .map_err(|x| format!("{}", x))?;
        let class = classifier::report(&image);
        let image = match resize {
            OutputSize::Px(Resolution{width, height}) if image.dimensions() > (width, height) => {
                image.resize(width, height, ::image::FilterType::Lanczos3)
            }
            OutputSize::Full | _ => image
        };
        let decoded = DecodedImage::from_image(&image)?;
        Ok(Source {
            class: class,
            source: decoded,
        })
    }
    pub fn open(path: &PathBuf, resize: OutputSize) -> Result<Self, String> {
        let source = std::fs::read(path).expect("read soruce file");
        Source::new(&source, resize)
    }
    fn terminate(&self, score: f64) -> bool {
        let mut threshold;
        let (width, height) = (self.source.width, self.source.height);
        let is_small = {
            width < 400 || height < 400
        };
        match self.class.class {
            Class::L0 if self.class.white_backdrop => {
                threshold = 96.0;
            }
            Class::L1 if self.class.white_backdrop => {
                threshold = 94.0;
            }
            Class::L2 if self.class.white_backdrop => {
                threshold = 93.0;
            }
            Class::L0 => {
                threshold = 99.0;
            }
            Class::L1 => {
                threshold = 98.0;
            }
            Class::L2 => {
                threshold = 96.0;
            }
            Class::M1 => {
                threshold = 92.0;
            }
            Class::H1 if !is_small => {
                threshold = 84.0;
            }
            Class::H2 if !is_small => {
                threshold = 76.0;
            }
            Class::H1 | Class::H2 => {
                assert!(is_small);
                threshold = 88.0;
            }
        }
        if (score >= threshold) {
            true
        } else {
            false
        }
    }
    fn find_starting_position(&self) -> Option<u8> {
        let reduce_starting_values = |qs: Vec<u8>| -> Option<u8> {
            let xs = qs
                .into_par_iter()
                .map(|q| -> (u8, bool) {
                    (q, self.run_instance(q).1)
                })
                .collect::<Vec<_>>()
                .into_iter()
                .sorted_by(|a, b| u8::cmp(&a.0, &b.0))
                .collect::<Vec<_>>();
            let mut result: Option<u8> = None;
            for (q, passed) in xs.into_iter().rev() {
                if !passed {
                    result = Some(q);
                    break;
                }
            }
            result
        };
        let bad_fallback = || reduce_starting_values(vec![
            90,
            80,
            70,
            60,
            50,
            40,
        ]);
        match self.class.class {
            Class::H2 => {
                reduce_starting_values(vec![
                    30,
                    20,
                    10,
                ])
            }
            Class::H1 => {
                reduce_starting_values(vec![
                    60,
                    50,
                    40,
                    30,
                    20,
                ])
            }
            Class::M1 => {
                reduce_starting_values(vec![
                    80,
                    70,
                    60,
                    50,
                    40,
                    30,
                ])
            }
            _ => bad_fallback()
        }
    }
    fn run_instance(&self, q: u8) -> (Vec<u8>, bool) {
        let compressed = crate::tool::mozjpeg::encode(&self.source, q);
        let report = {
            let reconstructed = DecodedImage::load_from_memory(&compressed)
                .expect("search failed - reconstructed");
            crate::tool::vmaf::report(&self.source, &reconstructed)
        };
        if self.terminate(report) {
            (compressed, true)
        } else {
            (compressed, false)
        }
    }
    pub fn run_search(&self) -> (Vec<u8>, OutMeta) {
        let mut passed_output: Option<(Vec<u8>, OutMeta)> = None;
        let starting_q = self.find_starting_position().unwrap_or(0);
        for q in starting_q..=98 {
            let (compressed, done) = self.run_instance(q);
            if done {
                let out_meta = OutMeta {
                    start_q: starting_q,
                    end_q: q,
                    passed: true,
                    class: self.class.class.clone(),
                };
                passed_output = Some((compressed, out_meta));
                break;
            }
        }
        match passed_output {
            // BAD
            None => {
                let fallback_q = 98;
                let payload = crate::tool::mozjpeg::encode(&self.source, fallback_q);
                let out_meta = OutMeta {
                    start_q: starting_q,
                    end_q: fallback_q,
                    passed: false,
                    class: self.class.class.clone(),
                };
                (payload, out_meta)
            }
            // BAD
            Some((_, bad_m)) if bad_m.start_q == 0 && bad_m.end_q == 0 => {
                let fallback_q = 75;
                let payload = crate::tool::mozjpeg::encode(&self.source, fallback_q);
                let out_meta = OutMeta {
                    start_q: starting_q,
                    end_q: fallback_q,
                    passed: false,
                    class: self.class.class.clone(),
                };
                (payload, out_meta)
            }
            // GOOD
            Some(x) => x
        }
    }
}




