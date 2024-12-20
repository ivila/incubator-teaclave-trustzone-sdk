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

pub const EXT_PROP_NAME_GP_TA_DESCRIPTION: &str = "gp.ta.description";
pub const EXT_PROP_NAME_GP_TA_VERSION: &str = "gp.ta.version";

/// Configuration options for TA
///
/// Examples
///
/// # use a standard configuration
/// ```rust
/// use optee_utee_build::TAConfig;
/// let ta_config = TAConfig::new_standard(
///     "0.1.0",
///     "hello world",
///     "this is a hello world TA",
/// );
/// ```
///
/// # make some modifications
/// ```rust
/// use optee_utee_build::TAConfig;
/// let ta_config = TAConfig::new_standard(
///     "0.1.0",
///     "hello world",
///     "this is a hello world TA",
/// ).ta_stack_size(10 * 1024).ta_data_size(32 * 1024);
/// ```
#[derive(Debug, Clone)]
pub struct TAConfig {
    pub ta_flags: u32,
    pub ta_data_size: u32,
    pub ta_stack_size: u32,
    pub ta_version: String,
    pub ta_description: String,
    pub trace_level: i32,
    pub trace_ext_prefix: String,
    pub ta_framework_stack_size: u32,
    pub ext_properties: Vec<Property>,
}

impl TAConfig {
    pub fn new_standard(ta_version: &str, ta_description: &str, gp_ta_description: &str) -> Self {
        Self {
            ta_flags: 0,
            ta_data_size: 1 * 1024 * 1024,
            ta_stack_size: 2 * 1024,
            ta_version: ta_version.to_string(),
            ta_description: ta_description.to_string(),
            trace_level: 4,
            trace_ext_prefix: "TA".to_string(),
            ta_framework_stack_size: 2048,
            ext_properties: vec![
                Property::new_gp_ta_description(gp_ta_description),
                Property::new_gp_ta_version(0x0010),
            ],
        }
    }
    pub fn ta_flags(mut self, flags: u32) -> Self {
        self.ta_flags = flags;
        self
    }
    pub fn ta_stack_size(mut self, stack_size: u32) -> Self {
        self.ta_stack_size = stack_size;
        self
    }
    pub fn ta_data_size(mut self, size: u32) -> Self {
        self.ta_data_size = size;
        self
    }
    pub fn trace_level(mut self, level: i32) -> Self {
        self.trace_level = level;
        self
    }
    pub fn trace_ext_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.trace_ext_prefix = prefix.into();
        self
    }
    pub fn ta_framework_stack_size(mut self, stack_size: u32) -> Self {
        self.ta_framework_stack_size = stack_size;
        self
    }
    pub fn add_ext_property(mut self, name: &str, value: PropertyValue) -> Self {
        self.ext_properties.push(Property::new(name, value));
        self
    }
}

#[derive(Debug, Clone)]
pub enum PropertyValue {
    Bool(bool),
    U32(u32),
    UUID(String),
    Identity(u32, String),
    Str(String),
    BinaryBlock(String),
    U64(u64),
}

#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub value: PropertyValue,
}

impl Property {
    pub fn new(name: &str, value: PropertyValue) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }
    pub fn new_gp_ta_description(desc: &str) -> Self {
        Self::new(
            EXT_PROP_NAME_GP_TA_DESCRIPTION,
            PropertyValue::Str(desc.to_string()),
        )
    }
    pub fn new_gp_ta_version(version: u32) -> Self {
        Self::new(EXT_PROP_NAME_GP_TA_VERSION, PropertyValue::U32(version))
    }
}
