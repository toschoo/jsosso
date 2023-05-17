use std::collections::HashMap;
use pacosso::{Stream, ParseResult, ParseError};

#[derive(Debug)]
pub enum Json {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Box<HashMap<String, Json>>),
}

pub mod serializing;
pub mod parsing;

#[cfg(test)]
mod test;
