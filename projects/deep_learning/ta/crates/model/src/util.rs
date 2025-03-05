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


use burn::prelude::*;
use proto::{Image, IMAGE_WIDTH, IMAGE_HEIGHT};

// Originally inspired by the burn/examples/mnist-inference-web package. You may
// refer to https://github.com/tracel-ai/burn/blob/v0.16.0/examples/mnist-inference-web
// for details.
pub fn image_to_tensor<B: Backend>(device: &B::Device, image: &Image) -> Tensor<B, 3> {
    let tensor = TensorData::from(image.as_slice()).convert::<B::FloatElem>();
    let tensor = Tensor::<B, 1>::from_data(tensor, device);
    let tensor = tensor.reshape([1, IMAGE_WIDTH, IMAGE_HEIGHT]);

    // Normalize input: make between [0,1] and make the mean=0 and std=1
    // values mean=0.1307,std=0.3081 were copied from Pytorch Mist Example
    // https://github.com/pytorch/examples/blob/54f4572509891883a947411fd7239237dd2a39c3/mnist/main.py#L122
    ((tensor / 255) - 0.1307) / 0.3081
}

pub fn images_to_tensors<B: Backend>(device: &B::Device, images: &[Image]) -> Tensor<B, 3> {
    let tensors = images.iter().map(|v| image_to_tensor(device, v)).collect();
    Tensor::cat(tensors, 0)
}

pub fn labels_to_tensors<B: Backend>(device: &B::Device, labels: &[u8]) -> Tensor<B, 1, Int> {
    let targets = labels
        .iter()
        .map(|item| Tensor::<B, 1, Int>::from_data([(*item as i64).elem::<B::IntElem>()], device))
        .collect();
    Tensor::cat(targets, 0)
}
