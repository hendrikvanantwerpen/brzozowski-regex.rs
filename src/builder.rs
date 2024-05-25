//! Regular expressions and their builders.

use std::fmt::Debug;
use std::hash::Hash;

use crate::Alphabet;

mod pure;
mod similarity;

pub use pure::Pure;
pub use similarity::ApproximatelySimilarCanonical;

/// The recommended regular expression builder.
pub type Default<S> = ApproximatelySimilarCanonical<S>;

/// Constructor methods for regular expressions.
pub trait Builder: Eq + Hash + Sized {
    type Symbol: Alphabet;

    fn empty_set() -> Regex<Self>;
    fn empty_string() -> Regex<Self>;
    fn symbol(value: Self::Symbol) -> Regex<Self>;
    fn closure(inner: Regex<Self>) -> Regex<Self>;
    fn concat(left: Regex<Self>, right: Regex<Self>) -> Regex<Self>;
    fn or(left: Regex<Self>, right: Regex<Self>) -> Regex<Self>;
    fn and(left: Regex<Self>, right: Regex<Self>) -> Regex<Self>;
    fn complement(inner: Regex<Self>) -> Regex<Self>;
}

/// Data type describing regular expressions over values of type S.
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Regex<B: Builder> {
    EmptySet,
    EmptyString,
    Symbol(B::Symbol),
    Concat(Box<Self>, Box<Self>),
    Closure(Box<Self>),
    Or(Box<Self>, Box<Self>),
    And(Box<Self>, Box<Self>),
    Complement(Box<Self>),
}

impl<B: Builder> Regex<B> {
    #[inline]
    pub fn empty_set() -> Self {
        B::empty_set()
    }

    #[inline]
    pub fn empty_string() -> Self {
        B::empty_string()
    }

    #[inline]
    pub fn symbol(value: B::Symbol) -> Self {
        B::symbol(value)
    }

    #[inline]
    pub fn closure(inner: Self) -> Self {
        B::closure(inner)
    }

    #[inline]
    pub fn concat(left: Self, right: Self) -> Self {
        B::concat(left, right)
    }

    #[inline]
    pub fn or(left: Self, right: Self) -> Self {
        B::or(left, right)
    }

    #[inline]
    pub fn and(left: Self, right: Self) -> Self {
        B::and(left, right)
    }

    #[inline]
    pub fn complement(inner: Self) -> Self {
        B::complement(inner)
    }
}

impl<B: Builder> Regex<B> {
    /// Rebuild this regular expression using a different builder over the same symbol type.
    pub fn rebuild<X: Builder<Symbol = B::Symbol>>(&self) -> Regex<X> {
        match self {
            Regex::EmptySet => X::empty_set(),
            Regex::EmptyString => X::empty_string(),
            Regex::Symbol(value) => X::symbol(value.clone()),
            Regex::Concat(left, right) => X::concat(left.rebuild(), right.rebuild()),
            Regex::Closure(inner) => X::closure(inner.rebuild()),
            Regex::Or(left, right) => X::or(left.rebuild(), right.rebuild()),
            Regex::And(left, right) => X::and(left.rebuild(), right.rebuild()),
            Regex::Complement(inner) => X::complement(inner.rebuild()),
        }
    }
}

impl<B: Builder> Clone for Regex<B> {
    fn clone(&self) -> Self {
        self.rebuild()
    }
}
