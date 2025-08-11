use std::collections::{HashMap, VecDeque};

use crate::{
    arena::{Arena, ID, state},
    canonicalize::{reverse_mapping, uncanonicalize_substitution},
    clause::{Goal, KnowledgeBase},
    solver::{GoalState, Solver, stack::DepthFirstNumber},
    substitution::Substitution,
};

/// Manages the SLG tables for the solver.
///
/// Maps between [`Goal`] to the [`ID<Table>`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tables {
    tables: Arena<Table, state::Default>,
    table_ids_by_goal: HashMap<Goal, ID<Table>>,
}

impl Tables {
    pub fn new() -> Self {
        Self { tables: Arena::new(), table_ids_by_goal: HashMap::new() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum EnsureAnswer {
    AnswerAvailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Error {
    NoMoreSolutions,
    PositiveCyclicDependency(DepthFirstNumber),
    NegativeCyclicDependency,
}

#[derive(Debug)]
enum PullAnswerFromStrand {
    Stale,
    NewAnswer,
    Progress,
}

impl Solver<'_> {
    /// Gets an ID to the table for the given goal.
    pub(super) fn get_table_id(
        &mut self,
        canonicalized_goal: &Goal,
    ) -> ID<Table> {
        if let Some(table_id) =
            self.tables.table_ids_by_goal.get(canonicalized_goal)
        {
            return *table_id;
        }

        let id = ID::new(self.tables.table_ids_by_goal.len() as u64);
        self.tables.table_ids_by_goal.insert(canonicalized_goal.clone(), id);

        let new_table =
            self.create_table(self.knowledge_base, canonicalized_goal);

        self.tables.tables.insert_with_id(id, new_table).unwrap();

        id
    }

    pub(super) fn get_answer(
        &self,
        table_id: ID<Table>,
        answer_index: usize,
    ) -> Option<&Substitution> {
        self.tables
            .tables
            .get(table_id)
            .and_then(|table| table.answers.get(answer_index))
    }

    pub(super) fn ensure_answer(
        &mut self,
        table_id: ID<Table>,
        answer_index: usize,
    ) -> Result<EnsureAnswer, Error> {
        let table = self.tables.tables.get(table_id).unwrap();

        // if the table already has answers (memoized), return it immediately
        if answer_index < table.answers.len() {
            // if the answer is already available, return it
            return Ok(EnsureAnswer::AnswerAvailable);
        }

        // if reaches here, it means the answer is not yet available, we need
        // to process a new strand.
        assert!(table.answers.len() == answer_index);

        if let Some(counter) = self.stack.is_active(table_id) {
            // if the table is already active, we cannot process it again
            return Err(Error::PositiveCyclicDependency(
                self.stack[counter].depth_first_number,
            ));
        }

        let stack_index = self.stack.push(table_id);

        // pull the next answer from the strand
        let result = self.pull_next_answer(table_id, stack_index);

        self.stack.pop();

        result.map(|()| EnsureAnswer::AnswerAvailable)
    }

    /// Pulls out a new answer from the strand to the [`Table::answers`] list.
    fn pull_next_answer(
        &mut self,
        table_id: ID<Table>,
        stack_index: usize,
    ) -> Result<(), Error> {
        let mut cyclic_counter = DepthFirstNumber::MAX;
        let mut delayed_strands = Vec::new();

        loop {
            match self.tables.tables[table_id].work_list.pop_front() {
                Some(strand) => {
                    let result =
                        self.try_pull_next_answer_from_strand(table_id, strand);

                    match result {
                        // new answer has been created, stop now enough progress
                        // has been made
                        Ok(PullAnswerFromStrand::NewAnswer) => {
                            // push the cyclic dependency found back for further
                            // processing
                            self.tables.tables[table_id]
                                .work_list
                                .extend(delayed_strands);

                            return Ok(());
                        }

                        // continue processing the next strand
                        Ok(PullAnswerFromStrand::Stale)
                        | Ok(PullAnswerFromStrand::Progress) => {
                            continue;
                        }

                        Err((Error::NegativeCyclicDependency, _)) => {
                            return Err(Error::NegativeCyclicDependency);
                        }

                        Err((
                            Error::PositiveCyclicDependency(counter),
                            strand,
                        )) => {
                            delayed_strands.push(strand);
                            cyclic_counter = counter.min(cyclic_counter);
                        }

                        Err((Error::NoMoreSolutions, _)) => {
                            // this strand can't produce any more answers,
                            // continue
                            continue;
                        }
                    }
                }

                // no more strand to produce answer, no more new answers
                None => {
                    if delayed_strands.is_empty() {
                        return Err(Error::NoMoreSolutions);
                    }

                    return Err(self.cyclic(
                        delayed_strands,
                        cyclic_counter,
                        stack_index,
                    ));
                }
            }
        }
    }

    fn cyclic(
        &mut self,
        cylic_strands: Vec<Strand>,
        cyclic_counter: DepthFirstNumber,
        stack_index: usize,
    ) -> Error {
        let current_table = self.stack[stack_index].table;
        let current_dfn = self.stack[stack_index].depth_first_number;

        match current_dfn.cmp(&cyclic_counter) {
            std::cmp::Ordering::Less => {
                // negative cyclic dependency
                Error::NegativeCyclicDependency
            }

            std::cmp::Ordering::Equal => {
                self.clear_strands_after_cycle(current_table, cylic_strands);

                Error::NoMoreSolutions
            }

            std::cmp::Ordering::Greater => {
                self.tables.tables[current_table]
                    .work_list
                    .extend(cylic_strands);

                Error::PositiveCyclicDependency(cyclic_counter)
            }
        }
    }

    fn clear_strands_after_cycle(
        &mut self,
        table_id: ID<Table>,
        strands: impl IntoIterator<Item = Strand>,
    ) {
        assert!(self.tables.tables[table_id].work_list.is_empty());

        for strand in strands {
            let selected_strand_table_id =
                strand.selected_subgoal_state.table_id;

            let strands = std::mem::take(
                &mut self.tables.tables[selected_strand_table_id].work_list,
            );

            self.clear_strands_after_cycle(selected_strand_table_id, strands);
        }
    }

    #[allow(clippy::result_large_err)]
    fn try_pull_next_answer_from_strand(
        &mut self,
        table_id: ID<Table>,
        mut selected_strand: Strand,
    ) -> Result<PullAnswerFromStrand, (Error, Strand)> {
        match self.ensure_answer(
            selected_strand.selected_subgoal_state.table_id,
            selected_strand.selected_subgoal_state.answer_index,
        ) {
            Ok(EnsureAnswer::AnswerAvailable) => {}

            Err(Error::PositiveCyclicDependency(counter)) => {
                // propagate the cyclic dependency error
                return Err((
                    Error::PositiveCyclicDependency(counter),
                    selected_strand,
                ));
            }

            Err(Error::NegativeCyclicDependency) => {
                // propagate the negative cyclic dependency error
                return Err((Error::NegativeCyclicDependency, selected_strand));
            }

            // if the answer is not available, this strand will be dropped,
            // e.g. removed from the table
            Err(Error::NoMoreSolutions) => {
                return Ok(PullAnswerFromStrand::Stale);
            }
        };

        // if reaches here, it means that the answer at the
        // `selected_strand.selected_subgoal_state` exists

        let pulled_answer = self.tables.tables
            [selected_strand.selected_subgoal_state.table_id]
            .answers[selected_strand.selected_subgoal_state.answer_index]
            .clone();

        let uncanonicalized_substitution = uncanonicalize_substitution(
            &pulled_answer,
            &selected_strand.selected_subgoal_state.canonical_mapping,
        );

        // here, we'll "fork" the strand, the current "selected_strand" will
        // pursue the next answer of the current selected subgoal, whereas the
        // `next_strand` will drop the current selected subgoal and pull a new
        // subgoal to prove from the work list.
        selected_strand.selected_subgoal_state.answer_index += 1;

        // no more subgoal left to prove, push to the answer list.
        if selected_strand.rest_subgoals.is_empty() {
            let table = &mut self.tables.tables[table_id];

            let mut answer = selected_strand.substitution.clone();
            answer.compose(uncanonicalized_substitution);

            let added = table.insert_answer(answer);
            table.work_list.push_back(selected_strand);

            // New answers have been added, report back to the caller.
            Ok(if added {
                PullAnswerFromStrand::NewAnswer
            } else {
                PullAnswerFromStrand::Progress
            })
        } else {
            let mut forked = selected_strand.clone();

            // compose a new substitution
            forked.substitution.compose(uncanonicalized_substitution);

            // pop the subgoal list
            forked.selected_subgoal = forked.rest_subgoals.pop_front().unwrap();

            // apply the substitution
            forked
                .substitution
                .apply_predicate(&mut forked.selected_subgoal.predicate);

            // canonicalize the new subgoal
            let mapping = forked.selected_subgoal.canonicalize();
            let mapping = reverse_mapping(&mapping);

            forked.selected_subgoal_state = GoalState {
                answer_index: 0,
                table_id: self.get_table_id(&forked.selected_subgoal),
                canonical_mapping: mapping,
            };

            // push the forked strand and the parent strand to the work lit
            let table = &mut self.tables.tables[table_id];

            // make sure a new forked strand is processed first.
            table.work_list.push_back(forked);
            table.work_list.push_back(selected_strand);

            Ok(PullAnswerFromStrand::Progress)
        }
    }
}

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

    /// The canonicalized goal being proven.
    canonicalized_goal: Goal,

    /// The maximum variable index found in the [`Self::canonicalized_goal`]
    max_inference_variable_index: Option<usize>,
}

impl Table {
    pub fn insert_answer(&mut self, mut answer: Substitution) -> bool {
        let answer_to_add =
            if let Some(max_index) = self.max_inference_variable_index {
                // if the answer has inference variables, we need to filter them
                // out to avoid storing unnecessary data
                answer.mapping.retain(|k, _| *k <= max_index);
                answer
            } else {
                Substitution::default()
            };

        // check if the answer is already present
        if self.answers.contains(&answer_to_add) {
            return false;
        }

        self.answers.push(answer_to_add);
        true
    }
}

impl Solver<'_> {
    fn create_table(
        &mut self,
        knowledge_base: &KnowledgeBase,
        canonicalized_goal: &Goal,
    ) -> Table {
        // create a new table by looking at the matching clauses
        let clauses =
            knowledge_base.get_clauses(&canonicalized_goal.predicate.name);

        let mut answers = Vec::new();
        let mut strands = VecDeque::new();
        let max_inference_variable_index =
            canonicalized_goal.max_variable_index();

        // find the applicable clause to create a new stand.
        for clause in clauses.into_iter().flatten() {
            // check if the clause is applicable

            let mut clause = clause.clone();
            clause.canonicalize_with_counter(
                max_inference_variable_index.map_or(0, |x| x + 1),
            );

            let Some(substitution) = Substitution::default()
                .unify_predicate(&canonicalized_goal.predicate, &clause.head)
            else {
                continue;
            };

            if clause.body.is_empty() {
                answers.push(substitution);
            } else {
                // select the first subgoal as the selected subgoal right away
                let mut selected_subgoal = clause.body[0].clone();

                substitution.apply_predicate(&mut selected_subgoal.predicate);
                let mapping = selected_subgoal.canonicalize();
                let mapping = reverse_mapping(&mapping);

                // push a new strand
                strands.push_back(Strand {
                    selected_subgoal_state: GoalState {
                        answer_index: 0,
                        table_id: self.get_table_id(&selected_subgoal),
                        canonical_mapping: mapping,
                    },

                    rest_subgoals: clause.body[1..].to_vec().into(),
                    selected_subgoal,
                    substitution,
                });
            }
        }

        Table {
            work_list: strands,
            answers,
            canonicalized_goal: canonicalized_goal.clone(),
            max_inference_variable_index: canonicalized_goal
                .max_variable_index(),
        }
    }
}

/// Represents a "way to prove the goal".
///
/// A strand consists of a series of subgoals that need to be proven in order
/// to establish the validity of the original goal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Strand {
    /// The rest of the subgoals that must be proven after the
    /// [`Self::selected_subgoal`] finishes.
    rest_subgoals: VecDeque<Goal>,

    /// The current subgoal being proven.
    selected_subgoal: Goal,

    /// The substitution built so far for this strand.
    substitution: Substitution,

    /// Describes how to pull out the answer from the
    /// [`Self::selected_subgoal`]
    selected_subgoal_state: GoalState,
}
