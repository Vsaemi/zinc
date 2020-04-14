//!
//! The semantic analyzer contract type element error.
//!

#[derive(Debug, PartialEq)]
pub enum Error {
    DuplicateField {
        type_identifier: String,
        field_name: String,
    },
}
