// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use termcolor::StandardStreamLock;

use crate::lexer::{TLexer, Token};
use crate::parser::attributes::{Attributes, AttributesParser};
use crate::parser::dump::Dump;
use crate::parser::errors::ParserError;
use crate::parser::expressions::{ExprNode, ExpressionParser};
use crate::parser::types::Type;
use crate::parser::Context;

#[derive(Clone, Debug, PartialEq)]
pub struct Dimension {
    pub size: Option<ExprNode>,
    pub attributes: Option<Attributes>,
}

impl Dump for Dimension {
    fn dump(&self, name: &str, prefix: &str, last: bool, stdout: &mut StandardStreamLock) {
        dump_obj!(self, name, "", prefix, last, stdout, size, attributes);
    }
}

pub type Dimensions = Vec<Dimension>;

impl Dump for Dimensions {
    fn dump(&self, name: &str, prefix: &str, last: bool, stdout: &mut StandardStreamLock) {
        dump_vec!(name, self, "dim", prefix, last, stdout);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Array {
    pub base: Option<Type>,
    pub dimensions: Dimensions,
}

impl Dump for Array {
    fn dump(&self, name: &str, prefix: &str, last: bool, stdout: &mut StandardStreamLock) {
        dump_obj!(self, name, "array", prefix, last, stdout, base, dimensions);
    }
}

pub struct ArrayParser<'a, L: TLexer> {
    lexer: &'a mut L,
}

impl<'a, L: TLexer> ArrayParser<'a, L> {
    pub(super) fn new(lexer: &'a mut L) -> Self {
        Self { lexer }
    }

    pub(super) fn parse(
        self,
        tok: Option<Token>,
        context: &mut Context,
    ) -> Result<(Option<Token>, Option<Array>), ParserError> {
        let mut tok = tok.unwrap_or_else(|| self.lexer.next_useful());
        let mut dimensions = Vec::new();

        loop {
            if tok != Token::LeftBrack {
                break;
            }

            let mut ep = ExpressionParser::new(self.lexer, Token::RightBrack);
            let (tk, size) = ep.parse(None, context)?;

            let tk = tk.unwrap_or_else(|| self.lexer.next_useful());
            if tk != Token::RightBrack {
                return Err(ParserError::InvalidTokenInArraySize {
                    sp: self.lexer.span(),
                    tok,
                });
            }

            let ap = AttributesParser::new(self.lexer);
            let (tk, attributes) = ap.parse(None, context)?;

            tok = tk.unwrap_or_else(|| self.lexer.next_useful());

            dimensions.push(Dimension { size, attributes });
        }

        Ok(if dimensions.is_empty() {
            (Some(tok), None)
        } else {
            (
                Some(tok),
                Some(Array {
                    base: None,
                    dimensions,
                }),
            )
        })
    }
}
