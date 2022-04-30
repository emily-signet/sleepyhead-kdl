use crate::ast::*;
use crate::lex::Token;
use crate::*;
use logos::Logos;

macro_rules! peek {
    ($parser:expr, $token_kind:pat) => {
        if let Some($token_kind) = $parser.inner.peek() {
            true
        } else {
            false
        }
    };
    ($parser:expr, KdlValues) => {
        peek!(
            $parser,
            Token::Integer(_)
                | Token::StringWithEscapes(_)
                | Token::StringWithNoEscapes(_)
                | Token::Float(_)
                | Token::True
                | Token::False
                | Token::Null
        )
    };
}

macro_rules! next_if {
    ($parser:expr, $token_kind:pat) => {
        #[allow(unreachable_patterns)]
        {
            $parser.inner.next_if(|v| match *v {
                $token_kind => true,
                _ => false,
            })
        }
    };
    (ret $val:tt; $parser:expr, $token_kind:pat, $extractor:pat) => {
        $parser
            .inner
            .next_if(|v| match *v {
                $token_kind => true,
                _ => false,
            })
            .map(|v| match v {
                $extractor => $val,
                _ => unreachable!(),
            })
    };
    ($parser:expr, KdlValues) => {
        next_if!(
            $parser,
            Token::Integer(_)
                | Token::StringWithEscapes(_)
                | Token::StringWithNoEscapes(_)
                | Token::Float(_)
                | Token::True
                | Token::False
                | Token::Null
        )
    };
    (ret IdentOrStr; $parser:expr) => {
        $parser
            .inner
            .next_if(|v| match *v {
                Token::StringWithEscapes(_)
                | Token::StringWithNoEscapes(_)
                | Token::Identifier(_) => true,
                _ => false,
            })
            .map(|v| match v {
                Token::StringWithEscapes(s) => KdlString::Escaped(s),
                Token::StringWithNoEscapes(s) | Token::Identifier(s) => KdlString::Escapeless(s),
                _ => unreachable!(),
            })
    };
}

// Only use this if you're sure you have a value token!
macro_rules! token_to_value {
    ($token:expr) => {
        match $token {
            Token::Integer(i) => KdlValue::Integer(i),
            Token::StringWithEscapes(s) => KdlValue::String(KdlString::Escaped(s)),
            Token::StringWithNoEscapes(s) => KdlValue::String(KdlString::Escapeless(s)),
            Token::Float(f) => KdlValue::Float(f),
            Token::True => KdlValue::Bool(true),
            Token::False => KdlValue::Bool(false),
            Token::Null => KdlValue::Null,
            _ => unreachable!(),
        }
    };
}

macro_rules! slash_dash {
    ($slash_var:ident, $if_not:block) => {
        if $slash_var {
            $slash_var = false;
        } else {
            $if_not
        }
    };
}

/// KDL parser! Acts as an iterator over [KdlEvent]s.
pub struct Parser<'input, T: Iterator<Item = Token<'input>>> {
    inner: core::iter::Peekable<T>,
    nodes_to_close: heapless::Deque<KdlString<'input>, 256>,
    bracketed_nodes_to_close: heapless::Deque<KdlString<'input>, 256>,
}

impl<'input> Parser<'input, logos::Lexer<'input, Token<'input>>> {
    /// Builds a parser from an str, using the default lexer.
    pub fn from_str(to_parse: &'input str) -> Parser<'input, logos::Lexer<'input, Token<'input>>> {
        Parser {
            inner: Token::lexer(to_parse).peekable(),
            nodes_to_close: heapless::Deque::new(),
            bracketed_nodes_to_close: heapless::Deque::new(),
        }
    }
}

impl<'input, T: Iterator<Item = Token<'input>>> Parser<'input, T> {
    /// Build a parser from a lexer / some source of tokens.
    pub fn new(inner: T) -> Parser<'input, T> {
        Parser {
            inner: inner.peekable(),
            nodes_to_close: heapless::Deque::new(),
            bracketed_nodes_to_close: heapless::Deque::new(),
        }
    }

    #[allow(non_snake_case, unused_variables, unused_must_use)]
    fn node_open(&mut self) -> ParseResult<KdlEvent<'input>> {
        let name = next_if!(ret IdentOrStr; self).ok_or(ParseError::NotANode)?;

        let mut attrs: Container<KdlProperty<'input>> = Container::new();
        let mut values: Container<TypedValue<'input>> = Container::new();
        let mut has_children = false;
        let mut slash_dashed = false;

        while let Some(next_token) = self.inner.peek() {
            match next_token {
                Token::BlockOpen => {
                    self.inner.next();
                    if slash_dashed {
                        'inner: while let Some(next_one) = self.inner.next() {
                            if next_one == Token::BlockClose {
                                break 'inner;
                            }
                        }

                    } else {
                        next_if!(self, Token::Newline);
                        has_children = true;
                    }

                    break;
                }
                Token::Backslash => {
                    self.inner.next();
                    if peek! { self, Token::Newline } {
                        self.inner.next();
                    }
                    continue;
                }
                Token::Newline | Token::Semicolon => {
                    break;
                }
                Token::Identifier(_)
                | Token::StringWithEscapes(_)
                | Token::StringWithNoEscapes(_) => {
                    let mut is_ident = false;
                    let ident = match self.inner.next() {
                        Some(v) => match v {
                            Token::Identifier(s) => {
                                is_ident = true;
                                KdlString::Escapeless(s)
                            }
                            Token::StringWithNoEscapes(s) => KdlString::Escapeless(s),
                            Token::StringWithEscapes(s) => KdlString::Escaped(s),
                            _ => unreachable!(),
                        },
                        None => unreachable!(),
                    };

                    if peek!(self, Token::Equals) {
                        let property = self.property(ident)?;

                        slash_dash!(slash_dashed, {
                            attrs.push(property);
                        });
                    } else if !is_ident {
                        slash_dash!(slash_dashed, {
                            values.push(TypedValue {
                                ty: None,
                                val: KdlValue::String(ident),
                            });
                        });
                    }
                }
                Token::TyDescriptor(_) => {
                    let name = match self.inner.next().unwrap() {
                        Token::TyDescriptor(n) => n,
                        _ => unreachable!(),
                    };

                    let val =
                        next_if!(self, KdlValues).ok_or(ParseError::TypeDescriptorWithNoValue)?;

                    slash_dash!(slash_dashed, {
                        values.push(TypedValue {
                            ty: Some(name),
                            val: token_to_value!(val),
                        });
                    });
                }
                Token::Integer(_) | Token::Float(_) | Token::True | Token::False | Token::Null => {
                    let val = self.inner.next().unwrap();

                    slash_dash!(slash_dashed, {
                        values.push(TypedValue {
                            ty: None,
                            val: token_to_value!(val),
                        });
                    });
                }
                Token::SlashDash => {
                    slash_dashed = true;
                    self.inner.next();
                    continue;
                }
                _ => return Err(ParseError::NotANode),
            }
        }

        if !has_children {
            self.nodes_to_close.push_back(name);
        } else {
            self.bracketed_nodes_to_close.push_back(name);
        }

        Ok(KdlEvent::NodeOpen {
            name,
            values,
            attrs,
            has_children,
        })
    }

    #[allow(unused_variables, non_snake_case)]
    fn property(&mut self, ident: KdlString<'input>) -> ParseResult<KdlProperty<'input>> {
        self.inner.next(); // the only invocation of this checks if we have an Equals, so it's safe to just assume that!

        let mut ty_desc = None;

        let value = match self.inner.next().ok_or(ParseError::IncompleteProperty)? {
            Token::TyDescriptor(desc) => {
                ty_desc = Some(desc);
                token_to_value!(next_if!(self, KdlValues).ok_or(ParseError::IncompleteProperty)?)
            }
            Token::Integer(i) => KdlValue::Integer(i),
            Token::StringWithEscapes(s) => KdlValue::String(KdlString::Escaped(s)),
            Token::StringWithNoEscapes(s) => KdlValue::String(KdlString::Escapeless(s)),
            Token::Float(f) => KdlValue::Float(f),
            Token::True => KdlValue::Bool(true),
            Token::False => KdlValue::Bool(false),
            Token::Null => KdlValue::Null,
            _ => return Err(ParseError::IncompleteProperty),
        };

        Ok(KdlProperty {
            key: ident,
            value: TypedValue {
                ty: ty_desc,
                val: value,
            },
        })
    }
}

impl<'input, T: Iterator<Item = Token<'input>>> Iterator for Parser<'input, T> {
    type Item = ParseResult<KdlEvent<'input>>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.inner.peek() {
            match next {
                Token::BlockClose => {
                    self.inner.next();

                    if let Some(to_close) = self.bracketed_nodes_to_close.pop_back() {
                        return Some(Ok(KdlEvent::BracketedNodeClose(to_close)));
                    } else {
                        return Some(Err(ParseError::MismatchedNodeClosing));
                    }
                }
                Token::Semicolon => {
                    self.inner.next();

                    if let Some(to_close) = self.nodes_to_close.pop_back() {
                        return Some(Ok(KdlEvent::NodeClose(to_close)));
                    } else {
                        return Some(Err(ParseError::MismatchedNodeClosing));
                    }
                }
                Token::Newline => {
                    self.inner.next();

                    if let Some(to_close) = self.nodes_to_close.pop_back() {
                        return Some(Ok(KdlEvent::NodeClose(to_close)));
                    } else {
                        continue;
                    }
                }
                Token::SlashDash => {
                    self.inner.next();

                    if let Err(e) = self.node_open() {
                        return Some(Err(e));
                    } else {
                        continue;
                    }
                }
                _ => return Some(self.node_open()),
            }
        }

        self.nodes_to_close
            .pop_back()
            .map(|to_close| Ok(KdlEvent::NodeClose(to_close)))
    }
}
