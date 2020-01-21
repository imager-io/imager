use std::path::PathBuf;
use prophet::Activation::Tanh;
use prophet::samples;
use rayon::prelude::*;
use crate::process::Class;

pub fn init_dataset() {
    use crate::process::Class;
    println!("init_dataset: ...");
    let load_group = |pattern: &str, class: Class| {
        glob::glob(pattern)
            .expect("input glob")
            .filter_map(Result::ok)
            .collect::<Vec<_>>()
            .into_par_iter()
            .enumerate()
            .for_each(|(ix, input_path)| {
                // LOAD & PROCESS IMAGE
                use image::{GenericImage, GenericImageView};
                let image = ::image::open(input_path).expect("load input image");
                let image = crate::process::quantizer(&image);
                let image = image.resize_exact(28, 28, ::image::FilterType::Lanczos3);
                // SAVE
                let dir_path = PathBuf::from(format!("data/cache/{}", class));
                let file_path = dir_path.join(format!("{}.png", ix));
                std::fs::create_dir_all(&dir_path);
                image.save(&file_path);
            });
    };
    load_group("assets/samples/focus/high/**/*.jpeg", Class::Hi);
    load_group("assets/samples/focus/low/**/*.jpeg", Class::Lo);
    load_group("assets/samples/focus/extra-low/**/*.jpeg", Class::ExLo);
    load_group("assets/samples/focus/high-basic/**/*.jpeg", Class::HiBasic);
    println!("init_dataset: done");
}


pub fn load_dataset() -> Vec<(PathBuf, Vec<f32>, Class)> {
    use crate::process::Class;
    use rand::seq::SliceRandom;
    let load = || -> Vec<(PathBuf, Vec<f32>, Class)> {
        Class::all_variants()
            .into_iter()
            .flat_map(|cls| {
                let mut images = glob::glob(&format!("data/cache/{}/**/*.png", cls))
                    .expect("input glob failed")
                    .filter_map(Result::ok)
                    .collect::<Vec<_>>();
                let mut rng = rand::thread_rng();
                images.shuffle(&mut rng);
                images
                    .into_par_iter()
                    .map(|input_path| (input_path.clone(), ::image::open(input_path).expect("load input image")))
                    .map(|(input_path, image)| {
                        let image = image
                            .to_luma()
                            .into_raw()
                            .into_iter()
                            .map(|x| (x as f32))
                            .collect::<Vec<_>>();
                        (input_path, image, cls.clone())
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    println!("loading dataset: ...");
    let dataset = {
        let xs = load();
        if xs.is_empty() {
            init_dataset();
            load()
        } else {
            xs
        }
    };
    println!("loading dataset: done");
    assert!(!dataset.is_empty());
    dataset
}

pub fn run() {
    use prophet::prelude::*;
    use ndarray::prelude::*;

    let (t, f)  = (1.0, -1.0);
    let mut train_samples  = Vec::new();

    fn map_class(class: Class) -> Vec<f32> {
        match class {
            Class::Hi => {
                vec![1.0, -1.0, -1.0, -1.0]
            }
            Class::HiBasic => {
                vec![-1.0, 1.0, -1.0, -1.0]
            }
            Class::Lo => {
                vec![-1.0, -1.0, 1.0, -1.0]
            }
            Class::ExLo => {
                vec![-1.0, -1.0, -1.0, 1.0]
            }
        }
    }

    for (name, image, class) in load_dataset() {
        
        // let x_train = NdArray::from_shape_vec(
        //     ndarray::IxDyn(&[num_image_train, 1, 28, 28]),
        //     train_x,
        // ).expect("init ngarray");

        // let y_train = NdArray::from_shape_vec(
        //     ndarray::IxDyn(&[num_label_train, 1]),
        //     train_y,
        // ).expect("init ngarray");

        train_samples.push(Sample {
            input: ndarray::Array1::from_vec(image),
            target: ndarray::Array1::from_vec(map_class(class).to_owned()),
        });
    }
    
    // create the topology for our neural network
    let top = Topology::input(784)
        .layer(1200, Tanh)
        .layer(64, Tanh)
        .layer(32, Tanh)
        .layer(16, Tanh)
        .layer(8, Tanh)
        .output(4, Tanh);
    
    let mut net = top.train(train_samples)
        .learn_rate(0.25)    // use the given learn rate
        .learn_momentum(0.6) // use the given learn momentum
        .log_config(LogConfig::Iterations(100)) // log state every 100 iterations
        .scheduling(Scheduling::Random)         // use random sample scheduling
        .criterion(Criterion::RecentMSE(0.05))  // train until the recent MSE is below 0.05
        .go()
        .expect("train model");
    
    // PROFIT! now you can use the neural network to predict data!

    // println!("result f -> {}", net.predict(&[f, f])[0].round());
    // println!("result t -> {}", net.predict(&[f, t])[0].round());
    
    // assert_eq!(net.predict(&[f, f])[0].round(), f);
    // assert_eq!(net.predict(&[f, t])[0].round(), t);
    for (name, image, class) in load_dataset() {
        let name = name.to_str().expect("PathBuf to str");
        let result = net.predict(&ndarray::Array1::from_vec(image))
            .to_vec()
            .into_iter()
            .map(|x| x.round())
            .collect::<Vec<_>>();
        println!("{:?}: {:?}: {}", class, result, name);
    }
}

