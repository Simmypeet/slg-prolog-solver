//! Contains the solver state machine and its associated data structures

use std::collections::{HashMap, VecDeque};

use enum_as_inner::EnumAsInner;

use crate::{
    clause::{Goal, KnowledgeBase},
    id::ID,
    subsitution::Substitution,
};

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
enum ProofTreeNode {
    Leaf(Goal),
    Branch(ProofTree),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProofTree {
    goal: Goal,
    children: VecDeque<ProofTreeNode>,
}

impl ProofTreeNode {
    /// Returns the left-most [`Goal`] leaf in the proof tree node.
    ///
    /// The returned goal is what the solver picked so that it has something to
    /// do.
    fn next_goal_leaf(mut self: &Self) -> Option<(&Goal, Vec<&Goal>)> {
        let mut stack = Vec::new();
        loop {
            match self {
                ProofTreeNode::Leaf(goal) => break Some((goal, stack)),
                ProofTreeNode::Branch(proof_tree) => {
                    stack.push(&proof_tree.goal);
                    self = proof_tree.children.front()?;
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SolutionState {
    Working,
    Done(Option<Substitution>),
}

/// Representing the state of a particular goal in the solver.
#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    /// The goal is being worked on, and the solver is looking for solutions.
    Working(HashMap<ID<Solution>, SolutionState>),

    /// The goal has been successfully solved, it's either proven or disproven.
    Result(Vec<Substitution>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Table {
    entries: HashMap<Goal, State>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Solution {
    proof_tree: ProofTreeNode,
    leaf_count: usize,
    substitution: Substitution,
    solution_id: ID<Solution>,
}

/// A solver is a state-machine allowing the user to query for solutions to a
/// particular goal
#[derive(Debug, Clone)]
pub struct Solver<'a> {
    work_list: VecDeque<Solution>,
    knowledge_base: &'a KnowledgeBase,

    initial_canonical_counter: usize,
    canonical_counter: usize,

    solution_id: ID<Solution>,

    table: Table,
}

impl<'a> Solver<'a> {
    /// Creates a new [`Solver`] that will search for solutions to the given
    /// [`Goal`].
    pub fn new(mut goal: Goal, knowledge_base: &'a KnowledgeBase) -> Self {
        let mut work_list = VecDeque::new();
        let counter = goal.canonicalize();
        let mut solution_id = ID::default();

        work_list.push_back(Solution {
            proof_tree: ProofTreeNode::Leaf(goal),
            leaf_count: 1,
            substitution: Substitution::default(),
            solution_id: solution_id.bump_id(),
        });

        Self {
            work_list,
            knowledge_base,
            initial_canonical_counter: counter,
            canonical_counter: counter,
            solution_id,
            table: Table::default(),
        }
    }

    /// Retrieves the next solution
    pub fn next_solution(&mut self) -> Option<Substitution> {
        while let Some(possible_solution) = self.work_list.pop_front() {
            let goal = possible_solution.proof_tree.next_goal_leaf();

            // obtain the goal that will be proven in this solution
            let Some((goal, stack)) = goal else {
                // proof tree has no more goal to prove, it means all the work
                // to prove this solution is all done.
                return Some(possible_solution.substitution);
            };

            let result = if let Some(state) = self.table.entries.get(goal) {
                match state {
                    State::Working(hash_map) => todo!(),
                    State::Result(substitutions) => todo!(),
                }
            } else {
                todo!()
            };
        }

        None
    }
}
