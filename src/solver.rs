//! Contains the solver state machine and its associated data structures
//!
//! # SLG Table Resolution
//!
//! The main data structure for SLG table resolution is, of course, the
//! [`Table`]. For each query, it will have a single table entry. For example,
//! a query like `?- p(X).` or `?- q(Y)` will have its own associated table
//! entry; this can be represented something like `HashMap<Goal, Table>` in the
//! solver code.
//!
//! In each query, it's possible that it will have multiple answers, and
//! within each possible answer, there may be multiple subgoals to prove.
//!
//! These three concepts of "a goal to prove", "there can be multiple answers to
//! prove a goal", and "there are subgoals to prove a particular answer", define
//! the structure of the SLG table.

use crate::{
    clause::{Goal, KnowledgeBase},
    solver::{stack::Stack, table::Tables},
    substitution::Substitution,
};

pub mod stack;
pub mod table;

/// A solver is a state-machine allowing the user to query for solutions to a
/// particular goal
#[derive(Debug, Clone)]
pub struct Solver<'a> {
    canonical_goal: Goal,
    knowledge_base: &'a KnowledgeBase,
    tables: Tables,
    stack: Stack,
}

impl Iterator for Solver<'_> {
    type Item = Substitution;

    fn next(&mut self) -> Option<Self::Item> { self.next_solution() }
}

impl<'a> Solver<'a> {
    /// Creates a new [`Solver`] that will search for solutions to the given
    /// [`Goal`].
    pub fn new(goal: Goal, knowledge_base: &'a KnowledgeBase) -> Self {
        Self {
            canonical_goal: goal,
            knowledge_base,
            tables: Tables::new(),
            stack: Stack::new(),
        }
    }

    pub fn canonical_goal(&self) -> &Goal { &self.canonical_goal }

    /// Retrieves the next solution
    pub fn next_solution(&mut self) -> Option<Substitution> { todo!() }
}

#[cfg(test)]
mod test;
