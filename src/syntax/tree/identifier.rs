//!
//! The identifier.
//!

use std::fmt;

use serde_derive::Serialize;

use crate::lexical::Location;

#[derive(Debug, Serialize, PartialEq)]
pub struct Identifier {
    pub location: Location,
    pub name: String,
}

impl Identifier {
    pub fn new(location: Location, name: String) -> Self {
        Self { location, name }
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
