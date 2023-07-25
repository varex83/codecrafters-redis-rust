mod redis_type;
mod request;

pub use redis_type::*;
pub use request::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Wrapper over `std::result::Result` that consumes only success return type `<T>`,
/// and for error returns `RedisError`
pub type Result<T> = std::result::Result<T, RedisError>;

pub type StringType = String;
pub type IntegerType = i128;
pub type EventChannelType = ();
pub type ArrayType = Vec<RedisType>;

/// RedisError - is main error functionality here
///
/// ## Variants
///
/// - `Custom(String)` - used for some unique cases of errors
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RedisError {
    ParseFailed,
    Custom(String),
}

impl From<String> for RedisError {
    fn from(err: String) -> Self {
        RedisError::Custom(err)
    }
}

impl Display for RedisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RedisError::ParseFailed => write!(f, "Parse failed"),
            RedisError::Custom(err) => write!(f, "{}", err),
        }
    }
}

/// Add error
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RedisOp {
    Add,
    Get,
    Delete,
    Ping,
    Error,
}
