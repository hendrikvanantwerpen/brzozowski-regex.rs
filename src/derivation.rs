//! Derivation and derivation-based matching for regular expressions.

use std::borrow::Borrow;
use std::collections::HashSet;

use itertools::Itertools;

use crate::builder::Builder;
use crate::builder::Regex;
use crate::Alphabet;

impl<B: Builder> Regex<B> {
    /// Returns the derivative of this regular expression w.r.t. the given symbols.
    pub fn derive_iter<I>(&self, symbols: impl IntoIterator<Item = I>) -> Regex<B>
    where
        I: Borrow<B::Symbol>,
    {
        let mut d = self.clone();
        for symbol in symbols {
            d = d.derive(symbol.borrow());
        }
        d
    }

    /// Returns the derivative of this regular expression w.r.t. to the given symbol.
    #[inline]
    pub fn derive(&self, symbol: &B::Symbol) -> Regex<B> {
        self.derive_symbols(&Symbols::include([symbol.clone()]))
    }

    pub(crate) fn derive_symbols(&self, symbol: &Symbols<B::Symbol>) -> Regex<B> {
        match self {
            Self::EmptySet => B::empty_set(),
            Self::EmptyString => B::empty_set(),
            Self::Symbol(inner) => {
                if symbol.matches(inner) {
                    B::empty_string()
                } else {
                    B::empty_set()
                }
            }
            Self::Concat(left, right) => B::or(
                B::concat(left.derive_symbols(symbol), *right.clone()),
                B::concat(left.nullable(), right.derive_symbols(symbol)),
            ),
            Self::Closure(inner) => {
                B::concat(inner.derive_symbols(symbol), B::closure(*inner.clone()))
            }
            Self::Or(left, right) => {
                B::or(left.derive_symbols(symbol), right.derive_symbols(symbol))
            }
            Self::And(left, right) => {
                B::and(left.derive_symbols(symbol), right.derive_symbols(symbol))
            }
            Self::Complement(inner) => B::complement(inner.derive_symbols(symbol)),
        }
    }

    /// Returns whether the string of symbols is in the language of this regular expression.
    pub fn is_match<I>(&self, symbols: impl IntoIterator<Item = I>) -> bool
    where
        I: Borrow<B::Symbol>,
    {
        self.derive_iter(symbols).is_nullable()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Symbols<S: Alphabet> {
    /// Only the given symbols.
    Include(HashSet<S>),
    /// All except the given symbols.
    Exclude(HashSet<S>),
}

impl<S: Alphabet> std::fmt::Display for Symbols<S>
where
    S: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbols::Include(symbols) => write!(f, "{{{}}}", symbols.iter().join(", ")),
            Symbols::Exclude(symbols) => write!(f, "Σ∖{{{}}}", symbols.iter().join(", ")),
        }
    }
}

impl<S: Alphabet> Symbols<S> {
    #[inline]
    pub(crate) fn include<const N: usize>(symbols: [S; N]) -> Self {
        Self::Include(HashSet::from(symbols))
    }

    #[cfg(test)]
    #[inline]
    pub(crate) fn exclude<const N: usize>(symbols: [S; N]) -> Self {
        Self::Exclude(HashSet::from(symbols))
    }

    pub(crate) fn matches(&self, symbol: &S) -> bool {
        match self {
            Self::Include(included) => included.contains(symbol),
            Self::Exclude(excluded) => !excluded.contains(symbol),
        }
    }
}

impl<S: Alphabet> std::ops::BitOr for Symbols<S> {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        // either this or the other regular expressions must match
        match (self, other) {
            // include all included symbols
            (Self::Include(left), Self::Include(right)) => {
                Self::Include(HashSet::union(&left, &right).cloned().collect())
            }
            // exclude shared excluded symbols
            (Self::Exclude(left), Self::Exclude(right)) => {
                Self::Exclude(HashSet::intersection(&left, &right).cloned().collect())
            }
            // exclude the excluded symbols except the included symbols
            (Self::Include(included), Self::Exclude(excluded))
            | (Self::Exclude(excluded), Self::Include(included)) => {
                Self::Exclude(excluded.difference(&included).cloned().collect())
            }
        }
    }
}

impl<S: Alphabet> std::ops::BitAnd for Symbols<S> {
    type Output = Self;

    fn bitand(self, other: Self) -> Self::Output {
        // both this and the other regular expression must match
        match (self, other) {
            // include shared included symbols
            (Self::Include(left), Self::Include(right)) => {
                Self::Include(HashSet::intersection(&left, &right).cloned().collect())
            }
            // exclude all excluded symbols
            (Self::Exclude(left), Self::Exclude(right)) => {
                Self::Exclude(HashSet::union(&left, &right).cloned().collect())
            }
            // include the included symbols except the excluded symbols
            (Self::Include(included), Self::Exclude(excluded))
            | (Self::Exclude(excluded), Self::Include(included)) => {
                Self::Include(included.difference(&excluded).cloned().collect())
            }
        }
    }
}

impl<S: Alphabet> std::ops::Not for Symbols<S> {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Include(symbols) => Self::Exclude(symbols),
            Self::Exclude(symbols) => Self::Include(symbols),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::ApproximatelySimilarCanonical;
    use crate::builder::Pure;
    use crate::ops::*;

    use super::*;

    #[test]
    fn test_derive_symbols() {
        let tests: Vec<(Regex<Pure<usize>>, Symbols<usize>, Regex<Pure<usize>>)> = vec![
            (().r(), Symbols::include([42]), ().r()),
            (().r(), Symbols::exclude([42]), ().r()),
            ([].r(), Symbols::include([42]), ().r()),
            ([].r(), Symbols::exclude([42]), ().r()),
            ([42.s()].r(), Symbols::include([42]), [].r()),
            ([42.s()].r(), Symbols::exclude([42]), ().r()),
            (!().r(), Symbols::include([42]), !().r()),
            (!().r(), Symbols::exclude([42]), !().r()),
            (!42.s(), Symbols::include([42]), ![].r()),
            (!42.s(), Symbols::exclude([42]), !().r()),
        ];
        for (r, symbols, expected) in tests {
            let actual = r.derive_symbols(&symbols);
            assert_eq!(
                expected, actual,
                "derive {} for {}, expected {}, got {}",
                r, symbols, expected, actual
            );
        }
    }

    #[test]
    fn test_is_match_pure() {
        test_is_match::<Pure<_>>();
    }

    #[test]
    fn test_is_match_asc() {
        test_is_match::<ApproximatelySimilarCanonical<_>>();
    }

    fn test_is_match<B: Builder<Symbol = usize> + Clone>() {
        let tests: Vec<(Regex<B>, Vec<_>, bool)> = vec![
            ((().r()), vec![], false),
            (([].r()), vec![], true),
            (42.s(), vec![42], true),
            (42.s(), vec![42, 42], false),
            (42.s(), vec![11], false),
            ((![42.s()].r()), vec![], true),
            (([42.s()].r()), vec![42], true),
            (([42.s(), (11.s() | 7.s())].r()), vec![42], false),
            (([42.s(), (11.s() | 7.s())].r()), vec![42, 11], true),
            (([42.s(), (11.s() | 7.s())].r()), vec![42, 7], true),
            ((42.s().c()), vec![], true),
            ((42.s().c()), vec![42], true),
            ((42.s().c()), vec![42, 42, 42], true),
            ((42.s().c()), vec![42, 11], false),
            ((42.s() & [].r()), vec![42], false),
            ((42.s() & 42.s().c()), vec![42], true),
            ((42.s() & 42.s().c()), vec![42, 42], false),
            ((42.s() | 11.s()), vec![42], true),
            ((42.s() | 11.s()), vec![11], true),
            ((42.s() | 11.s()), vec![11, 42], false),
            ((42.s() | 11.s().c()), vec![42], true),
            ((42.s() | 11.s().c()), vec![11], true),
            ((42.s() | 11.s().c()), vec![11, 11], true),
            ((42.s() | 11.s().c()), vec![42, 11], false),
            ((!().r()), vec![11], true),
            ((!11.s()), vec![42], true),
            ((!11.s()), vec![11], false),
        ];
        for test in tests {
            assert_eq!(test.2, test.0.is_match(test.1));
        }
    }
}
