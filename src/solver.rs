//! Contains the solver state machine and its associated data structures

use std::collections::VecDeque;

use enum_as_inner::EnumAsInner;

use crate::{
    clause::{Goal, KnowledgeBase},
    subsitution::Substitution,
};

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
enum ProofTreeNode {
    Leaf(Goal),
    Branch(ProofTree),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProofTree {
    root: Goal,
    children: VecDeque<ProofTreeNode>,
    is_in_cycle: bool,
}

impl ProofTreeNode {
    /// Returns the left-most [`Goal`] leaf in the proof tree node.
    ///
    /// The returned goal is what the solver picked so that it has something to
    /// do.
    fn next_goal_leaf(mut self: &Self) -> Option<&Goal> {
        loop {
            match self {
                ProofTreeNode::Leaf(goal) => break Some(goal),
                ProofTreeNode::Branch(proof_tree) => {
                    self = proof_tree.children.front()?;
                }
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Solution {
    proof_tree: ProofTreeNode,
    leaf_count: usize,
    substitution: Substitution,
}

/// A solver is a state-machine allowing the user to query for solutions to a
/// particular goal
pub struct Solver<'a> {
    work_list: VecDeque<Solution>,
    knowledge_base: &'a KnowledgeBase,
    initial_counter: usize,
    counter: usize,
}

impl<'a> Solver<'a> {
    /// Creates a new [`Solver`] that will search for solutions to the given
    /// [`Goal`].
    pub fn new(mut goal: Goal, knowledge_base: &'a KnowledgeBase) -> Self {
        let mut work_list = VecDeque::new();
        let counter = goal.canonicalize();

        work_list.push_back(Solution {
            proof_tree: ProofTreeNode::Leaf(goal),
            leaf_count: 1,
            substitution: Substitution::default(),
        });

        Self { work_list, knowledge_base, initial_counter: counter, counter }
    }

    /// Retrieves the next solution
    pub fn next_solution(&mut self) -> Option<Substitution> {
        while let Some(possible_solution) = self.work_list.pop_front() {
            let goal = possible_solution.proof_tree.next_goal_leaf();

            // obtain the goal that will be proven in this solution
            let Some(goal) = goal else {
                // proof tree has no more goal to prove, it means all the work
                // to prove this solution is all done.
                return Some(possible_solution.substitution);
            };
        }

        None
    }
}
