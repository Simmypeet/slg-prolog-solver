use std::fmt;

// Term representation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Term {
    Atom(String),
    Variable(usize),
    Compound(String, Vec<Term>),
}

impl Term {
    #[must_use]
    pub fn atom(name: impl Into<String>) -> Self { Term::Atom(name.into()) }

    #[must_use]
    pub fn variable(id: usize) -> Self { Term::Variable(id) }

    #[must_use]
    pub fn component(
        name: impl Into<String>,
        args: impl IntoIterator<Item = Term>,
    ) -> Self {
        Term::Compound(name.into(), args.into_iter().collect())
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Atom(name) => write!(f, "{name}"),
            Term::Variable(name) => write!(f, "{name}"),
            Term::Compound(name, args) => {
                write!(f, "{name}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
        }
    }
}
