mod builder;
mod code_generator;
mod error;
mod ta;

pub use builder::*;
pub use code_generator::RustEdition;
pub use error::Error;
pub use ta::*;

pub fn build(edition: RustEdition, uuid: &str, config: TAConfig) -> Result<(), Error> {
    let build_config = Config::new(edition, config);
    build_config.build(uuid)
}
