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

use std::collections::HashMap;

use crate::{
    arena::ID,
    canonicalize::{reverse_mapping, uncanonicalize_substitution},
    clause::{Goal, KnowledgeBase},
    solver::{
        stack::Stack,
        table::{EnsureAnswer, Table, Tables},
    },
    substitution::Substitution,
};

pub mod stack;
pub mod table;

/// A solver is a state-machine allowing the user to query for solutions to a
/// particular goal
#[derive(Debug, Clone)]
pub struct Solver<'a> {
    knowledge_base: &'a KnowledgeBase,
    tables: Tables,
    stack: Stack,
}

impl<'a> Solver<'a> {
    /// Creates a new [`Solver`] that will search for solutions to the given
    /// [`Goal`].
    pub fn new(knowledge_base: &'a KnowledgeBase) -> Self {
        Self { knowledge_base, tables: Tables::new(), stack: Stack::new() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalState {
    answer_index: usize,
    table_id: ID<Table>,
    canonical_mapping: HashMap<usize, usize>,
}

impl Solver<'_> {
    pub fn create_goal_state(&mut self, mut goal: Goal) -> GoalState {
        let mapping = goal.canonicalize();
        let mapping = reverse_mapping(&mapping);

        let table_id = self.get_table_id(&goal);

        GoalState { answer_index: 0, table_id, canonical_mapping: mapping }
    }

    pub fn pull_next_goal(
        &mut self,
        goal_state: &mut GoalState,
    ) -> Option<Substitution> {
        // make sure the answer we're interested is present
        let Ok(EnsureAnswer::AnswerAvailable) =
            self.ensure_answer(goal_state.table_id, goal_state.answer_index)
        else {
            return None;
        };

        // retrieve the answer and increment the counter for the next pull
        let substitution = self
            .get_answer(goal_state.table_id, goal_state.answer_index)
            .unwrap();

        goal_state.answer_index += 1;

        Some(uncanonicalize_substitution(
            substitution,
            &goal_state.canonical_mapping,
        ))
    }
}

#[cfg(test)]
mod test;
