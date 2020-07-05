//!
//! The program resource PATCH request.
//!

use serde_derive::Deserialize;

use zinc_compiler::SourceString;

///
/// The program resource PATCH request query.
///
#[derive(Debug, Deserialize)]
pub struct Query {
    /// The name of the uploaded program.
    pub name: String,
}

///
/// The program resource PATCH request body.
///
#[derive(Debug, Deserialize)]
pub struct Body {
    /// The JSON source code tree.
    pub source: SourceString,
}