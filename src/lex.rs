use logos::Logos;

#[derive(Logos, Debug, Clone, Copy, PartialEq)]
pub enum Token<'input> {
    #[token("{")]
    BlockOpen,
    #[token("}")]
    BlockClose,
    #[token("(")]
    ParenOpen,
    #[token(")")]
    ParenClose,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("=")]
    Equals,
    #[token("/-")]
    SlashDash,
    #[token(";")]
    Semicolon,
    #[token("null")]
    Null,
    #[token("\\")]
    Backslash,
    #[regex(r#"[\u000D\u000A\u0085\u000C\u2028\u2029]+"#)]
    Newline,
    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#, parsers::parse_str)]
    StringWithEscapes(&'input str),
    #[regex("r#*\"", parsers::parse_raw_string)]
    #[regex(r#""[^"\\]*""#, priority = 5, callback = parsers::parse_str)]
    StringWithNoEscapes(&'input str),
    #[regex(
        r"[0-9]*\.[0-9]+([eE][+-]?[0-9]+)?|[0-9]+[eE][+-]?[0-9]+",
        parsers::float
    )]
    Float(f64),
    #[regex(r"0[xX][0-9a-fA-F_]+", parsers::hex)]
    #[regex(r"0o[01234567_]+", parsers::oct)]
    #[regex(r"0b[10_]+", parsers::bin)]
    #[regex(r"[\d_]+", priority = 2, callback = parsers::int)]
    Integer(i64),
    #[regex(
        r#"\([^0-9\x00-\x20/\\(){}<>;\[\]=,"]+[^\x00-\x20/\\(){}<>;\[\]=,"]*\)"#,
        parsers::parse_str
    )]
    TyDescriptor(&'input str),
    #[regex(r#"[^0-9\x00-\x20/\\(){}<>;\[\]=,"\u000D\u000A\u0085\u000C\u2028\u2029\u0009\u0020\u00A0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200A\u202F\u205F\u3000]+[^\x00-\x20/\\(){}<>;\[\]=,"\u000D\u000A\u0085\u000C\u2028\u2029\u0009\u0020\u00A0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200A\u202F\u205F\u3000]*"#, callback = |lex| lex.slice())]
    Identifier(&'input str),
    #[error]
    #[regex(r"[\u0009\u0020\u00A0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200A\u202F\u205F\u3000]+", logos::skip)]
    #[token("/*", parsers::multiline_comment)] // c-style multiline comments
    #[regex(r"//[^\n]*\n", logos::skip)] // c-style single-line comments
    Error,
}

pub(crate) mod parsers {
    use super::Token;
    use core::num::NonZeroU8;
    use lexical::format::NumberFormatBuilder;
    use logos::Lexer;
    use memchr::memmem;

    const DEC_FORMAT: u128 = lexical::format::RUST_LITERAL;
    const HEX_FORMAT: u128 = NumberFormatBuilder::new()
        .digit_separator(NonZeroU8::new(b'_'))
        .required_digits(true)
        .no_positive_mantissa_sign(true)
        .no_special(true)
        .internal_digit_separator(true)
        .trailing_digit_separator(true)
        .consecutive_digit_separator(true)
        .radix(16)
        .build();
    const OCT_FORMAT: u128 = NumberFormatBuilder::new()
        .digit_separator(NonZeroU8::new(b'_'))
        .required_digits(true)
        .no_positive_mantissa_sign(true)
        .no_special(true)
        .internal_digit_separator(true)
        .trailing_digit_separator(true)
        .consecutive_digit_separator(true)
        .radix(8)
        .build();
    const BIN_FORMAT: u128 = NumberFormatBuilder::new()
        .digit_separator(NonZeroU8::new(b'_'))
        .required_digits(true)
        .no_positive_mantissa_sign(true)
        .no_special(true)
        .internal_digit_separator(true)
        .trailing_digit_separator(true)
        .consecutive_digit_separator(true)
        .radix(2)
        .build();

    pub(crate) fn float<'input>(lex: &mut Lexer<'input, Token<'input>>) -> Option<f64> {
        lexical::parse_with_options::<f64, _, DEC_FORMAT>(
            &lex.slice(),
            &lexical::ParseFloatOptions::new(),
        )
        .ok()
    }

    pub(crate) fn int<'input>(lex: &mut Lexer<'input, Token<'input>>) -> Option<i64> {
        lexical::parse_with_options::<i64, _, DEC_FORMAT>(
            &lex.slice(),
            &lexical::ParseIntegerOptions::new(),
        )
        .ok()
    }

    pub(crate) fn hex<'input>(lex: &mut Lexer<'input, Token<'input>>) -> Option<i64> {
        lexical::parse_with_options::<i64, _, HEX_FORMAT>(
            &lex.slice()[2..],
            &lexical::ParseIntegerOptions::new(),
        )
        .ok()
    }

    pub(crate) fn oct<'input>(lex: &mut Lexer<'input, Token<'input>>) -> Option<i64> {
        lexical::parse_with_options::<i64, _, OCT_FORMAT>(
            &lex.slice()[2..],
            &lexical::ParseIntegerOptions::new(),
        )
        .ok()
    }

    pub(crate) fn bin<'input>(lex: &mut Lexer<'input, Token<'input>>) -> Option<i64> {
        lexical::parse_with_options::<i64, _, BIN_FORMAT>(
            &lex.slice()[2..],
            &lexical::ParseIntegerOptions::new(),
        )
        .ok()
    }

    // from logos source code
    pub(crate) fn parse_raw_string<'input>(
        lexer: &mut Lexer<'input, Token<'input>>,
    ) -> Option<&'input str> {
        let q_hashes = concat!('"', "######", "######", "######", "######", "######");
        let closing = &q_hashes[..lexer.slice().len() - 1]; // skip initial 'r'

        memmem::find(lexer.remainder().as_bytes(), closing.as_bytes()).map(|i| {
            let s = &lexer.remainder()[..i];
            lexer.bump(i + closing.len());
            s
        })
    }

    pub(crate) fn parse_str<'input>(lexer: &mut Lexer<'input, Token<'input>>) -> &'input str {
        let slice = lexer.slice();
        &slice[1..slice.len() - 1]
    }

    pub(crate) fn multiline_comment<'input>(
        lexer: &mut Lexer<'input, Token<'input>>,
    ) -> logos::Skip {
        if let Some(idx) = memmem::find(lexer.remainder().as_bytes(), b"*/") {
            lexer.bump(idx + 2);
        }

        logos::Skip
    }
}
