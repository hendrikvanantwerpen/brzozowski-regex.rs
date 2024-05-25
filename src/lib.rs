//! Brzozowski regular expressions.

// FIXME add usage documentation (and move integration tests to here as doc tests?)

use std::hash::Hash;

mod automaton;
mod builder;
mod derivation;
mod display;
mod nullability;
pub mod ops;

pub type Regex<S> = builder::Regex<builder::Default<S>>;

pub use automaton::FiniteAutomaton;
pub use automaton::Matcher;

pub trait Alphabet: Clone + Eq + Hash + Ord {}

impl<S> Alphabet for S where S: Clone + Eq + Hash + Ord {}
