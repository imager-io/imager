use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use rayon::prelude::*;
use image::{DynamicImage, GenericImage, GenericImageView};
use imager_av::classifier::{self, Class};
use imager_av::vmaf;
use crate::encode::lossy::{encode};

#[derive(Clone, Serialize, Deserialize)]
pub struct OutMeta {
    pub class: Class,
    pub score: f64,
    pub end_q: u32,
    pub passed: bool,
    pub input_path: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
}

pub fn opt(source: &DynamicImage) -> (Vec<u8>, OutMeta) {
    let class = classifier::report(source);
    let vmaf_source = vmaf::Yuv420pImage::from_image(source);
    let run = |q: f32| -> (Vec<u8>, f64) {
        let compressed = encode(source, q);
        let score = {
            let vmaf_derivative = crate::decode::decode(&compressed);
            let vmaf_derivative = vmaf::Yuv420pImage::from_image(&vmaf_derivative);
            vmaf::report(&vmaf_source, &vmaf_derivative)
        };
        (compressed, score)
    };
    let fallback = |end_q, score| {
        let compressed = encode(source, 100.0);
        let meta = OutMeta {
            class: class.class.clone(),
            score,
            end_q,
            passed: false,
            input_path: None,
            output_path: None,
        };
        (compressed, meta)
    };
    let terminate = |score: f64| {
        let (width, height) = source.dimensions();
        let is_small = {
            width < 600 || height < 600
        };
        let mut threshold;
        match class.class {
            Class::L0 | Class::L1 | Class::L2 if is_small => {
                threshold = 99.0;
            }
            Class::L0 | Class::L1 | Class::L2 => {
                threshold = 95.0;
            }
            Class::M1 => {
                if is_small {
                    threshold = 98.0;
                } else {
                    threshold = 90.0;
                }
            }
            Class::H1 | Class::H2 if is_small => {
                threshold = 88.0;
            }
            Class::H1 => {
                threshold = 75.0;
            }
            Class::H2 => {
                threshold = 65.0;
            }
        }
        score >= threshold
    };
    // SEARCH
    let start_q = match class.class {
        Class::H1 | Class::H2 => 1,
        _ => 10,
    };
    let mut last_q = None;
    let mut last_score = None;
    for q in start_q..100 {
        let (compressed, score) = run(q as f32);
        last_q = Some(q);
        last_score = Some(score);
        if terminate(score) {
            let meta = OutMeta {
                class: class.class.clone(),
                score,
                end_q: q,
                passed: true,
                input_path: None,
                output_path: None,
            };
            return (compressed, meta);
        }
    }
    // FALLBACK
    let last_q = last_q.expect("should run at least once");
    let last_score = last_score.expect("should run at least once");
    fallback(last_q, last_score)
}

