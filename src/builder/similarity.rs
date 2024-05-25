//! A builder implementation that produces regular expressions in approximately-similar canonical form.

use std::cmp::Ordering;
use std::marker::PhantomData;

use itertools::Itertools;

use crate::builder::Builder;
use crate::builder::Regex;
use crate::Alphabet;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ApproximatelySimilarCanonical<S: Alphabet> {
    _phantom: PhantomData<S>,
}

impl<S: Alphabet> Builder for ApproximatelySimilarCanonical<S> {
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
    fn symbol(value: Self::Symbol) -> Regex<Self> {
        Regex::Symbol(value)
    }

    fn closure(inner: Regex<Self>) -> Regex<Self> {
        match inner {
            // ()* --> e
            Regex::EmptySet => Self::empty_string(),
            // e* --> e
            Regex::EmptyString => Self::empty_string(),
            // e** --> e*
            Regex::Closure(inner) => Regex::Closure(inner),
            // (build)
            inner => Regex::Closure(inner.into()),
        }
    }

    fn concat(left: Regex<Self>, right: Regex<Self>) -> Regex<Self> {
        match (left, right) {
            // 0 R --> 0
            (Regex::EmptySet, _) | (_, Regex::EmptySet) => Self::empty_set(),
            // e R --> R
            (Regex::EmptyString, inner) | (inner, Regex::EmptyString) => inner,
            // R (S T) --> (R S) T
            // (build)
            (left, right) => right
                .into_reverse_concat_iter()
                .chain(left.into_reverse_concat_iter())
                .collect_vec()
                .into_iter()
                .rev()
                .reduce(|l, r| Regex::Concat(l.into(), r.into()))
                .expect("at least two items"),
        }
    }

    fn or(left: Regex<Self>, right: Regex<Self>) -> Regex<Self> {
        match (left, right) {
            // 0 | R --> R
            (Regex::EmptySet, inner) | (inner, Regex::EmptySet) => inner,
            // !0 | R --> !0
            (any, _) | (_, any) if any.is_empty_set_complement() => {
                Self::complement(Self::empty_set())
            }
            // R | R --> R
            // R | (S | T) --> (R | S) | T
            // S | R --> R | S
            // (build)
            (left, right) => right
                .into_reverse_or_iter()
                .chain(left.into_reverse_or_iter())
                .sorted_by(cmp)
                .dedup()
                .reduce(|l, r| Regex::Or(l.into(), r.into()))
                .expect("at least two items"),
        }
    }

    fn and(left: Regex<Self>, right: Regex<Self>) -> Regex<Self> {
        match (left, right) {
            // 0 & R --> 0
            (Regex::EmptySet, _) | (_, Regex::EmptySet) => Self::empty_set(),
            // !0 & R --> R
            (any, inner) | (inner, any) if any.is_empty_set_complement() => inner,
            // R & R --> R
            // R & (S & T) --> (R & S) & T
            // S | R --> R | S
            // (build)
            (left, right) => right
                .into_reverse_and_iter()
                .chain(left.into_reverse_and_iter())
                .sorted_by(cmp)
                .dedup()
                .reduce(|l, r| Regex::And(l.into(), r.into()))
                .expect("at least two items"),
        }
    }

    fn complement(inner: Regex<Self>) -> Regex<Self> {
        match inner {
            // !!R --> R
            Regex::Complement(inner) => *inner,
            // (build)
            inner => Regex::Complement(inner.into()),
        }
    }
}

impl<S: Alphabet> Regex<ApproximatelySimilarCanonical<S>> {
    /// Iterate in reverse over nested "concat" regular expressions.
    fn into_reverse_concat_iter(self) -> impl Iterator<Item = Self> {
        ReverseIter(Some(self), |r| {
            if let Regex::Concat(next, value) = r {
                (*value, Some(*next))
            } else {
                (r, None)
            }
        })
    }

    /// Iterate in reverse over nested "or" regular expressions.
    fn into_reverse_or_iter(self) -> impl Iterator<Item = Self> {
        ReverseIter(Some(self), |r| {
            if let Regex::Or(next, value) = r {
                (*value, Some(*next))
            } else {
                (r, None)
            }
        })
    }

    /// Iterate in reverse over nested "and" regular expressions.
    fn into_reverse_and_iter(self) -> impl Iterator<Item = Self> {
        ReverseIter(Some(self), |r| {
            if let Regex::And(next, value) = r {
                (*value, Some(*next))
            } else {
                (r, None)
            }
        })
    }

    // Returns whether this regular expression is the complement of the empty set.
    fn is_empty_set_complement(&self) -> bool {
        if let Regex::Complement(inner) = self {
            matches!(inner.as_ref(), Regex::EmptySet)
        } else {
            false
        }
    }
}

struct ReverseIter<S, F>(Option<Regex<ApproximatelySimilarCanonical<S>>>, F)
where
    S: Alphabet,
    F: Fn(
        Regex<ApproximatelySimilarCanonical<S>>,
    ) -> (
        Regex<ApproximatelySimilarCanonical<S>>,
        Option<Regex<ApproximatelySimilarCanonical<S>>>,
    );

impl<S, F> Iterator for ReverseIter<S, F>
where
    S: Alphabet,
    F: Fn(
        Regex<ApproximatelySimilarCanonical<S>>,
    ) -> (
        Regex<ApproximatelySimilarCanonical<S>>,
        Option<Regex<ApproximatelySimilarCanonical<S>>>,
    ),
{
    type Item = Regex<ApproximatelySimilarCanonical<S>>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut r = None;
        std::mem::swap(&mut r, &mut self.0);
        if let Some(r) = r {
            let (value, next) = self.1(r);
            self.0 = next;
            Some(value)
        } else {
            None
        }
    }
}

fn cmp<B: Builder>(left: &Regex<B>, right: &Regex<B>) -> Ordering {
    match (left, right) {
        (Regex::Symbol(left_value), Regex::Symbol(right_value)) => left_value.cmp(right_value),
        (Regex::Concat(left_left, left_right), Regex::Concat(right_left, right_right)) => {
            cmp(left_left, right_left).then(cmp(left_right, right_right))
        }
        (Regex::Closure(left_inner), Regex::Closure(right_inner)) => cmp(&left_inner, &right_inner),
        (Regex::Or(left_left, left_right), Regex::Or(right_left, right_right)) => {
            cmp(left_left, right_left).then(cmp(left_right, right_right))
        }
        (Regex::And(left_left, left_right), Regex::And(right_left, right_right)) => {
            cmp(left_left, right_left).then(cmp(left_right, right_right))
        }
        (Regex::Complement(left_inner), Regex::Complement(right_inner)) => {
            cmp(&left_inner, &right_inner)
        }
        (left, right) => rank(left).cmp(&rank(right)),
    }
}

fn rank<B: Builder>(re: &Regex<B>) -> usize {
    match re {
        Regex::EmptySet => 1,
        Regex::EmptyString => 2,
        Regex::Symbol(_) => 3,
        Regex::Concat(_, _) => 4,
        Regex::Closure(_) => 5,
        Regex::Or(_, _) => 6,
        Regex::And(_, _) => 7,
        Regex::Complement(_) => 8,
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::Pure;
    use crate::ops::*;

    use super::*;

    #[test]
    fn test_canonical_forms() {
        let tests: Vec<(
            Regex<ApproximatelySimilarCanonical<usize>>,
            Regex<Pure<usize>>,
        )> = vec![
            (().r(), ().r()),
            (().r().c(), [].r()),
            ([].r().c(), [].r()),
            (11.s() & [42.s(), ().r()].r(), ().r()),
            (42.s() & 11.s() & 17.s(), 11.s() & 17.s() & 42.s()),
            (42.s() & !11.s() & 17.s(), 17.s() & 42.s() & !11.s()),
            (42.s() | 11.s() | 17.s(), 11.s() | 17.s() | 42.s()),
            (42.s() | !11.s() | 17.s(), 17.s() | 42.s() | !11.s()),
            (!42.s() & !11.s(), !11.s() & !42.s()),
        ];
        for test in tests {
            assert_eq!(test.1, test.0.rebuild());
        }
    }

    #[test]
    fn test_equivalent_forms() {
        let tests: Vec<(
            Regex<ApproximatelySimilarCanonical<usize>>,
            Regex<ApproximatelySimilarCanonical<usize>>,
        )> = vec![
            (11.s() & 42.s() & 7.s(), 7.s() & 42.s() & 11.s()),
            (11.s() & 7.s() & 42.s(), 42.s() & 7.s() & 11.s()),
            (11.s() | 42.s() | 7.s(), 7.s() | 42.s() | 11.s()),
            (11.s() | 7.s() | 42.s(), 42.s() | 7.s() | 11.s()),
            (42.s().c().c(), 42.s().c()),
            (11.s() | 11.s(), 11.s()),
            (11.s() & !().r(), 11.s()),
            (11.s() | !().r(), !().r()),
            (11.s() & (42.s() & 7.s()), 7.s() & 11.s() & 42.s()),
            (11.s() | (42.s() | 7.s()), 7.s() | 11.s() | 42.s()),
        ];
        for test in tests {
            assert_eq!(test.1, test.0.rebuild());
        }
    }
}
