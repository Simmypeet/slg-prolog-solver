use std::collections::HashMap;

use crate::term::Term;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Substitution {
    pub mapping: HashMap<usize, Term>,
}

impl Substitution {
    /// Applies a substitution to a term, replacing any [`Term::Variable`] with
    /// the corresponding term from the substitution mapping.
    pub fn apply(&self, term: &mut Term) {
        match term {
            Term::Atom(_) => {}

            Term::Variable(variable) => {
                if let Some(replacement) = self.mapping.get(variable) {
                    *term = replacement.clone();
                }
            }

            Term::Compound(_, terms) => {
                for subterm in terms {
                    self.apply(subterm);
                }
            }
        }
    }
}
