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

    fn check_cyclic(mut self: &mut Self, goal: &Goal) -> bool {
        let mut found_cyclic = false;
        loop {
            match self {
                ProofTreeNode::Leaf(_) => break found_cyclic,

                ProofTreeNode::Branch(proof_tree) => {
                    // check if found the cycle
                    if found_cyclic {
                        proof_tree.is_in_cycle = found_cyclic;
                    } else {
                        found_cyclic = proof_tree.goal == *goal;
                    }

                    let Some(next) = proof_tree.children.front_mut() else {
                        return found_cyclic;
                    };

                    self = next;
                }
            }
        }
    }

    fn progress(
        mut self: &mut Self,
        new_head: Goal,
        new_work_leaves: VecDeque<ProofTreeNode>,
    ) {
        loop {
            match self {
                ProofTreeNode::Leaf(_) => {
                    *self = ProofTreeNode::Branch(ProofTree {
                        goal: new_head,
                        children: new_work_leaves,
                        is_in_cycle: false,
                    });

                    break;
                }
                ProofTreeNode::Branch(proof_tree) => {
                    self = proof_tree
                        .children
                        .front_mut()
                        .expect("should have next goal");
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
    Working(HashMap<ID<Strand>, SolutionState>),

    /// The goal has been successfully solved, it's either proven or disproven.
    Result(Vec<Substitution>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Table {
    entries: HashMap<Goal, State>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Strand {
    proof_tree: ProofTreeNode,
    leaf_count: usize,
    substitution: Substitution,
    solution_id: ID<Strand>,
}

/// A solver is a state-machine allowing the user to query for solutions to a
/// particular goal
#[derive(Debug, Clone)]
pub struct Solver<'a> {
    canonical_goal: Goal,
    work_list: VecDeque<Strand>,
    knowledge_base: &'a KnowledgeBase,

    canonical_counter: usize,
    initial_canonical_counter: usize,

    solution_id: ID<Strand>,
}

impl<'a> Solver<'a> {
    /// Creates a new [`Solver`] that will search for solutions to the given
    /// [`Goal`].
    pub fn new(mut goal: Goal, knowledge_base: &'a KnowledgeBase) -> Self {
        let mut work_list = VecDeque::new();
        let counter = goal.canonicalize();
        let mut solution_id = ID::default();

        work_list.push_back(Strand {
            proof_tree: ProofTreeNode::Leaf(goal.clone()),
            leaf_count: 1,
            substitution: Substitution::default(),
            solution_id: solution_id.bump_id(),
        });

        Self {
            canonical_goal: goal,
            work_list,
            knowledge_base,
            initial_canonical_counter: counter,
            canonical_counter: counter,
            solution_id,
        }
    }

    pub fn canonical_goal(&self) -> &Goal { &self.canonical_goal }

    /// Retrieves the next solution
    pub fn next_solution(&mut self) -> Option<Substitution> {
        while let Some(mut strand) = self.work_list.pop_front() {
            let goal = strand.proof_tree.next_goal_leaf().cloned();

            // If proof tree has no more goal to prove, return the substitution
            let Some(goal) = goal else {
                strand
                    .substitution
                    .mapping
                    .retain(|&x, _| x < self.initial_canonical_counter);

                return Some(strand.substitution);
            };

            let has_cycle = strand.proof_tree.check_cyclic(&goal);

            // If proof tree has a cycle, we skip this strand
            if has_cycle {
                continue;
            }

            // these are the new strands that will be added to the work list.
            // if multiple strands are created, it means that this goal can have
            // multiple solutions. if no new strands are created, it means that
            // this goal has no solutions.
            let mut new_strands = Vec::new();

            let clauses = self.knowledge_base.get_clauses(&goal.predicate.name);

            for x in clauses.into_iter().flatten() {
                // rename all the variables in the clause
                self.canonical_counter =
                    x.clone().canonicalize_with_counter(self.canonical_counter);

                let Some(next_substitution) = strand
                    .substitution
                    .clone()
                    .unify_predicate(&goal.predicate, &x.head)
                else {
                    continue;
                };

                let mut head = x.head.clone();
                next_substitution.apply_predicate(&mut head);

                let mut next_proof_tree_leaves = VecDeque::new();

                for mut body in x.body.iter().cloned() {
                    next_substitution.apply_predicate(&mut body.predicate);
                    next_proof_tree_leaves.push_back(ProofTreeNode::Leaf(body));
                }

                new_strands.push((
                    head,
                    next_proof_tree_leaves,
                    next_substitution,
                ));
            }

            // handle each strand
            for (i, (head, next_proof_tree_leaves, substitution)) in
                new_strands.into_iter().enumerate()
            {
                let mut next_strand = strand.clone();

                next_strand.leaf_count -= 1;
                next_strand.leaf_count += next_proof_tree_leaves.len();
                next_strand.substitution = substitution;

                next_strand
                    .proof_tree
                    .progress(Goal { predicate: head }, next_proof_tree_leaves);

                if i != 0 {
                    next_strand.solution_id = self.solution_id.bump_id();
                }

                self.work_list.push_back(next_strand);
            }
        }

        None
    }
}

#[cfg(test)]
mod test;
