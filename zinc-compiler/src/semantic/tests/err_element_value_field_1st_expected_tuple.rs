//!
//! A semantic analyzer test.
//!

#![cfg(test)]

use crate::lexical::Location;

use crate::semantic::ElementError;
use crate::semantic::Error as SemanticError;
use crate::semantic::Type;
use crate::semantic::ValueError;

use crate::Error;

#[test]
fn test() {
    let input = r#"
fn main(input: (), witness: ()) {
    let value = [true, true, false].1;
}
"#;

    let expected = Err(Error::Semantic(SemanticError::Element(
        Location::new(3, 36),
        ElementError::Value(ValueError::OperatorFieldFirstOperandExpectedTuple(
            Type::new_array(Type::new_boolean(), 3).to_string(),
        )),
    )));

    let result = super::get_binary_result(input);

    assert_eq!(expected, result);
}
