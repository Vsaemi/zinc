//!
//! The array expression builder.
//!

use crate::lexical::token::location::Location;
use crate::syntax::tree::expression::array::Expression as ArrayExpression;
use crate::syntax::tree::expression::tree::Tree as ExpressionTree;

///
/// The array expression builder.
///
#[derive(Default)]
pub struct Builder {
    /// The location of the syntax construction.
    location: Option<Location>,
    /// The array element expressions, used in the list array literal.
    elements: Vec<ExpressionTree>,
    /// The array size expression, used in the repeated array literal.
    size_expression: Option<ExpressionTree>,
}

impl Builder {
    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_location(&mut self, value: Location) {
        self.location = Some(value);
    }

    ///
    /// Pushes the corresponding builder value.
    ///
    pub fn push_expression(&mut self, expression: ExpressionTree) {
        self.elements.push(expression);
    }

    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_size_expression(&mut self, value: ExpressionTree) {
        self.size_expression = Some(value);
    }

    ///
    /// Finalizes the builder and returns the built value.
    ///
    /// # Panics
    /// If some of the required items has not been set.
    ///
    pub fn finish(mut self) -> ArrayExpression {
        let location = self
            .location
            .take()
            .unwrap_or_else(|| panic!("{}{}", crate::panic::BUILDER_REQUIRES_VALUE, "location"));

        match self.size_expression.take() {
            Some(size_expression) => ArrayExpression::new_repeated(
                location,
                self.elements.pop().unwrap_or_else(|| {
                    panic!(
                        "{}{}",
                        crate::panic::BUILDER_REQUIRES_VALUE,
                        "size expression"
                    )
                }),
                size_expression,
            ),
            None => ArrayExpression::new_list(location, self.elements),
        }
    }
}
