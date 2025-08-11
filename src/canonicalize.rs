use std::collections::HashMap;

use crate::{
    clause::{Clause, Goal, Predicate},
    substitution::Substitution,
    term::Term,
};

impl Goal {
    pub fn canonicalize(&mut self) -> HashMap<usize, usize> {
        self.predicate.canonicalize()
    }
}

impl Predicate {
    pub fn canonicalize(&mut self) -> HashMap<usize, usize> {
        let mut counter = 0;
        let mut mapping = HashMap::new();

        for term in &mut self.arguments {
            term.canonicalize_internal(&mut counter, &mut mapping);
        }

        mapping
    }
}

impl Term {
    pub fn canonicalize(&mut self) -> usize {
        self.canonicalize_with_counter(0)
    }

    pub fn canonicalize_with_counter(&mut self, mut counter: usize) -> usize {
        let mut mapping = HashMap::new();
        self.canonicalize_internal(&mut counter, &mut mapping);

        counter
    }

    fn canonicalize_internal(
        &mut self,
        counter: &mut usize,
        mapping: &mut HashMap<usize, usize>,
    ) {
        match self {
            Term::Atom(_) => {}
            Term::Variable(id) => {
                if let Some(new_id) = mapping.get(id) {
                    *id = *new_id;
                } else {
                    let new_id = *counter;
                    mapping.insert(*id, new_id);
                    *id = new_id;
                    *counter += 1;
                }
            }
            Term::Compound(_, terms) => {
                for term in terms {
                    term.canonicalize_internal(counter, mapping);
                }
            }
        }
    }
}

impl Clause {
    pub fn canonicalize(&mut self) {
        let mut counter = 0;
        let mut mapping = HashMap::new();

        for term in &mut self.head.arguments {
            term.canonicalize_internal(&mut counter, &mut mapping);
        }

        for goal in &mut self.body {
            for term in &mut goal.predicate.arguments {
                term.canonicalize_internal(&mut counter, &mut mapping);
            }
        }
    }

    pub fn canonicalize_with_counter(&mut self, mut counter: usize) -> usize {
        let mut mapping = HashMap::new();
        self.canonicalize_internal(&mut counter, &mut mapping);

        counter
    }

    fn canonicalize_internal(
        &mut self,
        counter: &mut usize,
        mapping: &mut HashMap<usize, usize>,
    ) {
        for term in &mut self.head.arguments {
            term.canonicalize_internal(counter, mapping);
        }

        for goal in &mut self.body {
            for term in &mut goal.predicate.arguments {
                term.canonicalize_internal(counter, mapping);
            }
        }
    }
}

pub fn reverse_mapping(
    mapping: &HashMap<usize, usize>,
) -> HashMap<usize, usize> {
    mapping.iter().map(|(&k, &v)| (v, k)).collect()
}

pub fn uncanonicalize_substitution(
    canonicalized_substitution: &Substitution,
    uncanonicalized_mapping: &HashMap<usize, usize>,
) -> Substitution {
    fn apply_uncanonicalization(
        term: &mut Term,
        uncanonicalized_mapping: &HashMap<usize, usize>,
    ) {
        match term {
            Term::Variable(variable) => {
                if let Some(&uncanonicalized_var) =
                    uncanonicalized_mapping.get(variable)
                {
                    *term = Term::Variable(uncanonicalized_var);
                }
            }
            Term::Compound(_, terms) => {
                for subterm in terms {
                    apply_uncanonicalization(subterm, uncanonicalized_mapping);
                }
            }
            _ => {}
        }
    }

    Substitution {
        mapping: canonicalized_substitution
            .mapping
            .iter()
            .map(|(var, term)| {
                (uncanonicalized_mapping.get(var).cloned().unwrap_or(*var), {
                    let mut term = term.clone();
                    apply_uncanonicalization(
                        &mut term,
                        uncanonicalized_mapping,
                    );
                    term
                })
            })
            .collect(),
    }
}
