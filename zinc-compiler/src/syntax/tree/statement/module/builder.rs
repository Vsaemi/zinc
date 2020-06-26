//!
//! The `mod` statement builder.
//!

use crate::lexical::token::location::Location;
use crate::syntax::tree::identifier::Identifier;
use crate::syntax::tree::statement::module::Statement as ModStatement;

///
/// The `mod` statement builder.
///
#[derive(Default)]
pub struct Builder {
    /// The location of the syntax construction.
    location: Option<Location>,
    /// The module identifier.
    identifier: Option<Identifier>,
}

impl Builder {
    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_location(&mut self, value: Location) {
        self.location = Some(value);
    }

    ///
    /// Sets the corresponding builder value.
    ///
    pub fn set_identifier(&mut self, value: Identifier) {
        self.identifier = Some(value);
    }

    ///
    /// Finalizes the builder and returns the built value.
    ///
    /// # Panics
    /// If some of the required items has not been set.
    ///
    pub fn finish(mut self) -> ModStatement {
        ModStatement::new(
            self.location.take().unwrap_or_else(|| {
                panic!("{}{}", crate::panic::BUILDER_REQUIRES_VALUE, "location")
            }),
            self.identifier.take().unwrap_or_else(|| {
                panic!("{}{}", crate::panic::BUILDER_REQUIRES_VALUE, "identifier")
            }),
        )
    }
}
