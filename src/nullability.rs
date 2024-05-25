//! Nullability of regular expressions.

use crate::builder::Builder;
use crate::builder::Regex;

impl<B: Builder> Regex<B> {
    /// Returns whether the empty string is in the language of this regular expression.
    pub fn is_nullable(&self) -> bool {
        match self {
            Self::EmptySet => false,
            Self::EmptyString => true,
            Self::Symbol(_) => false,
            Self::Concat(left, right) => left.is_nullable() && right.is_nullable(),
            Self::Closure(_) => true,
            Self::Or(left, right) => left.is_nullable() || right.is_nullable(),
            Self::And(left, right) => left.is_nullable() && right.is_nullable(),
            Self::Complement(inner) => !inner.is_nullable(),
        }
    }

    /// Returns empty string if this regular expression is nullable, otherwise returns empty set.
    pub fn nullable(&self) -> Regex<B> {
        if self.is_nullable() {
            Self::EmptyString
        } else {
            Self::EmptySet
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
    fn test_is_nullable_pure() {
        test_is_nullable::<Pure<_>>();
    }

    #[test]
    fn test_is_nullable_asc() {
        test_is_nullable::<ApproximatelySimilarCanonical<_>>();
    }

    fn test_is_nullable<B: Builder<Symbol = usize> + Clone>() {
        let tests: Vec<(Regex<B>, bool)> = vec![
            (!().r(), true),
            ([].r(), true),
            (42.s(), false),
            (().r().c(), true),
            (42.s().c(), true),
            ([42.s(), [].r()].r(), false),
            ([[].r(), 42.s()].r(), false),
            ([[].r(), 42.s().c()].r(), true),
            ((42.s() | [].r()), true),
            (([].r() | 42.s()), true),
            (([].r() | 42.s().c()), true),
            ((42.s() & ().r()), false),
            ((().r() & 42.s()), false),
            (([].r() & 42.s().c()), true),
            (([].r() & 42.s()), false),
            (!().r(), true),
            ((!42.s()), true),
        ];
        for test in tests {
            assert_eq!(test.1, test.0.is_nullable());
        }
    }
}
