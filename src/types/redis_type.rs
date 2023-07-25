use crate::types::{ArrayType, EventChannelType, IntegerType, RedisError, Result, StringType};
use serde::{Deserialize, Serialize};

/// RedisType - is implementation of basic Redis' types
/// Derives `Serialize`/`Deserialize`
///
/// ## Variants
///
/// - `String(StringType)` -- Basic string
/// - `Integer(IntegerType)` -- Basic integer type
/// - `Array(ArrayType)` -- Array implementation
/// - `EventChannel(EventChannelType)` -- Channel type
/// - `Null` - Just nothing :D
#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialOrd, PartialEq)]
pub enum RedisType {
    String(StringType),
    Integer(IntegerType),
    Array(ArrayType),
    EventChannel(EventChannelType),
    Error(StringType),
    Null,
    Ok,
}

impl RedisType {
    /// Rewrite this
    pub fn parse(str: String) -> Result<Self> {
        let first_byte = str
            .chars()
            .next()
            .ok_or(RedisError::Custom("Empty string".to_string()))?;

        let other = str
            .strip_prefix(first_byte)
            .ok_or(RedisError::Custom("Stripping prefix failed".to_string()))?
            .to_string();

        Ok(match first_byte {
            '+' => RedisType::String(other),
            ':' => RedisType::Integer(
                other
                    .parse::<IntegerType>()
                    .map_err(|_| RedisError::Custom("Parsing integer failed".to_string()))?,
            ),
            '*' => {
                let mut arr = ArrayType::new();
                let mut lines = other.lines();
                lines.next();
                for line in lines {
                    arr.push(RedisType::parse(line.to_string())?);
                }
                RedisType::Array(arr)
            }
            '$' => {
                if other.as_str() == "$-1" {
                    RedisType::Null
                } else {
                    RedisType::Error(format!("Unknown type"))
                }
            }
            '-' => RedisType::Error(other),
            _ => RedisType::Error(format!("Unknown type {}", str)),
        })
    }

    pub fn as_string_formatted(&self) -> String {
        match self {
            RedisType::String(str) => {
                let mut res = String::from("+");
                res.push_str(str);
                res.push_str("\r\n");
                res
            }
            RedisType::Integer(int) => {
                let mut res = String::from(":");
                res.push_str(&int.to_string());
                res.push_str("\r\n");
                res
            }
            RedisType::Array(arr) => {
                let mut res = String::from("*");
                res.push_str(&arr.len().to_string());
                res.push_str("\r\n");
                for item in arr {
                    res.push_str(&item.as_string_formatted().to_string());
                    res.push_str("\r\n");
                }
                res
            }
            RedisType::Error(str) => {
                let mut res = String::from("-");
                res.push_str(str);
                res.push_str("\r\n");
                res
            }
            RedisType::EventChannel(_ch) => "+OK\r\n".to_string(),
            RedisType::Null => "$-1\r\n".to_string(),
            RedisType::Ok => "+OK\r\n".to_string(),
        }
    }
}
