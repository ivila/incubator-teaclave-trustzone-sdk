// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use std::io::{Cursor, Read};
use std::path::PathBuf;

use crate::tee::Trainer;
use optee_teec::Context;
use proto::Image;
use rand::seq::SliceRandom;

#[derive(clap::Parser, Debug)]
pub struct Args {
    #[arg(short, long, default_value_t = 6)]
    num_epochs: usize,
    #[arg(short, long, default_value_t = 64)]
    batch_size: usize,
    #[arg(short, long, default_value_t = 0.0001)]
    learning_rate: f64,
    #[arg(short, long, default_value = "model.bin")]
    output: String,
}

fn convert_datasets(images: &Vec<Image>, labels: &[u8]) -> Vec<(Image, u8)> {
    let mut datasets: Vec<(Image, u8)> = images
        .iter()
        .map(|v| v.to_owned())
        .zip(labels.iter().copied())
        .collect();
    datasets.shuffle(&mut rand::rng());
    datasets
}

pub fn execute(args: &Args) {
    // Initialize trainer
    let mut ctx = Context::new().unwrap();
    let mut trainer = Trainer::new(&mut ctx, args.learning_rate).unwrap();
    let output_path = std::path::absolute(&args.output).unwrap();
    // Download mnist data
    let data = check_download_mnist_data();
    // Prepare datasets
    let train_datasets = convert_datasets(&data.train_data, &data.train_labels);
    let valid_datasets = convert_datasets(&data.test_data, &data.test_labels);
    // Training loop, Originally inspired by burn/crates/custom-training-loop
    for epoch in 1..args.num_epochs + 1 {
        for (iteration, data) in train_datasets.chunks(args.batch_size).enumerate() {
            let images: Vec<Image> = data.iter().map(|v| v.0).collect();
            let labels: Vec<u8> = data.iter().map(|v| v.1).collect();
            let output = trainer.train(&images, &labels).unwrap();
            println!(
                "[Train - Epoch {} - Iteration {}] Loss {:.3} | Accuracy {:.3} %",
                epoch, iteration, output.loss, output.accuracy,
            );
        }

        for (iteration, data) in valid_datasets.chunks(args.batch_size).enumerate() {
            let images: Vec<Image> = data.iter().map(|v| v.0).collect();
            let labels: Vec<u8> = data.iter().map(|v| v.1).collect();
            let output = trainer.valid(&images, &labels).unwrap();
            println!(
                "[Valid - Epoch {} - Iteration {}] Loss {:.3} | Accuracy {:.3} %",
                epoch, iteration, output.loss, output.accuracy,
            );
        }
    }
    // Export the model to the given path
    let record = trainer.export().unwrap();
    println!("Export record to \"{}\"", output_path.display());
    std::fs::write(&output_path, &record).unwrap();
    println!("Train Success");
}

fn check_download_mnist_data() -> rust_mnist::Mnist {
    const DATA_PATH: &str = "./data/";

    let folder = PathBuf::from(DATA_PATH);
    if !folder.exists() {
        std::fs::create_dir_all(&folder).unwrap();
    }
    for (filename, gz_size, flat_size) in [
        ("train-images-idx3-ubyte", 9912422, 47040016),
        ("train-labels-idx1-ubyte", 28881, 60008),
        ("t10k-images-idx3-ubyte", 1648877, 7840016),
        ("t10k-labels-idx1-ubyte", 4542, 10008),
    ]
    .iter()
    {
        let file = folder.join(filename);
        if file.exists() && file.is_file() && std::fs::metadata(&file).unwrap().len() == *flat_size
        {
            println!("File {} exist, skip.", file.display());
            continue;
        }

        let url = format!(
            "https://storage.googleapis.com/cvdf-datasets/mnist/{}.gz",
            filename
        );
        println!("Download {} from {}", filename, url);
        let body = ureq::get(&url)
            .call()
            .unwrap()
            .body_mut()
            .read_to_vec()
            .unwrap();

        assert_eq!(body.len(), *gz_size as usize);

        let mut gz = flate2::bufread::GzDecoder::new(Cursor::new(body));
        let mut buffer = Vec::with_capacity(*flat_size as usize);
        gz.read_to_end(&mut buffer).unwrap();

        std::fs::write(file, &buffer).unwrap();
    }

    rust_mnist::Mnist::new(DATA_PATH)
}
