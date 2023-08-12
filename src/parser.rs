use crate::parser::Token::{
    Array, BulkString, Command, Error, Integer, NullBulkString, SimpleString,
};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Hash)]
pub enum CommandIdent {
    Ping,
    Echo,
    Get,
    Set,
    Px,
}

impl CommandIdent {
    pub fn from_str(some_str: &str) -> Option<CommandIdent> {
        match some_str.to_uppercase().as_str() {
            "PING" => Some(CommandIdent::Ping),
            "ECHO" => Some(CommandIdent::Echo),
            "GET" => Some(CommandIdent::Get),
            "SET" => Some(CommandIdent::Set),
            "PX" => Some(CommandIdent::Px),
            _ => None,
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Hash)]
pub enum Token {
    Array(i32, Vec<Token>),
    BulkString(i32, String),
    Integer(i32),
    SimpleString(String),
    Error(String),
    NullBulkString,
    Command(CommandIdent),
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Array(len, tokens) => {
                write!(
                    f,
                    "*{}\r\n{}\r\n",
                    len,
                    tokens
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<String>>()
                        .join("\r\n"),
                )
            }
            BulkString(_, str) => {
                write!(f, "${}\r\n{}\r\n", str.len(), str)
            }
            Integer(int) => {
                write!(f, ":{}\r\n", int)
            }
            SimpleString(str) => {
                write!(f, "+{}\r\n", str)
            }
            Error(str) => {
                write!(f, "-{}\r\n", str)
            }
            NullBulkString => {
                write!(f, "$-1\r\n")
            }
            Command(command) => {
                write!(f, "Command({:?})", command)
            }
        }
    }
}

impl Token {
    pub fn next_token(stream: &mut VecDeque<String>) -> Token {
        let front = stream.pop_front().unwrap();

        let first_char = front.chars().next().unwrap();
        let other_chars = front.chars().skip(1).collect::<String>();

        match first_char {
            '$' => {
                let len = i32::from_str(other_chars.as_str()).unwrap();

                if len == -1 {
                    return NullBulkString;
                }

                let str = stream.pop_front().unwrap();

                let op = CommandIdent::from_str(str.as_str());

                if op.is_some() {
                    Command(op.unwrap())
                } else {
                    BulkString(len, str)
                }
            }
            '-' => Error(other_chars),
            ':' => {
                let int = i32::from_str(other_chars.as_str()).unwrap();

                Integer(int)
            }
            '+' => SimpleString(other_chars),
            '*' => {
                let len = i32::from_str(other_chars.as_str()).unwrap();
                let mut tokens = Vec::new();

                for _ in 0..len {
                    tokens.push(Token::next_token(stream));
                }

                Array(len, tokens)
            }
            _ => {
                panic!("Invalid token")
            }
        }
    }
}

pub struct Parser {
    pub input: VecDeque<String>,
}

impl Parser {
    pub fn new(mut input: String) -> Self {
        input = input.trim().to_string();

        let sp_input: VecDeque<String> = input.split("\r\n").map(|s| s.to_string()).collect();

        Self { input: sp_input }
    }
}

impl Iterator for Parser {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            return None;
        }

        Some(Token::next_token(&mut self.input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let input = "*1\r\n$4\r\nPING\r\n".to_string();
        let mut sp_input: VecDeque<String> = input.split("\r\n").map(|s| s.to_string()).collect();

        let token = Token::next_token(&mut sp_input);

        assert_eq!(token, Array(1, vec![Command(CommandIdent::Ping)]));
    }

    #[test]
    fn test_parser_with_more_complicated_string() {
        let input = "*2\r\n$4\r\nECHO\r\n$4\r\nPING\r\n".to_string();
        let mut sp_input: VecDeque<String> = input.split("\r\n").map(|s| s.to_string()).collect();

        let token = Token::next_token(&mut sp_input);

        assert_eq!(
            token,
            Array(
                2,
                vec![Command(CommandIdent::Echo), Command(CommandIdent::Ping)]
            )
        );
    }

    #[test]
    fn test_into_vec() {
        let input =
            "*2\r\n$4\r\nECHO\r\n$4\r\nPING\r\n*3\r\n$4\r\nECHO\r\n$4\r\nPING\r\n$4\r\nPING\r\n"
                .to_string();
        let parser = Parser::new(input);

        let tokens: Vec<Token> = parser.collect();

        assert_eq!(
            tokens,
            vec![
                Array(
                    2,
                    vec![Command(CommandIdent::Echo), Command(CommandIdent::Ping)]
                ),
                Array(
                    3,
                    vec![
                        Command(CommandIdent::Echo),
                        Command(CommandIdent::Ping),
                        Command(CommandIdent::Ping)
                    ]
                )
            ]
        );
    }

    #[test]
    fn test_get_set() {
        let input =
            "*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n"
                .to_string();

        let parser = Parser::new(input);

        let tokens: Vec<Token> = parser.collect();

        assert_eq!(
            tokens,
            vec![
                Array(
                    3,
                    vec![
                        Command(CommandIdent::Set),
                        BulkString(3, "key".to_string()),
                        BulkString(5, "value".to_string())
                    ]
                ),
                Array(
                    2,
                    vec![Command(CommandIdent::Get), BulkString(3, "key".to_string())]
                )
            ]
        );
    }
}
