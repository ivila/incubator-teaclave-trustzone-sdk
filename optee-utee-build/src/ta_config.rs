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
use crate::Error;
use std::convert::TryInto;
use std::str::FromStr;

pub const EXT_PROP_NAME_GP_TA_DESCRIPTION: &str = "gp.ta.description";
pub const EXT_PROP_NAME_GP_TA_VERSION: &str = "gp.ta.version";

/// Configuration options for TA
///
/// Examples
///
/// # use a standard configuration
/// ```rust
/// use optee_utee_build::TAConfig;
/// # use optee_utee_build::Error;
/// # fn main() -> Result<(), Error> {
/// const UUID: &str = "d93c2970-b1a6-4b86-90ac-b42830e78d9b";
/// let ta_config = TAConfig::new_standard(
///     UUID,
///     "0.1.0",
///     "hello world",
/// )?;
/// # Ok(())
/// # }
/// ```
///
/// and since we already have `version` and `description` in `cargo.toml`,
/// we can make it simpler by using them as parameters:
///
/// ```rust
/// use optee_utee_build::TAConfig;
/// # use optee_utee_build::Error;
/// # fn main() -> Result<(), Error> {
/// const UUID: &str = "d93c2970-b1a6-4b86-90ac-b42830e78d9b";
/// let ta_config = TAConfig::new_standard_with_cargo_env(UUID)?;
/// # Ok(())
/// # }
/// ```
///
/// # make some modifications
/// ```rust
/// use optee_utee_build::TAConfig;
/// # use optee_utee_build::Error;
/// # fn main() -> Result<(), Error> {
/// const UUID: &str = "d93c2970-b1a6-4b86-90ac-b42830e78d9b";
/// let ta_config = TAConfig::new_standard(
///     UUID,
///     "0.1.0",
///     "hello world",
/// )?.ta_stack_size(10 * 1024).ta_data_size(32 * 1024);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct TAConfig {
    pub uuid: uuid::Uuid,
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
    /// generate a standard config by uuid, retrieve version and description from cargo.toml
    pub fn new_standard_with_cargo_env(uuid_str: &str) -> Result<Self, Error> {
        Self::new_standard(
            uuid_str,
            std::env::var("CARGO_PKG_VERSION")?.as_str(),
            std::env::var("CARGO_PKG_DESCRIPTION")?.as_str(),
        )
    }
    /// generate a standard config
    ///
    /// the ta_version should be in semver formation
    pub fn new_standard(
        uuid_str: &str,
        ta_version: &str,
        ta_description: &str,
    ) -> Result<Self, Error> {
        Ok(Self {
            uuid: uuid_str.try_into()?,
            ta_flags: 0,
            ta_data_size: 1 * 1024 * 1024,
            ta_stack_size: 2 * 1024,
            ta_version: ta_version.to_string(),
            ta_description: ta_description.to_string(),
            trace_level: 4,
            trace_ext_prefix: "TA".to_string(),
            ta_framework_stack_size: 2048,
            ext_properties: vec![
                Property::new_gp_ta_description(ta_description),
                Property::new_gp_ta_version(parse_version_str(ta_version)?),
            ],
        })
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

/// An enum of PropertyValue, with its type and value combined
///
/// Usage:
/// ```rust
/// # use optee_utee_build::Error;
/// # fn main() -> Result<(), Error> {
/// # use optee_utee_build::PropertyValue;
/// # use std::convert::TryInto;
/// # const UUID: &str = "d93c2970-b1a6-4b86-90ac-b42830e78d9b";
/// # const LOGIN: u32 = 1;
/// PropertyValue::Bool(true);
/// PropertyValue::U32(1);
/// PropertyValue::UUID(UUID.try_into()?);
/// PropertyValue::Identity(LOGIN, UUID.try_into()?);
/// // a string value, must not append '\0' at last, we will add it for you.
/// PropertyValue::Str("hello world".to_string());
/// // a base64 string value, must not append '\0' at last, we will add it for you.
/// PropertyValue::BinaryBlock("c2RmYXNm".to_string());
/// PropertyValue::U64(1);
/// # Ok(())
/// # }
#[derive(Debug, Clone)]
pub enum PropertyValue {
    Bool(bool),
    U32(u32),
    UUID(uuid::Uuid),
    Identity(u32, uuid::Uuid),
    Str(String),
    BinaryBlock(String),
    U64(u64),
}

/// A GP property pair, use it to set ta_properties
///
/// must not append a '\0' in name, we will add it automatically if neccessary.
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    /// value of the property
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

/// parse a version_str in semver format into a gp standard u32 version value.
pub fn parse_version_str(version_str: &str) -> Result<u32, Error> {
    let version = semver::Version::from_str(version_str)
        .map_err(|err| Error::InvalidVersion(err.to_string()))?;
    if version.major > (std::u16::MAX as u64) {
        return Err(Error::InvalidVersion(format!(
            "Major version must not greater than {}, got {}",
            std::u16::MAX,
            version.major
        )));
    }
    if version.minor > (std::u8::MAX as u64) {
        return Err(Error::InvalidVersion(format!(
            "Minor version must not greater than {}, got {}",
            std::u8::MAX,
            version.minor
        )));
    }

    if version.patch > (std::u8::MAX as u64) {
        return Err(Error::InvalidVersion(format!(
            "Patch version must not greater than {}, got {}",
            std::u8::MAX,
            version.patch
        )));
    }
    Ok(((version.major << 16) | (version.minor << 8) | (version.patch)) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_str() {
        let gp_version = parse_version_str("0.0.1").unwrap();
        assert_eq!(gp_version, 0x0001);
        let gp_version = parse_version_str("0.1.0").unwrap();
        assert_eq!(gp_version, 0x0100);
        let gp_version = parse_version_str("1.0.0").unwrap();
        assert_eq!(gp_version, 0x10000);
    }
}
