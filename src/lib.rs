#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
pub type Container<A> = std::vec::Vec<A>;

#[cfg(all(feature = "alloc", not(feature = "std")))]
pub type Container<A> = alloc::vec::Vec<A>;

#[cfg(all(not(feature = "alloc"), not(feature = "std")))]
pub type Container<A> = heapless::Vec<A, 256>;

pub mod ast;
pub mod lex;
