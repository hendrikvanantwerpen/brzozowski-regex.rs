//! Build a finite automaton from a regular expression.

use std::borrow::Borrow;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

use crate::builder::ApproximatelySimilarCanonical;
use crate::builder::Regex;
use crate::derivation::Symbols;
use crate::Alphabet;

#[derive(Clone)]
pub struct FiniteAutomaton<S: Alphabet> {
    states: Vec<State<S>>,
}

#[derive(Clone)]
struct State<S: Alphabet> {
    regex: Regex<ApproximatelySimilarCanonical<S>>,
    accepting: bool,
    transitions: HashMap<S, usize>,
    default_transition: usize,
}

impl<S: Alphabet> Regex<ApproximatelySimilarCanonical<S>> {
    // FIXME add docs
    pub fn to_automaton(&self) -> FiniteAutomaton<S> {
        let mut symbols = HashSet::new();
        self.collect_symbols(&mut symbols);
        let default_symbols = Symbols::Exclude(symbols.clone());

        let mut regexes: HashMap<Self, usize> = HashMap::new();
        let mut states = Vec::new();

        let mut queue = VecDeque::new();
        fn get_or_insert<S: Alphabet>(
            regex: Regex<ApproximatelySimilarCanonical<S>>,
            queue: &mut VecDeque<Regex<ApproximatelySimilarCanonical<S>>>,
            regexes: &mut HashMap<Regex<ApproximatelySimilarCanonical<S>>, usize>,
        ) -> usize {
            if let Some(idx) = regexes.get(&regex) {
                *idx
            } else {
                let idx = regexes.len();
                regexes.insert(regex.clone(), idx);
                queue.push_back(regex);
                idx
            }
        }

        get_or_insert(self.clone(), &mut queue, &mut regexes);
        while let Some(regex) = queue.pop_front() {
            let accepting = regex.is_nullable();
            let mut transitions = HashMap::default();
            for symbol in &symbols {
                let next = regex.derive_symbols(&Symbols::include([symbol.clone()]));
                let next_idx = get_or_insert(next, &mut queue, &mut regexes);
                transitions.insert(symbol.clone(), next_idx);
            }
            let default_transition = {
                let next = regex.derive_symbols(&default_symbols);
                let next_id = get_or_insert(next, &mut queue, &mut regexes);
                next_id
            };
            states.push(State {
                regex,
                accepting,
                transitions,
                default_transition,
            });
        }

        // FIXME compute states that cannot reach accepting states

        FiniteAutomaton { states }
    }

    fn collect_symbols(&self, symbols: &mut HashSet<S>) {
        match self {
            Regex::EmptySet => {}
            Regex::EmptyString => {}
            Regex::Symbol(symbol) => {
                symbols.insert(symbol.clone());
            }
            Regex::Concat(left, right) => {
                left.collect_symbols(symbols);
                right.collect_symbols(symbols);
            }
            Regex::Closure(inner) => inner.collect_symbols(symbols),
            Regex::Or(left, right) => {
                left.collect_symbols(symbols);
                right.collect_symbols(symbols);
            }
            Regex::And(left, right) => {
                left.collect_symbols(symbols);
                right.collect_symbols(symbols);
            }
            Regex::Complement(inner) => inner.collect_symbols(symbols),
        }
    }
}

impl<S: Alphabet> FiniteAutomaton<S> {
    pub fn to_matcher<'a>(&'a self) -> Matcher<'a, S> {
        Matcher {
            fa: Cow::Borrowed(self),
            state: 0,
        }
    }

    pub fn into_matcher(self) -> Matcher<'static, S> {
        Matcher {
            fa: Cow::Owned(self),
            state: 0,
        }
    }

    fn next(&self, current: usize, symbol: &S) -> usize {
        self.states[current]
            .transitions
            .get(symbol)
            .cloned()
            .unwrap_or(self.states[current].default_transition)
    }

    fn is_accepting(&self, current: usize) -> bool {
        self.states[current].accepting
    }
}

pub struct Matcher<'a, S: Alphabet> {
    fa: Cow<'a, FiniteAutomaton<S>>,
    state: usize,
}

impl<'a, S: Alphabet> Matcher<'a, S> {
    pub fn next(&mut self, symbol: &S) -> bool {
        self.state = self.fa.next(self.state, symbol);
        self.fa.is_accepting(self.state)
    }

    pub fn next_iter<I>(&mut self, symbols: impl IntoIterator<Item = I>) -> bool
    where
        I: Borrow<S>,
    {
        for symbol in symbols {
            self.next(symbol.borrow());
        }
        self.fa.is_accepting(self.state)
    }

    pub fn regex(&self) -> &Regex<ApproximatelySimilarCanonical<S>> {
        &self.fa.states[self.state].regex
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::builder::ApproximatelySimilarCanonical;
    use crate::builder::Regex;
    use crate::ops::*;

    #[test]
    fn test_matcher() {
        let tests: Vec<(Regex<ApproximatelySimilarCanonical<usize>>, Vec<_>, bool)> = vec![
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
            assert_eq!(
                test.2,
                test.0.to_automaton().to_matcher().next_iter(&test.1),
                "expected {} matching {} with [{}]",
                test.2,
                test.0,
                test.1.iter().join(", ")
            );
        }
    }
}
