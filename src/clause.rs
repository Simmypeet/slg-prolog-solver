use std::collections::HashMap;

use crate::term::Term;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Predicate {
    pub name: String,
    pub arguments: Vec<Term>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Goal {
    pub predicate: Predicate,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Clause {
    pub head: Predicate,
    pub body: Vec<Goal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KnowledgeBase {
    clauses_by_predicate_name: HashMap<String, Vec<Clause>>,
}

impl KnowledgeBase {
    /// Returns clauses for a given predicate name
    pub fn get_clauses(&self, predicate_name: &str) -> Option<&Vec<Clause>> {
        self.clauses_by_predicate_name.get(predicate_name)
    }
    pub fn new() -> Self {
        KnowledgeBase { clauses_by_predicate_name: HashMap::new() }
    }

    pub fn add_clause(&mut self, mut clause: Clause) {
        clause.canonicalize();

        self.clauses_by_predicate_name
            .entry(clause.head.name.clone())
            .or_default()
            .push(clause);
    }
}
