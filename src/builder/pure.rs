use std::marker::PhantomData;

use crate::builder::Builder;
use crate::builder::Regex;
use crate::Alphabet;

/// A pure regular expression builder that keeps the structure of the
/// constructor calls in the result
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Pure<S: Alphabet> {
    _phantom: PhantomData<S>,
}

impl<S: Alphabet> Builder for Pure<S> {
    type Symbol = S;

    #[inline]
    fn empty_set() -> Regex<Self> {
        Regex::EmptySet
    }

    #[inline]
    fn empty_string() -> Regex<Self> {
        Regex::EmptyString
    }

    #[inline]
    fn symbol(value: S) -> Regex<Self> {
        Regex::Symbol(value)
    }

    #[inline]
    fn closure(inner: Regex<Self>) -> Regex<Self> {
        Regex::Closure(inner.into())
    }

    #[inline]
    fn concat(left: Regex<Self>, right: Regex<Self>) -> Regex<Self> {
        Regex::Concat(left.into(), right.into())
    }

    #[inline]
    fn or(left: Regex<Self>, right: Regex<Self>) -> Regex<Self> {
        Regex::Or(left.into(), right.into())
    }

    #[inline]
    fn and(left: Regex<Self>, right: Regex<Self>) -> Regex<Self> {
        Regex::And(left.into(), right.into())
    }

    #[inline]
    fn complement(inner: Regex<Self>) -> Regex<Self> {
        Regex::Complement(inner.into())
    }
}
