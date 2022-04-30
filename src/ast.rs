use crate::unescape::{self, EscapingIter};
use crate::*;

#[cfg(not(feature = "std"))]
use core::fmt;

#[cfg(feature = "std")]
use std::fmt;

#[cfg(all(feature = "std", not(feature = "alloc")))]
use std::borrow::Cow;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::borrow::Cow;

/// A kdl value with an optional type annotation
#[derive(Debug, Copy, Clone)]
pub struct TypedValue<'a> {
    pub ty: Option<&'a str>,
    pub val: KdlValue<'a>,
}

impl<'a> PartialEq for TypedValue<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}

impl<'a> fmt::Display for TypedValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(t) = self.ty {
            write!(f, "({}){}", t, self.val)
        } else {
            write!(f, "{}", self.val)
        }
    }
}

/// A kdl Value; either a string, integer (represented as i64), float (represented as f64), bool, or null.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KdlValue<'a> {
    String(KdlString<'a>),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Null,
}

impl<'a> KdlValue<'a> {
    pub fn as_kdlstring(&self) -> Option<&KdlString<'a>> {
        if let KdlValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn as_str(&self) -> Option<Cow<'a, str>> {
        if let KdlValue::String(s) = self {
            s.unescape().ok()
        } else {
            None
        }
    }

    pub fn as_int(&self) -> Option<&i64> {
        if let KdlValue::Integer(i) = self {
            Some(i)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<&f64> {
        if let KdlValue::Float(f) = self {
            Some(f)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<&bool> {
        if let KdlValue::Bool(b) = self {
            Some(b)
        } else {
            None
        }
    }
}

impl<'a> fmt::Display for KdlValue<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use KdlValue::*;
        match self {
            String(s) => write!(f, "{}", s),
            Integer(i) => write!(f, "{}", i),
            Float(v) => write!(f, "{}", v),
            Bool(b) => write!(f, "{}", b),
            Null => write!(f, "nil"),
        }
    }
}

/// Wrapper type over a string that may or may not contain escapes. This serves to make any escape-processing lazy and allows for a hot path where the string doesn't have any escapes.
/// ## Panics
/// In a no_std build, comparing two KdlStrings can lead to a panic if one of the strings contains invalid escapes.
#[derive(Debug, Copy, Clone)]
pub enum KdlString<'a> {
    Escapeless(&'a str),
    Escaped(&'a str),
}

impl<'a> KdlString<'a> {
    /// Unescapes the string if needed.
    /// returns an Iterator over the chars which lazily processes escape codes.
    /// ## Panics
    /// If the string contains escapes, it will panic at any invalid escape. Use [unescape] if possible.
    pub fn unescape_iter(&self) -> EscapingIter<'a> {
        match self {
            KdlString::Escapeless(s) => EscapingIter::shim(s),
            KdlString::Escaped(s) => EscapingIter::unescape(s),
        }
    }

    /// Unescapes the string if needed.
    /// returns a Cow<'a, str>, which will only be owned if unescaping was needed.
    /// returns an error if any invalid escape was encountered.
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn unescape(&self) -> Result<Cow<'a, str>, ParseError> {
        match self {
            KdlString::Escapeless(s) => Ok(Cow::Borrowed(s)),
            KdlString::Escaped(s) => Ok(Cow::Owned(unescape::unescape_std(s)?)),
        }
    }
}

impl<'a> PartialEq for KdlString<'a> {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            match (self.unescape(), other.unescape()) {
                (Ok(lhs), Ok(rhs)) => lhs == rhs,
                _ => false,
            }
        }

        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        {
            self.unescape_iter().eq(other.unescape_iter())
        }
    }
}

impl<'a> fmt::Display for KdlString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            return match self {
                KdlString::Escapeless(s) => write!(f, "{}", s),
                KdlString::Escaped(s) => {
                    write!(f, "{}", unescape::unescape_std(s).map_err(|_| fmt::Error)?)
                }
            };
        }

        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        {
            return match self {
                KdlString::Escapeless(s) => write!(f, "{}", s),
                KdlString::Escaped(s) => {
                    let mut buf: heapless::String<256> = heapless::String::new();
                    let mut iter = EscapingIter::unescape(s);

                    for next_c in iter {
                        if buf.push(next_c).is_err() {
                            write!(f, "{}", buf)?;
                            buf.clear();
                            buf.push(next_c);
                        }
                    }

                    write!(f, "{}", buf)?;

                    Ok(())
                }
            };
        }
    }
}

/// A kdl property, containing a [key](KdlString) and a [value](KdlValue).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct KdlProperty<'a> {
    pub key: KdlString<'a>,
    pub value: TypedValue<'a>,
}
