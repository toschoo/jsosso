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

/// Implements the Json serialiser.
pub mod serializing;

/// Implements the Json parser.
pub mod parsing;

/// Implements the random Json value generator. 
pub mod arbitrary;

#[cfg(test)]
mod test;
