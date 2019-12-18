use std::path::PathBuf;
use itertools::Itertools;
use serde::{Serialize, Deserialize};
use rayon::prelude::*;
use image::{DynamicImage, GenericImage, GenericImageView};
use crate::data::{VideoBuffer, Yuv420P};
use crate::classifier::{self, Class};
use crate::vmaf;
use crate::codec::webp::encode::lossy::{encode};

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    let vmaf_source = VideoBuffer::from_image(source)
        .expect("image to yuv frame");
    let run = |q: f32| -> (Vec<u8>, f64) {
        let compressed = encode(source, q);
        let score = {
            let vmaf_derivative = crate::codec::webp::decode::decode(&compressed);
            let vmaf_derivative = VideoBuffer::from_image(&vmaf_derivative)
                .expect("image to yuv frame");
            vmaf::get_report(&vmaf_source, &vmaf_derivative)
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
            (width * height) < (600 * 600)
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
                threshold = 70.0;
            }
            Class::H1 => {
                threshold = 60.0;
            }
            Class::H2 => {
                threshold = 55.0;
            }
        }
        score >= threshold
    };
    // SEARCH
    let start_q = {
        let reduce_starting_values = |qs: Vec<u8>| -> Option<u8> {
            let mut last_q = 0;
            for q in qs {
                let vmaf_score = run(q as f32).1;
                let passed = terminate(vmaf_score);
                if passed && q <= 10 {
                    return Some(0);
                }
                if passed {
                    return Some(last_q);
                }
                last_q = q;
            }
            None
        };
        let bad_fallback_low_range = || reduce_starting_values(vec![
            10,
            35,
            65,
            75,
            85,
        ]);
        let bad_fallback = || reduce_starting_values(vec![
            0,
            10,
            20,
            30,
            40,
            50,
            60,
            70,
            90,
        ]);
        // TODO:
        match class.class {
            Class::L0 | Class::L1 | Class::L2 => {
                bad_fallback_low_range()
            }
            _ => bad_fallback()
        }
    };
    let start_q = start_q.unwrap_or(1) as u32;
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