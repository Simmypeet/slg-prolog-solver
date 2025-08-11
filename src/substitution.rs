use std::collections::HashMap;

use crate::{clause::Predicate, term::Term};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Substitution {
    pub mapping: HashMap<usize, Term>,
}

impl Substitution {
    /// Applies a substitution to a term, replacing any [`Term::Variable`] with
    /// the corresponding term from the substitution mapping.
    pub fn apply_term(&self, term: &mut Term) {
        match term {
            Term::Atom(_) => {}

            Term::Variable(variable) => {
                if let Some(replacement) = self.mapping.get(variable) {
                    *term = replacement.clone();
                }
            }

            Term::Compound(_, terms) => {
                for subterm in terms {
                    self.apply_term(subterm);
                }
            }
        }
    }

    fn compose_mapping_in_term(
        term: &mut Term,
        variable: usize,
        term_to_insert: &Term,
    ) {
        match term {
            Term::Variable(v) if *v == variable => {
                *term = term_to_insert.clone();
            }
            Term::Compound(_, terms) => {
                for subterm in terms {
                    Self::compose_mapping_in_term(
                        subterm,
                        variable,
                        term_to_insert,
                    );
                }
            }
            _ => {}
        }
    }

    pub fn insert_mapping(&mut self, variable: usize, term: Term) {
        // compose the existing mapping with the new term
        for value in self.mapping.values_mut() {
            Self::compose_mapping_in_term(value, variable, &term);
        }

        self.mapping.insert(variable, term);
    }

    pub fn apply_predicate(&self, goal: &mut Predicate) {
        for term in goal.arguments.iter_mut() {
            self.apply_term(term);
        }
    }

    pub fn unify_terms(
        mut self,
        lhs: &Term,
        rhs: &Term,
    ) -> Option<Substitution> {
        let mut lhs = lhs.clone();
        let mut rhs = rhs.clone();

        self.apply_term(&mut lhs);
        self.apply_term(&mut rhs);

        match (&lhs, &rhs) {
            (Term::Variable(v1), Term::Variable(v2)) if v1 == v2 => Some(self),
            (Term::Variable(v), t) | (t, Term::Variable(v)) => {
                if occurs_check(v, t) {
                    None
                } else {
                    self.insert_mapping(*v, t.clone());
                    Some(self)
                }
            }
            (Term::Atom(a1), Term::Atom(a2)) if a1 == a2 => Some(self),
            (Term::Compound(f1, args1), Term::Compound(f2, args2))
                if f1 == f2 && args1.len() == args2.len() =>
            {
                let mut current_sub = self;

                for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                    current_sub = current_sub.unify_terms(arg1, arg2)?;
                }

                Some(current_sub)
            }
            _ => None,
        }
    }

    pub fn unify_predicate(
        mut self,
        lhs: &Predicate,
        rhs: &Predicate,
    ) -> Option<Substitution> {
        if lhs.name != rhs.name || lhs.arguments.len() != rhs.arguments.len() {
            return None;
        }

        for (arg1, arg2) in lhs.arguments.iter().zip(rhs.arguments.iter()) {
            self = self.unify_terms(arg1, arg2)?;
        }

        Some(self)
    }

    /// Composes the `other` substitution into `self`.
    ///
    /// Given the `other` substitution and `self` substitution, after applying
    /// composition, the `self` substitution will be equivalent of
    /// `other(self(x))`
    pub fn compose(&mut self, other: Substitution) {
        for (var, term) in other.mapping {
            self.insert_mapping(var, term);
        }
    }
}

fn occurs_check(variable: &usize, term: &Term) -> bool {
    match term {
        Term::Atom(_) => false,
        Term::Variable(v) => v == variable,
        Term::Compound(_, terms) => {
            terms.iter().any(|t| occurs_check(variable, t))
        }
    }
}
