#[derive(Debug)]
pub enum Error {
    Env(std::env::VarError),
    IO(std::io::Error),
    UUID(uuid::Error),
    PropertyNotFound(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<std::env::VarError> for Error {
    fn from(value: std::env::VarError) -> Self {
        Self::Env(value)
    }
}

impl From<uuid::Error> for Error {
    fn from(value: uuid::Error) -> Self {
        Self::UUID(value)
    }
}
