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

use core::convert::TryInto;

use crate::tee::Trainer;
use optee_teec::Context;
use proto::{Image, IMAGE_SIZE};
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

fn convert_datasets(images: &[u8], labels: &[u8]) -> Vec<(Image, u8)> {
    let mut datasets: Vec<(Image, u8)> = images
        .chunks_exact(IMAGE_SIZE)
        .map(|v| v.try_into().unwrap())
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
    let data = mnist::MnistBuilder::new()
        .base_url("https://storage.googleapis.com/cvdf-datasets/mnist/")
        .base_path(
            std::env::temp_dir()
                .join("teaclave_example_mnist_rs/")
                .to_str()
                .expect("Should be a valid str"),
        )
        .download_and_extract()
        .training_set_length(60_000)
        .validation_set_length(10_000)
        .test_set_length(0)
        .finalize();
    // Prepare datasets
    let train_datasets = convert_datasets(&data.trn_img, &data.trn_lbl);
    let valid_datasets = convert_datasets(&data.val_img, &data.val_lbl);
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
}
