use crate::lex::Token;
use crate::Container;

#[derive(Debug, Copy, Clone)]
pub enum ParseError {
    IncompleteProperty,
    MismatchedNodeClosing,
    NotANode,
}

pub type ParseResult<T> = core::result::Result<T, ParseError>;

macro_rules! peek {
    ($parser:expr, $token_kind:pat) => {
        if let Some(v) = $parser.inner.peek() {
            match v {
                $token_kind => true,
                _ => false,
            }
        } else {
            false
        }
    };
    ($parser:expr, KdlValues) => {
        peek!(
            $parser,
            Token::Integer(_)
                | Token::String(_)
                | Token::Float(_)
                | Token::True
                | Token::False
                | Token::Null
        )
    };
}

macro_rules! next_if {
    ($parser:expr, $token_kind:pat) => {
        $parser.inner.next_if(|v| match *v {
            $token_kind => true,
            _ => false,
        })
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
                | Token::String(_)
                | Token::Float(_)
                | Token::True
                | Token::False
                | Token::Null
        )
    };
}

// Only use this if you're sure you have a value token!
macro_rules! token_to_value {
    ($token:expr) => {
        match $token {
            Token::Integer(i) => KdlValue::Integer(i),
            Token::String(s) => KdlValue::String(s),
            Token::Float(f) => KdlValue::Float(f),
            Token::True => KdlValue::Bool(true),
            Token::False => KdlValue::Bool(false),
            Token::Null => KdlValue::Null,
            _ => unreachable!(),
        }
    };
}

#[derive(Debug, Copy, Clone)]
pub enum KdlValue<'a> {
    String(&'a str),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Null,
}

#[derive(Debug, Copy, Clone)]
pub struct KdlProperty<'a> {
    pub key: &'a str,
    pub value: KdlValue<'a>,
}

pub struct Parser<'input, T: Iterator<Item = Token<'input>>> {
    inner: core::iter::Peekable<T>,
    nodes_to_close: heapless::Deque<&'input str, 256>,
    bracketed_nodes_to_close: heapless::Deque<&'input str, 256>
}

#[derive(Debug, Clone)]
pub enum KdlEvent<'input> {
    NodeOpen {
        name: &'input str,
        attrs: Container<KdlProperty<'input>>,
        values: Container<KdlValue<'input>>,
        has_children: bool,
    },
    NodeClose(&'input str),
}

impl<'input, T: Iterator<Item = Token<'input>>> Parser<'input, T> {
    pub fn new(inner: T) -> Parser<'input, T> {
        Parser {
            inner: inner.peekable(),
            nodes_to_close: heapless::Deque::new(),
            bracketed_nodes_to_close: heapless::Deque::new()
        }
    }

    pub fn node_open(&mut self) -> ParseResult<KdlEvent<'input>> {
        let name = next_if!(ret name; self, Token::Identifier(_), Token::Identifier(name))
            .ok_or(ParseError::NotANode)?;

        let mut attrs: Container<KdlProperty<'input>> = Container::new();
        let mut values: Container<KdlValue<'input>> = Container::new();
        let mut has_children: bool = false;

        while let Some(next_token) = self.inner.peek() {
            match next_token {
                Token::BlockOpen => {
                    self.inner.next();
                    next_if!(self, Token::Newline);
                    has_children = true;
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
                Token::Integer(_)
                | Token::String(_)
                | Token::Float(_)
                | Token::True
                | Token::False
                | Token::Null => {
                    let val = self.inner.next().unwrap();
                    values.push(token_to_value!(val));
                }
                Token::Identifier(_) => {
                    let ident = match self.inner.next() {
                        Some(Token::Identifier(ident)) => ident,
                        _ => unreachable!(),
                    };
                    attrs.push(self.property(ident)?);
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

    pub fn property(&mut self, ident: &'input str) -> ParseResult<KdlProperty<'input>> {
        next_if!(self, Token::Equals).ok_or(ParseError::IncompleteProperty)?;
        let value = next_if!(self, KdlValues).ok_or(ParseError::IncompleteProperty)?;

        Ok(KdlProperty {
            key: ident,
            value: token_to_value!(value),
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
                        return Some(Ok(KdlEvent::NodeClose(to_close)))
                    } else {
                        return Some(Err(ParseError::MismatchedNodeClosing))
                    }
                }
                Token::Semicolon => {
                    self.inner.next();
                    
                    if let Some(to_close) = self.nodes_to_close.pop_back() {
                        return Some(Ok(KdlEvent::NodeClose(to_close)))
                    } else {
                        return Some(Err(ParseError::MismatchedNodeClosing))
                    }
                }
                Token::Newline => {
                    self.inner.next();

                    if let Some(to_close) = self.nodes_to_close.pop_back() {
                        return Some(Ok(KdlEvent::NodeClose(to_close)))
                    } else {
                        continue;
                    }
                }
                _ => return Some(self.node_open()),
            }
        }
        
        if let Some(to_close) = self.nodes_to_close.pop_back() {
            Some(Ok(KdlEvent::NodeClose(to_close)))
        } else {
            None
        }
    }
}
