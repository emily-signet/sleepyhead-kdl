#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

/// Alias for a container; either alloc::Vec, std::Vec, or heapless::Vec
#[cfg(feature = "std")]
pub type Container<A> = std::vec::Vec<A>;

/// Alias for a container; either alloc::Vec, std::Vec, or heapless::Vec
#[cfg(all(feature = "alloc", not(feature = "std")))]
pub type Container<A> = alloc::vec::Vec<A>;

/// Alias for a container; either alloc::Vec, std::Vec, or heapless::Vec
#[cfg(all(not(feature = "alloc"), not(feature = "std")))]
pub type Container<A> = heapless::Vec<A, 128>;

/// A parser error.
#[derive(Debug, Copy, Clone)]
pub enum ParseError {
    IncompleteProperty,
    MismatchedNodeClosing,
    NotANode,
    UnrecognizedEscape,
    UnexpectedEOF,
    BadUnicodeEscape,
    TypeDescriptorWithNoValue,
}

/// Result alias.
pub type ParseResult<T> = core::result::Result<T, ParseError>;

/// utils to assemble a series of events into [KdlNode]s
#[cfg(any(feature = "alloc", feature = "std"))]
pub mod assembler;
/// AST types; [ast::KdlValue] and [ast::KdlString]
pub mod ast;
/// default kdl lexer
pub mod lex;
/// the kdl parser!
pub mod parser;
/// utils for processing string escapes
pub mod unescape;

use ast::*;

/// An event emitted during parsing; either the opening or closing of a node.
#[derive(Debug, Clone)]
pub enum KdlEvent<'input> {
    /// Start of a node; contains it's name, properties/attributes, values, and whether ot not it has children.
    NodeOpen {
        name: KdlString<'input>,
        attrs: Container<KdlProperty<'input>>,
        values: Container<TypedValue<'input>>,
        has_children: bool,
    },
    /// End of a childless node.
    NodeClose(KdlString<'input>),
    /// End of a node that had children / a children block ({}).
    BracketedNodeClose(KdlString<'input>),
}
