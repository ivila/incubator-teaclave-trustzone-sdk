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

use clap::Parser;
use image::EncodableLayout;
use optee_teec::Context;
use proto::{Image, IMAGE_SIZE};

#[derive(Parser, Debug)]
pub struct Args {
    /// The path of the model.
    #[arg(short, long)]
    model: String,
    /// The path of the input binary, must be 768 byte binary, can be multiple
    #[arg(short, long)]
    binary: Vec<String>,
    /// The path of the input image, must be dimension of 28x28, can be multiple
    #[arg(short, long)]
    image: Vec<String>,
}

pub fn execute(args: &Args) {
    let model_path = std::path::absolute(&args.model).unwrap();
    println!("Load model from \"{}\"", model_path.display());
    let record = std::fs::read(&model_path).unwrap();
    let mut ctx = Context::new().unwrap();
    let mut caller = crate::tee::Model::new(&mut ctx, &record).unwrap();

    let mut binaries: Vec<Image> = args
        .binary
        .iter()
        .map(|v| {
            let data = std::fs::read(v).unwrap();
            assert_eq!(data.len(), IMAGE_SIZE);
            data.try_into().unwrap()
        })
        .collect();
    let images: Vec<Image> = args
        .image
        .iter()
        .map(|v| {
            let img = image::open(v).unwrap().to_luma8();
            let bytes = img.as_bytes();
            assert_eq!(bytes.len(), IMAGE_SIZE);
            bytes.try_into().unwrap()
        })
        .collect();
    binaries.extend(images);

    let result = caller.infer_batch(&binaries).unwrap();
    assert_eq!(binaries.len(), result.len());

    for (i, binary) in args.binary.iter().enumerate() {
        println!("{}. {}: {}", i + 1, binary, result[i]);
    }

    for (i, image) in args.image.iter().enumerate() {
        println!(
            "{}. {}: {}",
            i + args.binary.len() + 1,
            image,
            result[args.binary.len()]
        );
    }
    println!("Infer Success");
}
