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

// Originally copied from the burn/crates/no-std-tests package. You may refer
// to https://github.com/tracel-ai/burn/blob/v0.16.0/crates/burn-no-std-test for
// details.

use crate::{
    conv::{ConvBlock, ConvBlockConfig},
    mlp::{Mlp, MlpConfig},
};
use alloc::vec::Vec;
use burn::{
    prelude::*,
    record::{FullPrecisionSettings, Recorder, RecorderError},
    tensor::cast::ToElement,
};
use proto::{Image, IMAGE_SIZE, NUM_CLASSES};

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    mlp: Mlp<B>,
    conv: ConvBlock<B>,
    input: nn::Linear<B>,
    output: nn::Linear<B>,
}

impl<B: Backend> Model<B> {
    pub fn new(device: &B::Device) -> Self {
        let mlp_config = MlpConfig::new();
        let mlp = Mlp::new(&mlp_config, device);
        let input = nn::LinearConfig::new(IMAGE_SIZE, mlp_config.d_model).init(device);
        let output = nn::LinearConfig::new(mlp_config.d_model, NUM_CLASSES).init(device);
        let conv = ConvBlock::new(&ConvBlockConfig::new([1, 1]), device);

        Self {
            mlp,
            conv,
            output,
            input,
        }
    }

    pub fn forward(&self, input: Tensor<B, 3>) -> Tensor<B, 2> {
        let [batch_size, height, width] = input.dims();

        let x = input.reshape([batch_size, 1, height, width]).detach();
        let x = self.conv.forward(x);
        let x = x.reshape([batch_size, height * width]);

        let x = self.input.forward(x);
        let x = self.mlp.forward(x);

        self.output.forward(x)
    }

    pub fn infer(&self, device: &B::Device, input: &Image) -> u8 {
        let tensor = crate::util::image_to_tensor(device, input);
        let output = self.forward(tensor);
        let output = burn::tensor::activation::softmax(output, 1);
        output.argmax(1).into_scalar().to_u8()
    }

    pub fn export(&self) -> Result<Vec<u8>, RecorderError> {
        let recorder = burn::record::BinBytesRecorder::<FullPrecisionSettings>::new();
        recorder.record(self.clone().into_record(), ())
    }

    pub fn import(device: &B::Device, record: Vec<u8>) -> Result<Self, RecorderError> {
        let recorder = burn::record::BinBytesRecorder::<FullPrecisionSettings>::new();
        let record = recorder.load(record, device)?;

        let m = Self::new(device);
        Ok(m.load_record(record))
    }
}
