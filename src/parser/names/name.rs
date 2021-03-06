// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use termcolor::StandardStreamLock;

use super::dtor::{Destructor, DtorParser};
use super::operator::{Operator, OperatorParser};
use crate::lexer::{TLexer, Token};
use crate::parser::dump::Dump;
use crate::parser::errors::ParserError;
use crate::parser::expressions::{Parameters, ParametersParser};
use crate::parser::Context;

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Identifier {
    pub val: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    id: Identifier,
    params: Parameters,
    //keyword: bool, TODO: set to true when we've A::template B<...>::...
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum Name {
    Identifier(Identifier),
    Destructor(Destructor),
    //Template(Template),
    Operator(Box<Operator>),
    Empty,
    //Decltype(ExprNode), TODO: add that
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        match self {
            Name::Identifier(id) => &id.val,
            _ => "",
        }
    }
}

impl Eq for Name {}

impl ToString for Name {
    fn to_string(&self) -> String {
        match self {
            Name::Identifier(id) => id.val.clone(),
            //Name::Template(t) => t.id.val.clone(),
            Name::Destructor(d) => format!("~{}", d.name),
            Name::Operator(op) => op.to_string(),
            Name::Empty => "".to_string(),
        }
    }
}

#[macro_export]
macro_rules! mk_id {
    ( $( $name:expr ),* ) => {
        Qualified {
            names: vec![
                $(
                    crate::parser::names::Name::Identifier(crate::parser::names::Identifier { val: $name.to_string()}),
                )*
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Qualified {
    pub names: Vec<Name>,
}

impl ToString for Qualified {
    fn to_string(&self) -> String {
        let mut buf = String::new();
        if let Some((last, names)) = self.names.split_last() {
            for name in names.iter() {
                buf.push_str(&name.to_string());
                buf.push_str("::");
            }
            buf.push_str(&last.to_string());
        }
        buf
    }
}

impl Dump for Qualified {
    fn dump(&self, name: &str, prefix: &str, last: bool, stdout: &mut StandardStreamLock) {
        dump_str!(name, self.to_string(), prefix, last, stdout);
    }
}

impl Qualified {
    pub fn is_conv_op(&self) -> bool {
        if let Name::Operator(op) = self.names.last().unwrap() {
            op.is_conv()
        } else {
            false
        }
    }

    pub fn get_first_name(mut self) -> String {
        if let Name::Identifier(id) = self.names.pop().unwrap() {
            id.val
        } else {
            unreachable!("Not a valid identifier");
        }
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }
}

pub struct QualifiedParser<'a, L: TLexer> {
    lexer: &'a mut L,
}

impl<'a, L: TLexer> QualifiedParser<'a, L> {
    pub(crate) fn new(lexer: &'a mut L) -> Self {
        Self { lexer }
    }

    pub(crate) fn parse(
        self,
        tok: Option<Token>,
        first: Option<String>,
        context: &mut Context,
    ) -> Result<(Option<Token>, Option<Qualified>), ParserError> {
        let mut tok = tok.unwrap_or_else(|| self.lexer.next_useful());
        let mut names = Vec::new();
        let mut wait_id = if let Some(val) = first {
            names.push(Name::Identifier(Identifier { val }));
            false
        } else {
            true
        };

        loop {
            match tok {
                Token::ColonColon => {
                    if names.is_empty() {
                        // ::foo::bar
                        names.push(Name::Empty);
                    }
                    wait_id = true;
                }
                /*Token::Lower => {
                    let id = if let Some(Name::Identifier(id)) = names.pop() {
                        id
                    } else {
                        unreachable!("Cannot have two templates args");
                    };

                    let pp = ParametersParser::new(self.lexer, Token::Greater);
                    let (_, params) = pp.parse(None, None);

                    names.push(Name::Template(Template {
                        id,
                        params: params.unwrap(),
                    }));

                    wait_id = false;
                }*/
                Token::Identifier(val) if wait_id => {
                    names.push(Name::Identifier(Identifier { val }));
                    wait_id = false;
                }
                Token::Identifier(_) if !wait_id => {
                    return Ok((Some(tok), Some(Qualified { names })));
                }
                Token::Operator => {
                    let op = OperatorParser::new(self.lexer);
                    let (tok, operator) = op.parse(Some(tok), context)?;

                    names.push(Name::Operator(Box::new(operator.unwrap())));

                    return Ok((tok, Some(Qualified { names })));
                }
                Token::Tilde => {
                    if wait_id {
                        let dp = DtorParser::new(self.lexer);
                        let (tok, dtor) = dp.parse(Some(tok), context)?;

                        names.push(Name::Destructor(dtor.unwrap()));
                        return Ok((tok, Some(Qualified { names })));
                    } else {
                        return Ok((Some(tok), Some(Qualified { names })));
                    }
                }
                _ => {
                    if names.is_empty() {
                        return Ok((Some(tok), None));
                    } else {
                        return Ok((Some(tok), Some(Qualified { names })));
                    }
                }
            }
            tok = self.lexer.next_useful();
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lexer::{preprocessor::context::DefaultContext, Lexer};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_name_one() {
        let mut l = Lexer::<DefaultContext>::new(b"abc");
        let p = QualifiedParser::new(&mut l);
        let mut context = Context::default();
        let (_, q) = p.parse(None, None, &mut context).unwrap();

        assert_eq!(q.unwrap(), mk_id!("abc"));
    }

    #[test]
    fn test_name_two() {
        let mut l = Lexer::<DefaultContext>::new(b"abc::defg");
        let p = QualifiedParser::new(&mut l);
        let mut context = Context::default();
        let (_, q) = p.parse(None, None, &mut context).unwrap();

        assert_eq!(q.unwrap(), mk_id!("abc", "defg"));
    }

    #[test]
    fn test_name_three() {
        let mut l = Lexer::<DefaultContext>::new(b"abc::defg::hijkl");
        let p = QualifiedParser::new(&mut l);
        let mut context = Context::default();
        let (_, q) = p.parse(None, None, &mut context).unwrap();

        assert_eq!(q.unwrap(), mk_id!("abc", "defg", "hijkl"));
    }

    /*#[test]
    fn test_name_template_zero() {
        let mut l = Lexer::<DefaultContext>::new(b"A<>");
        let p = QualifiedParser::new(&mut l);
        let (_, q) = p.parse(None, None);

        assert_eq!(
            q.unwrap(),
            Qualified {
                names: vec![Name::Template(Template {
                    id: Identifier {
                        val: "A".to_string()
                    },
                    params: vec![],
                }),],
            }
        );
    }

    #[test]
    fn test_name_template_one() {
        let mut l = Lexer::<DefaultContext>::new(b"A<B>");
        let p = QualifiedParser::new(&mut l);
        let (_, q) = p.parse(None, None);

        assert_eq!(
            q.unwrap(),
            Qualified {
                names: vec![Name::Template(Template {
                    id: Identifier {
                        val: "A".to_string()
                    },
                    params: vec![Some(ExprNode::Qualified(Box::new(mk_id!("B")))),],
                }),],
            }
        );
    }

    #[test]
    fn test_name_complex() {
        let mut l = Lexer::<DefaultContext>::new(b"A::B<C::D, E::F, G>::H<I>");
        let p = QualifiedParser::new(&mut l);
        let (_, q) = p.parse(None, None);

        assert_eq!(
            q.unwrap(),
            Qualified {
                names: vec![
                    Name::Identifier(Identifier {
                        val: "A".to_string(),
                    }),
                    Name::Template(Template {
                        id: Identifier {
                            val: "B".to_string()
                        },
                        params: vec![
                            Some(ExprNode::Qualified(Box::new(mk_id!("C", "D")))),
                            Some(ExprNode::Qualified(Box::new(mk_id!("E", "F")))),
                            Some(ExprNode::Qualified(Box::new(mk_id!("G")))),
                        ]
                    }),
                    Name::Template(Template {
                        id: Identifier {
                            val: "H".to_string()
                        },
                        params: vec![Some(ExprNode::Qualified(Box::new(mk_id!("I")))),]
                    })
                ]
            }
        );
    }*/
}
