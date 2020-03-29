//!
//! The array/tuple/structure access operand parser.
//!

use std::cell::RefCell;
use std::rc::Rc;

use crate::error::Error;
use crate::lexical;
use crate::lexical::Lexeme;
use crate::lexical::Symbol;
use crate::lexical::Token;
use crate::lexical::TokenStream;
use crate::syntax::error::Error as SyntaxError;
use crate::syntax::parser::expression::path::Parser as PathOperandParser;
use crate::syntax::parser::expression::terminal::list::Parser as ExpressionListParser;
use crate::syntax::parser::expression::Parser as ExpressionParser;
use crate::syntax::tree::expression::tree::builder::Builder as ExpressionTreeBuilder;
use crate::syntax::tree::expression::tree::node::operand::Operand as ExpressionOperand;
use crate::syntax::tree::expression::tree::node::operator::Operator as ExpressionOperator;
use crate::syntax::tree::expression::tree::Tree as ExpressionTree;
use crate::syntax::tree::literal::integer::Literal as IntegerLiteral;
use crate::syntax::tree::member_integer::builder::Builder as MemberIntegerBuilder;
use crate::syntax::tree::member_string::builder::Builder as MemberStringBuilder;

#[derive(Debug, Clone, Copy)]
pub enum State {
    PathOperand,
    ExclamationMarkOrNext,
    AccessOrCallOrEnd,
    IndexExpression,
    BracketSquareRight,
    FieldDescriptor,
    ArgumentList,
    ParenthesisRight,
}

impl Default for State {
    fn default() -> Self {
        State::PathOperand
    }
}

#[derive(Default)]
pub struct Parser {
    state: State,
    next: Option<Token>,
    builder: ExpressionTreeBuilder,

    is_indexed: bool,
}

impl Parser {
    pub fn parse(
        mut self,
        stream: Rc<RefCell<TokenStream>>,
        mut initial: Option<Token>,
    ) -> Result<(ExpressionTree, Option<Token>), Error> {
        loop {
            match self.state {
                State::PathOperand => {
                    let (expression, next) =
                        PathOperandParser::default().parse(stream.clone(), initial.take())?;
                    self.next = next;
                    self.builder.eat(expression);
                    self.state = State::ExclamationMarkOrNext;
                }
                State::ExclamationMarkOrNext => {
                    match crate::syntax::parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::ExclamationMark),
                            ..
                        } => {
                            // self.auxiliary = Some((location, ExpressionAuxiliary::CallBuiltIn));
                            // TODO
                        }
                        token => self.next = Some(token),
                    }
                    self.state = State::AccessOrCallOrEnd;
                }
                State::AccessOrCallOrEnd => {
                    match crate::syntax::parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::BracketSquareLeft),
                            location,
                        } => {
                            self.builder
                                .eat_operator(ExpressionOperator::Index, location);
                            self.is_indexed = true;
                            self.state = State::IndexExpression;
                        }
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::ParenthesisLeft),
                            location,
                        } => {
                            self.builder
                                .eat_operator(ExpressionOperator::Call, location);
                            self.state = State::ArgumentList;
                        }
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::Dot),
                            location,
                        } => {
                            self.builder
                                .eat_operator(ExpressionOperator::Field, location);
                            self.is_indexed = true;
                            self.state = State::FieldDescriptor;
                        }
                        token => {
                            return Ok((self.builder.finish(), Some(token)));
                        }
                    }
                }
                State::IndexExpression => {
                    let (expression, next) =
                        ExpressionParser::default().parse(stream.clone(), self.next.take())?;
                    self.next = next;
                    self.builder.eat(expression);
                    self.state = State::BracketSquareRight;
                }
                State::BracketSquareRight => {
                    match crate::syntax::parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::BracketSquareRight),
                            ..
                        } => {
                            self.state = State::AccessOrCallOrEnd;
                        }
                        Token { lexeme, location } => {
                            return Err(Error::Syntax(SyntaxError::expected_one_of_or_operator(
                                location,
                                vec!["]"],
                                lexeme,
                                None,
                            )))
                        }
                    }
                }
                State::FieldDescriptor => {
                    match crate::syntax::parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme:
                                Lexeme::Literal(lexical::Literal::Integer(
                                    literal @ lexical::IntegerLiteral::Decimal { .. },
                                )),
                            location,
                        } => {
                            let mut builder = MemberIntegerBuilder::default();
                            builder.set_location(location);
                            builder.set_literal(IntegerLiteral::new(location, literal));
                            self.builder.eat_operand(
                                ExpressionOperand::MemberInteger(builder.finish()),
                                location,
                            );
                            self.state = State::AccessOrCallOrEnd;
                        }
                        Token {
                            lexeme: Lexeme::Identifier(identifier),
                            location,
                        } => {
                            let mut builder = MemberStringBuilder::default();
                            builder.set_location(location);
                            builder.set_name(identifier.name);
                            self.builder.eat_operand(
                                ExpressionOperand::MemberString(builder.finish()),
                                location,
                            );
                            self.state = State::AccessOrCallOrEnd;
                        }
                        Token { lexeme, location } => {
                            return Err(Error::Syntax(SyntaxError::expected_field_identifier(
                                location, lexeme, None,
                            )))
                        }
                    }
                }
                State::ArgumentList => {
                    let (expressions, location, next) =
                        ExpressionListParser::default().parse(stream.clone(), None)?;
                    self.next = next;
                    self.builder
                        .eat_operand(ExpressionOperand::List(expressions), location);
                    self.state = State::ParenthesisRight;
                }
                State::ParenthesisRight => {
                    match crate::syntax::parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::ParenthesisRight),
                            ..
                        } => {
                            self.state = State::AccessOrCallOrEnd;
                        }
                        Token { lexeme, location } => {
                            return Err(Error::Syntax(SyntaxError::expected_one_of_or_operator(
                                location,
                                vec![")"],
                                lexeme,
                                None,
                            )))
                        }
                    }
                }
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use std::cell::RefCell;
//     use std::rc::Rc;
//
//     use super::Error;
//     use super::Parser;
//     use crate::lexical;
//     use crate::lexical::Lexeme;
//     use crate::lexical::Location;
//     use crate::lexical::Symbol;
//     use crate::lexical::Token;
//     use crate::lexical::TokenStream;
//     use crate::syntax::error::Error as SyntaxError;
//     use crate::syntax::tree::expression::auxiliary::Auxiliary as ExpressionAuxiliary;
//     use crate::syntax::tree::expression::tree::node::operand::Operand as ExpressionOperand;
//     use crate::syntax::tree::expression::tree::node::operator::Operator as ExpressionOperator;
//     use crate::syntax::tree::identifier::Identifier;
//     use crate::syntax::tree::literal::integer::Literal as IntegerLiteral;
//     use crate::syntax::tree::member_integer::MemberInteger;
//     use crate::syntax::tree::member_string::MemberString;
//
//     #[test]
//     fn ok() {
//         let input = r#"array[42].25.value"#;
//
//         let expected = Ok((
//             Expression::new(
//                 Location::new(1, 1),
//                 vec![
//                     ExpressionElement::new(
//                         Location::new(1, 1),
//                         ExpressionObject::Operand(ExpressionOperand::Identifier(Identifier::new(
//                             Location::new(1, 1),
//                             "array".to_owned(),
//                         ))),
//                     ),
//                     ExpressionElement::new(
//                         Location::new(1, 7),
//                         ExpressionObject::Operand(ExpressionOperand::LiteralInteger(
//                             IntegerLiteral::new(
//                                 Location::new(1, 7),
//                                 lexical::IntegerLiteral::new_decimal("42".to_owned()),
//                             ),
//                         )),
//                     ),
//                     ExpressionElement::new(
//                         Location::new(1, 6),
//                         ExpressionObject::Operator(ExpressionOperator::Index),
//                     ),
//                     ExpressionElement::new(
//                         Location::new(1, 11),
//                         ExpressionObject::Operand(ExpressionOperand::MemberInteger(
//                             MemberInteger::new(
//                                 Location::new(1, 11),
//                                 IntegerLiteral::new(
//                                     Location::new(1, 11),
//                                     lexical::IntegerLiteral::new_decimal("25".to_owned()),
//                                 ),
//                             ),
//                         )),
//                     ),
//                     ExpressionElement::new(
//                         Location::new(1, 10),
//                         ExpressionObject::Operator(ExpressionOperator::Field),
//                     ),
//                     ExpressionElement::new(
//                         Location::new(1, 14),
//                         ExpressionObject::Operand(ExpressionOperand::MemberString(
//                             MemberString::new(Location::new(1, 14), "value".to_owned()),
//                         )),
//                     ),
//                     ExpressionElement::new(
//                         Location::new(1, 13),
//                         ExpressionObject::Operator(ExpressionOperator::Field),
//                     ),
//                     ExpressionElement::new(
//                         Location::new(1, 19),
//                         ExpressionObject::Auxiliary(ExpressionAuxiliary::PlaceEnd),
//                     ),
//                 ],
//             ),
//             Some(Token::new(Lexeme::Eof, Location::new(1, 19))),
//         ));
//
//         let result = Parser::default().parse(Rc::new(RefCell::new(TokenStream::new(input))), None);
//
//         assert_eq!(result, expected);
//     }
//
//     #[test]
//     fn error_expected_bracket_square_right() {
//         let input = r#"array[42)"#;
//
//         let expected: Result<_, Error> = Err(Error::Syntax(SyntaxError::expected_one_of(
//             Location::new(1, 9),
//             vec!["]"],
//             Lexeme::Symbol(Symbol::ParenthesisRight),
//             None,
//         )));
//
//         let result = Parser::default().parse(Rc::new(RefCell::new(TokenStream::new(input))), None);
//
//         assert_eq!(result, expected);
//     }
//
//     #[test]
//     fn error_expected_parenthesis_right() {
//         let input = r#"sort(42, 69]"#;
//
//         let expected: Result<_, Error> = Err(Error::Syntax(SyntaxError::expected_one_of(
//             Location::new(1, 12),
//             vec![")"],
//             Lexeme::Symbol(Symbol::BracketSquareRight),
//             None,
//         )));
//
//         let result = Parser::default().parse(Rc::new(RefCell::new(TokenStream::new(input))), None);
//
//         assert_eq!(result, expected);
//     }
// }
