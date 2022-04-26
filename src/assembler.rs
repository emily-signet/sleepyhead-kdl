#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::borrow::Cow;

#[cfg(all(not(feature = "alloc"), feature = "std"))]
use std::borrow::Cow;

use crate::ast::*;
use crate::lex::Token;
use crate::parser::Parser;
use crate::{KdlEvent, ParseResult};

/// An assembled KdlNode.
#[derive(Debug, Clone)]
pub struct KdlNode<'a> {
    pub name: Cow<'a, str>,
    pub attrs: Vec<KdlProperty<'a>>,
    pub values: Vec<TypedValue<'a>>,
    pub children: Vec<KdlNode<'a>>,
}

/// Parses a document into a vector of it's top-level nodes.
pub fn parse_document<'a, T: Iterator<Item = Token<'a>>>(
    parser: &mut Parser<'a, T>,
) -> ParseResult<Vec<KdlNode<'a>>> {
    let mut output = Vec::new();
    add_children(parser, &mut output)?;
    Ok(output)
}

/// Adds children nodes to a vector, consuming events from a parser.
pub(crate) fn add_children<'a, T: Iterator<Item = Token<'a>>>(
    parser: &mut Parser<'a, T>,
    children: &mut Vec<KdlNode<'a>>,
) -> ParseResult<()> {
    while let Some(next_event) = parser.next() {
        let next_event = next_event?;
        match next_event {
            KdlEvent::NodeOpen {
                name,
                attrs,
                values,
                has_children,
            } => {
                let mut child = KdlNode {
                    name: name.unescape()?,
                    children: Vec::new(),
                    attrs,
                    values,
                };

                if has_children {
                    add_children(parser, &mut child.children)?;
                }

                children.push(child);
            }
            KdlEvent::BracketedNodeClose(_) => return Ok(()),
            KdlEvent::NodeClose(_) => continue,
        }
    }

    Ok(())
}
