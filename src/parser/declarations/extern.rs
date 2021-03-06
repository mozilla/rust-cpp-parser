// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::rc::Rc;
use termcolor::StandardStreamLock;

use super::{
    DeclHint, Declaration, DeclarationListParser, Declarations, Specifier, TypeDeclaratorParser,
};
use crate::lexer::lexer::{TLexer, Token};
use crate::parser::dump::Dump;
use crate::parser::errors::ParserError;
use crate::parser::Context;

#[derive(Clone, Debug, PartialEq)]
pub struct Extern {
    pub language: String,
    pub decls: Declarations,
    pub multiple: bool,
}

impl Dump for Extern {
    fn dump(&self, name: &str, prefix: &str, last: bool, stdout: &mut StandardStreamLock) {
        dump_obj!(self, name, "extern", prefix, last, stdout, language, decls, multiple);
    }
}

pub(crate) struct ExternParser<'a, L: TLexer> {
    lexer: &'a mut L,
}

impl<'a, L: TLexer> ExternParser<'a, L> {
    pub(super) fn new(lexer: &'a mut L) -> Self {
        Self { lexer }
    }

    pub(super) fn parse(
        self,
        tok: Option<Token>,
        context: &mut Context,
    ) -> Result<(Option<Token>, Option<Declaration>), ParserError> {
        let tok = tok.unwrap_or_else(|| self.lexer.next_useful());
        if tok != Token::Extern {
            return Ok((Some(tok), None));
        }

        let tok = self.lexer.next_useful();

        if let Token::LiteralString(language) = tok {
            let tok = self.lexer.next_useful();
            let has_brace = tok == Token::LeftBrace;
            let dlp = DeclarationListParser::new(self.lexer);

            let (tok, list) = dlp.parse(None, context)?;

            if has_brace {
                let tok = tok.unwrap_or_else(|| self.lexer.next_useful());
                if tok == Token::RightBrace {
                    Ok((
                        None,
                        Some(Declaration::Extern(Extern {
                            language,
                            decls: list.unwrap(),
                            multiple: true,
                        })),
                    ))
                } else {
                    Err(ParserError::InvalidTokenInExtern {
                        sp: self.lexer.span(),
                        tok,
                    })
                }
            } else {
                Ok((
                    tok,
                    Some(Declaration::Extern(Extern {
                        language,
                        decls: list.unwrap(),
                        multiple: false,
                    })),
                ))
            }
        } else {
            let tdp = TypeDeclaratorParser::new(self.lexer);
            let hint = DeclHint::Specifier(Specifier::EXTERN);
            let (tok, typ) = tdp.parse(Some(tok), Some(hint), true, context)?;
            let typ = typ.unwrap();
            context.add_type_decl(Rc::clone(&typ));

            Ok((tok, Some(Declaration::Type(typ))))
        }
    }
}

#[cfg(test)]
mod tests {

    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::lexer::{preprocessor::context::DefaultContext, Lexer};
    use crate::parser::declarations::{types, *};
    use crate::parser::names::*;
    use crate::parser::types::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_extern_c() {
        let mut l = Lexer::<DefaultContext>::new(
            br#"
extern "C" {
    double sqrt(double);
}
        "#,
        );
        let p = ExternParser::new(&mut l);
        let mut context = Context::default();
        let (_, ext) = p.parse(None, &mut context).unwrap();

        let ext = ext.unwrap();

        assert_eq!(
            ext,
            Declaration::Extern(Extern {
                language: "C".to_string(),
                decls: vec![Declaration::Type(Rc::new(TypeDeclarator {
                    typ: Type {
                        base: BaseType::Function(Box::new(Function {
                            return_type: Some(Type {
                                base: BaseType::Primitive(Primitive::Double),
                                cv: CVQualifier::empty(),
                                pointers: None,
                            }),
                            params: vec![Parameter {
                                attributes: None,
                                decl: Rc::new(TypeDeclarator {
                                    typ: Type {
                                        base: BaseType::Primitive(Primitive::Double),
                                        cv: CVQualifier::empty(),
                                        pointers: None,
                                    },
                                    specifier: Specifier::empty(),
                                    identifier: types::Identifier {
                                        identifier: None,
                                        attributes: None
                                    },
                                    init: None,
                                    bitfield_size: None,
                                }),
                            }],
                            cv: CVQualifier::empty(),
                            refq: RefQualifier::None,
                            except: None,
                            attributes: None,
                            trailing: None,
                            virt_specifier: VirtSpecifier::empty(),
                            status: FunStatus::None,
                            requires: None,
                            ctor_init: None,
                            body: RefCell::new(None)
                        })),
                        cv: CVQualifier::empty(),
                        pointers: None,
                    },
                    specifier: Specifier::empty(),
                    identifier: types::Identifier {
                        identifier: Some(mk_id!("sqrt")),
                        attributes: None
                    },
                    init: None,
                    bitfield_size: None,
                }))],
                multiple: true,
            })
        );
    }

    #[test]
    fn test_extern_decl() {
        let mut l = Lexer::<DefaultContext>::new(
            br#"
extern double sqrt(double);
        "#,
        );
        let p = ExternParser::new(&mut l);
        let mut context = Context::default();
        let (_, ext) = p.parse(None, &mut context).unwrap();

        let ext = ext.unwrap();

        assert_eq!(
            ext,
            Declaration::Type(Rc::new(TypeDeclarator {
                typ: Type {
                    base: BaseType::Function(Box::new(Function {
                        return_type: Some(Type {
                            base: BaseType::Primitive(Primitive::Double),
                            cv: CVQualifier::empty(),
                            pointers: None,
                        }),
                        params: vec![Parameter {
                            attributes: None,
                            decl: Rc::new(TypeDeclarator {
                                typ: Type {
                                    base: BaseType::Primitive(Primitive::Double),
                                    cv: CVQualifier::empty(),
                                    pointers: None,
                                },
                                specifier: Specifier::empty(),
                                identifier: types::Identifier {
                                    identifier: None,
                                    attributes: None
                                },
                                init: None,
                                bitfield_size: None,
                            }),
                        }],
                        cv: CVQualifier::empty(),
                        refq: RefQualifier::None,
                        except: None,
                        attributes: None,
                        trailing: None,
                        virt_specifier: VirtSpecifier::empty(),
                        status: FunStatus::None,
                        requires: None,
                        ctor_init: None,
                        body: RefCell::new(None)
                    })),
                    cv: CVQualifier::empty(),
                    pointers: None,
                },
                specifier: Specifier::EXTERN,
                identifier: types::Identifier {
                    identifier: Some(mk_id!("sqrt")),
                    attributes: None
                },
                init: None,
                bitfield_size: None,
            }))
        );
    }
}
