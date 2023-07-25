use crate::types::{RedisOp, RedisType, Result};
use log::{info};

/// Trait for support of different log approaches
pub trait RedisLogger: Clone {
    /// Basic log function, that takes Operation, Key and Value
    fn log(
        &self,
        operation: RedisOp,
        key: Option<RedisType>,
        value: Option<RedisType>,
    ) -> Result<()>;
}

/// Pre-defined logger, that basically prints logs to console
#[derive(Default, Clone, Debug)]
pub struct DefaultLogger;

impl RedisLogger for DefaultLogger {
    // TODO: Maybe it would be better to use transaction method to save state changes
    fn log(
        &self,
        operation: RedisOp,
        key: Option<RedisType>,
        value: Option<RedisType>,
    ) -> Result<()> {
        info!("LOG: {:?} {:?} {:?}", operation, key, value);
        Ok(())
    }
}

impl DefaultLogger {
    pub fn new() -> Self {
        Self
    }
}

// TODO: Add file logger
/// File logger : In addition to DefaultLogger, that writes logs to external logging file
/// saving everything in the transactional way
pub struct FileLogger;
