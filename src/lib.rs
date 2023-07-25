pub mod event_loop;
pub mod traits;
pub mod types;

use crate::traits::RedisLogger;
use crate::types::{RedisError, RedisOp, RedisRequest, RedisType, Result};
use dashmap::DashMap;
use smart_default::SmartDefault;
use std::collections::VecDeque;

/// Defines default size of values stored in Redis at once
pub const DEFAULT_REDIS_SIZE: usize = 10_000;

/// # Redis
///
/// The main structure with the core functionality
///
/// ## Fields
///
/// - `capacity` - capacity of Redis
/// - `logger` - logger
#[derive(Debug, Clone, SmartDefault)]
pub struct Redis<L: RedisLogger + Clone + Default> {
    table: DashMap<RedisType, RedisType>,
    queue: VecDeque<RedisType>,
    #[default(L::default())]
    logger: L,
    #[default(DEFAULT_REDIS_SIZE)]
    capacity: usize,
}

impl<L: RedisLogger + Clone + Default> Redis<L> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            ..Self::default()
        }
    }

    pub fn add(&mut self, key: RedisType, value: RedisType) -> Result<()> {
        self.logger
            .log(RedisOp::Add, Some(key.clone()), Some(value.clone()))?;

        self.table.insert(key.clone(), value.clone());
        self.queue.push_back(key.clone());

        if self.queue.len() > self.capacity {
            let key = self.queue.pop_front().ok_or(RedisError::Custom(
                "Failed to pop key from queue".to_string(),
            ))?;
            self.table.remove(&key).ok_or(RedisError::Custom(
                "Failed to remove key from table".to_string(),
            ))?;
        }

        Ok(())
    }

    pub fn get(&mut self, key: RedisType) -> RedisType {
        self.logger
            .log(RedisOp::Get, Some(key.clone()), None)
            .unwrap_or_default();

        if let Some(value) = self.table.get(&key) {
            return value.clone();
        }
        RedisType::Null
    }

    pub fn ping(&self) -> RedisType {
        RedisType::String("PONG".to_string())
    }

    pub fn delete(&mut self, key: RedisType) -> Result<()> {
        self.logger.log(RedisOp::Delete, Some(key.clone()), None)?;
        self.table.remove(&key);
        Ok(())
    }

    pub fn handle_command(&mut self, command: RedisRequest) -> RedisType {
        match command.op {
            RedisOp::Add => {
                if let Some(key) = command.key {
                    if let Some(value) = command.value {
                        let res = self.add(key, value);

                        if res.is_ok() {
                            return RedisType::Ok;
                        }

                        RedisType::Error(
                            format!("Error while adding: {}", res.unwrap_err()).to_string(),
                        )
                    } else {
                        RedisType::Error("Value is not provided".to_string())
                    }
                } else {
                    RedisType::Error("Key is not provided".to_string())
                }
            }
            RedisOp::Get => {
                if let Some(key) = command.key {
                    self.get(key)
                } else {
                    RedisType::Error("Key is not provided".to_string())
                }
            }
            RedisOp::Delete => {
                if let Some(key) = command.key {
                    let res = self.delete(key);

                    if res.is_ok() {
                        RedisType::Ok
                    } else {
                        RedisType::Error("Error while deleting".to_string())
                    }
                } else {
                    RedisType::Error("Key is not provided".to_string())
                }
            }
            RedisOp::Ping => self.ping(),
            RedisOp::Error => RedisType::Error("Unsupported operation".to_string()),
        }
    }
}
