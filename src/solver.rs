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

use std::collections::{HashMap, VecDeque};

use crate::{
    clause::{Goal, KnowledgeBase},
    substitution::Substitution,
};

/// Represents a "goal to prove" aspect of the SLG solver.
///
/// Table contains multiple [`Strand`]s each of which represents a possible
/// way to prove the goal. These strands are stored in a [`VecDeque`] which
/// will be processed in a round-robin fashion.
///
/// After processing a strand, it will yield a new answer and possibly create
/// new more strands to explore.
///
/// With this model, it's possible to lazily explore the search space, only
/// generating new answers as needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    /// The work list of strands to be processed.
    ///
    /// If a [`Self::work_list`] is empty, it means there are no more possible
    /// answers to create.
    work_list: VecDeque<Strand>,

    /// The list of answers that have been found so far.
    answers: Vec<Substitution>,
}

/// Represents a "way to prove the goal".
///
/// A strand consists of a series of subgoals that need to be proven in order
/// to establish the validity of the original goal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Strand {
    /// The subgoals left unproven for it to be considered proven.
    ///
    /// When each subgoal is proven, it will be removed from this
    /// [`Self::subgoals`] queue until no more subgoals remain.
    subgoals: VecDeque<Goal>,

    /// The substitution built so far for this strand.
    substitution: Substitution,

    /// The answer index of the last subgoal in the [`Self::subgoals`] queue
    /// to pull from the [`Table`].
    answer_index: usize,
}

/// A solver is a state-machine allowing the user to query for solutions to a
/// particular goal
#[derive(Debug, Clone)]
pub struct Solver<'a> {
    canonical_goal: Goal,
    knowledge_base: &'a KnowledgeBase,
    tables: HashMap<Goal, Table>,
    stack: Vec<Goal>,
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
            tables: HashMap::new(),
            stack: Vec::new(),
        }
    }

    pub fn canonical_goal(&self) -> &Goal { &self.canonical_goal }

    /// Retrieves the next solution
    pub fn next_solution(&mut self) -> Option<Substitution> { todo!() }

    fn ensure_answer(&mut self, mut goal: Goal, answer_index: usize) {
        let mapping = goal.canonicalize();

        // gets the existing table if has created.
        if let Some(table) = self.tables.get_mut(&goal) {
        } else {
            // create a new table by looking at the matching clauses
            let clauses = self.knowledge_base.get_clauses(&goal.predicate.name);

            // find the applicable clause to create a new stand.
            for mut clause in clauses.into_iter().flatten().cloned() {
                if let Some(substitution) = Substitution::default()
                    .unify_predicate(&goal.predicate, &clause.head)
                {
                }
            }
        }
    }
}

#[cfg(test)]
mod test;
