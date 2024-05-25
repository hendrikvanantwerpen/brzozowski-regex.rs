use crate::builder::Builder;
use crate::builder::Regex;

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
enum Level {
    None,
    Binary,
    Unary,
    Atom,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Context {
    Inner,
    Left,
}

impl<B: Builder> std::fmt::Display for Regex<B>
where
    B::Symbol: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f, Context::Inner, Level::None)
    }
}

impl<B: Builder> Regex<B>
where
    B::Symbol: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, ctx: Context, level: Level) -> std::fmt::Result {
        match ctx {
            Context::Inner | Context::Left if self.level() <= level => {
                write!(f, "(")?;
            }
            _ => {}
        }
        match self {
            Regex::EmptySet => write!(f, "∅")?,
            Regex::EmptyString => write!(f, "ε")?,
            Regex::Symbol(value) => write!(f, "{}", value)?,
            Regex::Concat(left, right) => {
                self.fmt_left(f, left, level)?;
                write!(f, " ")?;
                self.fmt_right_or_inner(f, right)?;
            }
            Regex::Closure(inner) => {
                self.fmt_right_or_inner(f, inner)?;
                write!(f, "*")?;
            }
            Regex::Or(left, right) => {
                self.fmt_left(f, left, level)?;
                write!(f, " | ")?;
                self.fmt_right_or_inner(f, right)?;
            }
            Regex::And(left, right) => {
                self.fmt_left(f, left, level)?;
                write!(f, " & ")?;
                self.fmt_right_or_inner(f, right)?;
            }
            Regex::Complement(inner) => {
                write!(f, "¬")?;
                self.fmt_right_or_inner(f, inner)?;
            }
        };
        match ctx {
            Context::Inner if self.level() <= level => {
                write!(f, ")")?;
            }
            _ => {}
        }
        Ok(())
    }

    fn fmt_left(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        left: &Regex<B>,
        outer_level: Level,
    ) -> std::fmt::Result {
        match (self, left) {
            (Self::Concat(_, _), Self::Concat(_, _))
            | (Self::Or(_, _), Self::Or(_, _))
            | (Self::And(_, _), Self::And(_, _)) => left.fmt(f, Context::Left, outer_level),
            _ => left.fmt(f, Context::Inner, self.level()),
        }
    }

    fn fmt_right_or_inner(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        right_or_inner: &Regex<B>,
    ) -> std::fmt::Result {
        right_or_inner.fmt(f, Context::Inner, self.level())
    }
}

impl<B: Builder> Regex<B> {
    fn level(&self) -> Level {
        match self {
            Regex::EmptySet | Regex::EmptyString | Regex::Symbol(_) => Level::Atom,
            Regex::Concat(_, _) | Regex::Or(_, _) | Regex::And(_, _) => Level::Binary,
            Regex::Closure(_) | Regex::Complement(_) => Level::Unary,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::Pure;
    use crate::builder::Regex;
    use crate::ops::*;

    #[test]
    fn test_display() {
        let tests: Vec<(&str, Regex<Pure<usize>>)> = vec![
            ("∅", ().r()),
            ("¬∅", !().r()),
            ("¬(11*)", !11.s().c()),
            ("ε | 11*", [].r() | 11.s().c()),
            ("¬∅ & (11 7)", !().r() & [11.s(), 7.s()].r()),
            ("1 & 2 & 4", 1.s() & 2.s() & 4.s()),
            ("1 & (2 & 4)", 1.s() & (2.s() & 4.s())),
            ("(1 & 2) | 4", (1.s() & 2.s()) | 4.s()),
            ("¬(1 2)", !(1.s() + 2.s())),
        ];
        for (expected, r) in tests {
            assert_eq!(expected, r.to_string());
        }
    }
}
