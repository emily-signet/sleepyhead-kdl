#[cfg(any(feature = "std", feature = "alloc"))]
use crate::ParseError;

pub struct EscapingIter<'a> {
    inner: core::str::Chars<'a>,
    shim: bool,
}

impl<'a> EscapingIter<'a> {
    pub fn unescape(inner: &'a str) -> EscapingIter<'a> {
        EscapingIter {
            inner: inner.chars(),
            shim: false,
        }
    }

    pub fn shim(inner: &'a str) -> EscapingIter<'a> {
        EscapingIter {
            inner: inner.chars(),
            shim: true,
        }
    }
}

impl<'a> Iterator for EscapingIter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        if self.shim {
            return self.inner.next();
        }

        let next_char = self.inner.next()?;

        if next_char == '\\' {
            Some(match self.inner.next()? {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '\\' => '\u{005C}',
                'b' => '\u{0008}',
                'f' => '\u{000C}',
                '/' => '/',
                '"' => '"',
                'u' => {
                    assert_eq!(
                        self.next()?,
                        '{',
                        "invalid unicode escape: needs to start with {{"
                    );
                    let mut codepoint: u32 = 0;

                    let mut idx = 0;
                    loop {
                        if idx > 6 {
                            panic!("unicode escape is too long");
                        }

                        let next_codepoint = match self.next()? as u8 {
                            c @ b'0'..=b'9' => c - b'0',
                            c @ b'a'..=b'f' => c - b'a' + 10,
                            c @ b'A'..=b'F' => c - b'A' + 10,
                            b'}' => break,
                            _ => panic!("invalid character in unicode escape"),
                        };

                        codepoint = codepoint << 4 | ((next_codepoint & 0xF) as u32);
                        idx += 1;
                    }

                    char::from_u32(codepoint)?
                }
                c => c,
            })
        } else {
            Some(next_char)
        }
    }
}

#[cfg(feature = "alloc")]
use alloc::string::String;

#[cfg(any(feature = "std", feature = "alloc"))]
pub(crate) fn unescape_std(s: &str) -> Result<String, ParseError> {
    let mut buf = String::with_capacity(s.len());
    let mut chars = s.chars();

    loop {
        match chars.next() {
            Some('\\') => buf.push(match chars.next().ok_or(ParseError::UnexpectedEOF)? {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '\\' => '\u{005C}',
                'b' => '\u{0008}',
                'f' => '\u{000C}',
                '/' => '/',
                '"' => '"',
                'u' => {
                    match chars.next() {
                        Some('{') => (),
                        _ => return Err(ParseError::BadUnicodeEscape),
                    }

                    let mut codepoint: u32 = 0;

                    let mut idx = 0;
                    loop {
                        if idx > 6 {
                            return Err(ParseError::BadUnicodeEscape);
                        }

                        let next_codepoint =
                            match chars.next().ok_or(ParseError::UnexpectedEOF)? as u8 {
                                c @ b'0'..=b'9' => c - b'0',
                                c @ b'a'..=b'f' => c - b'a' + 10,
                                c @ b'A'..=b'F' => c - b'A' + 10,
                                b'}' => break,
                                _ => return Err(ParseError::BadUnicodeEscape),
                            };

                        codepoint = codepoint << 4 | ((next_codepoint & 0xF) as u32);
                        idx += 1;
                    }

                    char::from_u32(codepoint).ok_or(ParseError::BadUnicodeEscape)?
                }
                _ => return Err(ParseError::UnrecognizedEscape),
            }),
            Some(c) => buf.push(c),
            None => break,
        }
    }

    Ok(buf)
}
