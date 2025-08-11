use std::collections::{HashMap, VecDeque};

use crate::{
    arena::{Arena, ID},
    clause::{Goal, KnowledgeBase},
    solver::Solver,
    substitution::Substitution,
    term::Term,
};

/// Manages the SLG tables for the solver.
///
/// Maps between [`Goal`] to the [`ID<Table>`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tables {
    tables: Arena<Table>,
    table_ids_by_goal: HashMap<Goal, ID<Table>>,
}

impl Tables {
    pub fn new() -> Self {
        Self { tables: Arena::new(), table_ids_by_goal: HashMap::new() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum EnsureAnswer {
    AnswerAvailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Error {
    NoMoreSolutions,
    CyclicDependency,
}

enum PullAnswerFromStrandError {
    CyclicDependency(Strand),
}

enum PullAnswerFromStrand {
    Stale(Strand),
    NewAnswer,
    Progress,
}

impl Solver<'_> {
    /// Gets an ID to the table for the given goal.
    ///
    /// The [`Goal`] is allowed to be non-canonicalized, the function will
    /// ensure that the goal is canonicalized before looking it up.
    fn get_table_id(&mut self, canonicalized_goal: &Goal) -> ID<Table> {
        if let Some(table_id) =
            self.tables.table_ids_by_goal.get(canonicalized_goal)
        {
            return *table_id;
        }

        let new_table =
            self.create_table(self.knowledge_base, canonicalized_goal);

        let id = self.tables.tables.insert(new_table);
        self.tables.table_ids_by_goal.insert(canonicalized_goal.clone(), id);

        id
    }

    fn ensure_answer(
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

        if self.stack.is_active(table_id).is_some() {
            // if the table is already active, we cannot process it again
            return Err(Error::CyclicDependency);
        }

        self.stack.push(table_id);

        todo!()
    }

    /// Pulls out a new answer from the strand to the [`Table::answers`] list.
    fn pull_next_answer(&mut self, table_id: ID<Table>) -> Result<(), Error> {
        loop {
            match self.tables.tables[table_id].work_list.pop_front() {
                Some(strand) => {
                    let result =
                        self.try_pull_next_answer_from_strand(table_id, strand);

                    todo!()
                }
                None => todo!(),
            }
        }
    }

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

            Err(Error::CyclicDependency) => {
                // propagate the cyclic dependency error
                return Err((Error::NoMoreSolutions, selected_strand));
            }

            // if the answer is not available, this strand will be dropped,
            // e.g. removed from the table
            Err(Error::NoMoreSolutions) => {
                return Ok(PullAnswerFromStrand::Stale(selected_strand));
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

            table.answers.push(answer);
            table.work_list.push_back(selected_strand);

            // New answers have been added, report back to the caller.
            Ok(PullAnswerFromStrand::NewAnswer)
        } else {
            let mut forked = selected_strand.clone();

            // compose a new substitution
            forked.substitution.compose(uncanonicalized_substitution);

            // pop the subgoal list
            forked.selected_subgoal =
                selected_strand.rest_subgoals.pop_front().unwrap();

            // apply the substitution
            forked
                .substitution
                .apply_predicate(&mut forked.selected_subgoal.predicate);

            // canonicalize the new subgoal
            let mapping = forked.selected_subgoal.canonicalize();
            let mapping = reverse_mapping(&mapping);

            forked.selected_subgoal_state = SubgoalState {
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

fn reverse_mapping(mapping: &HashMap<usize, usize>) -> HashMap<usize, usize> {
    mapping.iter().map(|(&k, &v)| (v, k)).collect()
}

fn uncanonicalize_substitution(
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

        // find the applicable clause to create a new stand.
        for clause in clauses.into_iter().flatten().cloned() {
            // check if the clause is applicable
            let Some(substitution) = Substitution::default()
                .unify_predicate(&canonicalized_goal.predicate, &clause.head)
            else {
                continue;
            };

            // it's a fact, directly put it to the answer
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
                    selected_subgoal_state: SubgoalState {
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

        Table { work_list: strands, answers }
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
    selected_subgoal_state: SubgoalState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubgoalState {
    answer_index: usize,
    table_id: ID<Table>,
    canonical_mapping: HashMap<usize, usize>,
}
