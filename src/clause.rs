use std::collections::HashMap;

use crate::term::Term;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Predicate {
    pub predicate: String,
    pub args: Vec<Term>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Goal {
    pub polar: bool,
    pub predicate: Predicate,
}

impl Goal {
    pub fn canonicalize(&mut self) -> usize {
        let mut counter = 0;
        let mut mapping = HashMap::new();

        for term in &mut self.predicate.args {
            term.canonicalize_internal(&mut counter, &mut mapping);
        }

        counter
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Clause {
    pub head: Predicate,
    pub body: Vec<Goal>,
}

impl Clause {
    pub fn canonicalize(&mut self) {
        let mut counter = 0;
        let mut mapping = HashMap::new();

        for term in &mut self.head.args {
            term.canonicalize_internal(&mut counter, &mut mapping);
        }

        for goal in &mut self.body {
            for term in &mut goal.predicate.args {
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
        for term in &mut self.head.args {
            term.canonicalize_internal(counter, mapping);
        }

        for goal in &mut self.body {
            for term in &mut goal.predicate.args {
                term.canonicalize_internal(counter, mapping);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KnowledgeBase {
    clauses_by_predicate_name: HashMap<String, Vec<Clause>>,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        KnowledgeBase { clauses_by_predicate_name: HashMap::new() }
    }

    pub fn add_clause(&mut self, mut clause: Clause) {
        clause.canonicalize();

        self.clauses_by_predicate_name
            .entry(clause.head.predicate.clone())
            .or_default()
            .push(clause);
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
            Term::Atom(_) => todo!(),
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
