// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use termcolor::StandardStreamLock;

use super::{Statement, StatementParser};
use crate::lexer::lexer::{TLexer, Token};
use crate::parser::attributes::Attributes;
use crate::parser::dump::Dump;
use crate::parser::errors::ParserError;
use crate::parser::expressions::{ExprNode, ExpressionParser};
use crate::parser::{Context, ScopeKind};

#[derive(Clone, Debug, PartialEq)]
pub struct Do {
    pub attributes: Option<Attributes>,
    pub body: Statement,
    pub condition: ExprNode,
}

impl Dump for Do {
    fn dump(&self, name: &str, prefix: &str, last: bool, stdout: &mut StandardStreamLock) {
        dump_obj!(self, name, "do", prefix, last, stdout, attributes, body, condition);
    }
}

pub struct DoStmtParser<'a, L: TLexer> {
    lexer: &'a mut L,
}

impl<'a, L: TLexer> DoStmtParser<'a, L> {
    pub(super) fn new(lexer: &'a mut L) -> Self {
        Self { lexer }
    }

    pub(super) fn parse(
        self,
        attributes: Option<Attributes>,
        context: &mut Context,
    ) -> Result<(Option<Token>, Option<Do>), ParserError> {
        context.set_current(None, ScopeKind::DoBlock);
        let sp = StatementParser::new(self.lexer);
        let (tok, body) = sp.parse(None, context)?;
        context.pop();

        let tok = tok.unwrap_or_else(|| self.lexer.next_useful());
        if tok != Token::While {
            return Err(ParserError::InvalidTokenInDo {
                sp: self.lexer.span(),
                tok,
            });
        }

        let tok = self.lexer.next_useful();
        if tok != Token::LeftParen {
            return Err(ParserError::InvalidTokenInDo {
                sp: self.lexer.span(),
                tok,
            });
        }

        let mut ep = ExpressionParser::new(self.lexer, Token::RightParen);
        let (tok, condition) = ep.parse(None, context)?;

        let tok = tok.unwrap_or_else(|| self.lexer.next_useful());
        if tok != Token::RightParen {
            return Err(ParserError::InvalidTokenInDo {
                sp: self.lexer.span(),
                tok,
            });
        }

        let tok = self.lexer.next_useful();
        match tok {
            Token::SemiColon => Ok((
                None,
                Some(Do {
                    attributes,
                    body: body.unwrap(),
                    condition: condition.unwrap(),
                }),
            )),
            _ => Err(ParserError::InvalidTokenInDo {
                sp: self.lexer.span(),
                tok,
            }),
        }
    }
}
