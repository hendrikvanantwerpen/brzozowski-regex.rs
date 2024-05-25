//! Convenience syntax for compactly writing literal regular expression.

use crate::builder::Builder;
use crate::builder::Regex;

pub trait IntoRegex<B: Builder> {
    fn r(self) -> Regex<B>;
}

pub trait IntoSymbol<B: Builder> {
    fn s(self) -> Regex<B>;
}

pub trait IntoClosure<B: Builder> {
    fn c(self) -> Regex<B>;
}

impl<B: Builder> IntoRegex<B> for () {
    #[inline]
    fn r(self) -> Regex<B> {
        B::empty_set()
    }
}

// empty string is a special case of concat

pub fn sym<B: Builder>(value: B::Symbol) -> Regex<B> {
    B::symbol(value.into())
}

impl<B: Builder> IntoSymbol<B> for B::Symbol {
    #[inline]
    fn s(self) -> Regex<B> {
        B::symbol(self)
    }
}

impl<B: Builder> IntoClosure<B> for Regex<B> {
    #[inline]
    fn c(self) -> Regex<B> {
        B::closure(self)
    }
}

impl<B: Builder> std::ops::Add for Regex<B> {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        B::concat(self, rhs)
    }
}

impl<B: Builder, const X: usize> IntoRegex<B> for [Regex<B>; X] {
    fn r(self) -> Regex<B> {
        match X {
            0 => B::empty_string(),
            1 => self.into_iter().next().expect("can get only item"),
            _ => self
                .into_iter()
                .reduce(B::concat)
                .expect("can reduce multiple items"),
        }
    }
}

impl<B: Builder> std::ops::BitOr for Regex<B> {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        B::or(self, rhs)
    }
}

impl<B: Builder> std::ops::BitAnd for Regex<B> {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        B::and(self, rhs)
    }
}

impl<B: Builder> std::ops::Not for Regex<B> {
    type Output = Self;
    #[inline]
    fn not(self) -> Self::Output {
        B::complement(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::ApproximatelySimilarCanonical;
    use crate::builder::Pure;

    use super::*;

    #[test]
    fn test_ops_pure() {
        test_ops::<Pure<_>>()
    }

    #[test]
    fn test_ops_asc() {
        test_ops::<ApproximatelySimilarCanonical<_>>()
    }

    fn test_ops<B: Builder<Symbol = usize> + Clone>() {
        let _: Vec<Regex<B>> = vec![
            42.s(),
            ().r() & 42.s(),
            [1.s(), 2.s()].r(),
            [1.s(), 3.s()].r() & 7.s(),
            ().r() | 7.s(),
            !().r(),
            [].r(),
        ];
    }
}
