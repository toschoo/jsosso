//! Jsosso is a simple Json parser.
//! It's main purpose is to serve as a demonstrator for [`pacosso`].
//! As such, it highlights simplicity, not performance
//! or other features you may expect from a full-fledged Json parser.
//!
//! This crate implements a Json Enum and contains modules
//! - to serialise Json values
//! - to parse Json values from streams
//! - and to generate random Json values.
//!
//! It also provides an executable with some examples and benchmarks.
//!
//!  [`pacosso`]: https://github.com/toschoo/pacosso

use std::collections::HashMap;
use pacosso::{Stream, ParseResult, ParseError};

/// Representation of a Json value.
#[derive(Debug, PartialEq)]
pub enum Json {
    /// Represents a Json 'null' value.
    Null,
    /// Represents a Json boolean.
    Boolean(bool),
    /// Represents a Json number.
    Number(f64),
    /// Represents a Json string.
    String(String),
    /// Rpresents a Json array.
    Array(Vec<Json>),
    /// Rpresents a Json object.
    Object(Box<HashMap<String, Json>>),
}

impl From<bool> for Json {
    fn from(b: bool) -> Json {
        Json::Boolean(b)
    }
}

impl From<String> for Json {
    fn from(s: String) -> Json {
        Json::String(s)
    }
}

impl<'a> From<&'a str> for Json {
    fn from(s: &'a str) -> Json {
        Json::String(s.to_string())
    }
}

// From<n> for all number types n
macro_rules! impl_from_num_for_json {
    ( $( $t:ident)* ) => {
        $(
            impl From<$t> for Json {
                fn from(n: $t) -> Json {
                    Json::Number(n as f64)
                }
            }
        )*
    };
}

impl_from_num_for_json!(
    u8 i8 u16 i16 u32 i32 u64 i64 usize isize f32 f64
);


/// Implements the Json serialiser.
pub mod serializing;

/// Implements the Json parser.
pub mod parsing;

/// Implements and embedded Json representation language. 
#[macro_use] mod dsl;

/// Implements the random Json value generator. 
pub mod arbitrary;

#[cfg(test)]
mod test;
