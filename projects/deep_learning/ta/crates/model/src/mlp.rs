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

use alloc::vec::Vec;

use burn::{
    config::Config,
    module::Module,
    nn,
    tensor::{backend::Backend, Tensor},
};

/// Configuration to create a [Multilayer Perceptron](Mlp) layer.
#[derive(Config)]
pub struct MlpConfig {
    /// The number of layers.
    #[config(default = 3)]
    pub num_layers: usize,
    /// The dropout rate.
    #[config(default = 0.5)]
    pub dropout: f64,
    /// The size of each layer.
    #[config(default = 256)]
    pub d_model: usize,
}

/// Multilayer Perceptron module.
#[derive(Module, Debug)]
pub struct Mlp<B: Backend> {
    linears: Vec<nn::Linear<B>>,
    dropout: nn::Dropout,
    activation: nn::Relu,
}

impl<B: Backend> Mlp<B> {
    /// Create the module from the given configuration.
    pub fn new(config: &MlpConfig, device: &B::Device) -> Self {
        let mut linears = Vec::with_capacity(config.num_layers);

        for _ in 0..config.num_layers {
            linears.push(nn::LinearConfig::new(config.d_model, config.d_model).init(device));
        }

        Self {
            linears,
            dropout: nn::DropoutConfig::new(0.3).init(),
            activation: nn::Relu::new(),
        }
    }

    /// Applies the forward pass on the input tensor.
    ///
    /// # Shapes
    ///
    /// - input: `[batch_size, d_model]`
    /// - output: `[batch_size, d_model]`
    pub fn forward(&self, input: Tensor<B, 2>) -> Tensor<B, 2> {
        let mut x = input;

        for linear in self.linears.iter() {
            x = linear.forward(x);
            x = self.dropout.forward(x);
            x = self.activation.forward(x);
        }

        x
    }
}
