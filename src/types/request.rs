use crate::types::RedisError::ParseFailed;
use crate::types::{RedisOp, RedisType, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RedisRequest {
    pub op: RedisOp,
    pub key: Option<RedisType>,
    pub value: Option<RedisType>,
}

impl RedisRequest {
    pub fn new(op: RedisOp, key: Option<RedisType>, value: Option<RedisType>) -> Self {
        Self { op, key, value }
    }

    /// Make it to return result
    pub fn parse(mut stream: String) -> Result<Self> {
        stream = stream.replace('\0', "");

        let stream = stream.split("\r\n").collect::<Vec<&str>>();

        if stream.first().is_none() {
            return Err(ParseFailed);
        }

        let op = stream[0].to_string();
        let op = match op.as_str() {
            "ADD" => RedisOp::Add,
            "GET" => RedisOp::Get,
            "DELETE" => RedisOp::Delete,
            "PING" => RedisOp::Ping,
            "ERROR" => RedisOp::Error,
            _ => {
                return Ok(Self::new(
                    RedisOp::Error,
                    Some(RedisType::Error("Unknown command".to_string())),
                    None,
                ));
            }
        };

        if stream.get(1).is_none() {
            return Ok(Self::new(op, None, None));
        }

        let key = RedisType::parse(stream[1].to_string())?;

        if stream.get(2).is_none() {
            return Ok(Self::new(op, Some(key), None));
        }

        let value = RedisType::parse(stream[2].to_string())?;

        Ok(Self::new(op, Some(key), Some(value)))
    }
}
