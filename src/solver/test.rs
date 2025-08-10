// Basic tests for the SLG solver
use crate::{
    clause::{Clause, Goal, KnowledgeBase, Predicate},
    solver::Solver,
    term::Term,
};

#[test]
fn test_simple_fact() {
    // fact: parent(alice, bob).
    let clause = Clause {
        head: Predicate {
            name: "parent".to_string(),
            arguments: vec![Term::atom("alice"), Term::atom("bob")],
        },
        body: vec![],
    };
    let mut kb = KnowledgeBase::new();
    kb.add_clause(clause);

    let goal = Goal {
        predicate: Predicate {
            name: "parent".to_string(),
            arguments: vec![Term::atom("alice"), Term::atom("bob")],
        },
    };
    let mut solver = Solver::new(goal, &kb);
    let solution = solver.next_solution().unwrap();

    // no inference variable in the query, therefore, the mapping should be
    // empty
    assert!(solution.mapping.is_empty());

    assert!(solver.next_solution().is_none());
}

#[test]
fn test_simple_rule() {
    // rule: grandparent(X, Y) :- parent(X, Z), parent(Z, Y).
    let clause = Clause {
        head: Predicate {
            name: "grandparent".to_string(),
            arguments: vec![Term::variable(0), Term::variable(1)],
        },
        body: vec![
            Goal {
                predicate: Predicate {
                    name: "parent".to_string(),
                    arguments: vec![Term::variable(0), Term::variable(2)],
                },
            },
            Goal {
                predicate: Predicate {
                    name: "parent".to_string(),
                    arguments: vec![Term::variable(2), Term::variable(1)],
                },
            },
        ],
    };

    let fact1 = Clause {
        head: Predicate {
            name: "parent".to_string(),
            arguments: vec![Term::atom("alice"), Term::atom("bob")],
        },
        body: vec![],
    };
    let fact2 = Clause {
        head: Predicate {
            name: "parent".to_string(),
            arguments: vec![Term::atom("bob"), Term::atom("carol")],
        },
        body: vec![],
    };

    let mut kb = KnowledgeBase::new();
    kb.add_clause(clause);
    kb.add_clause(fact1);
    kb.add_clause(fact2);

    let goal = Goal {
        predicate: Predicate {
            name: "grandparent".to_string(),
            arguments: vec![Term::atom("alice"), Term::atom("carol")],
        },
    };
    let mut solver = Solver::new(goal, &kb);
    let solution = solver.next_solution().unwrap();

    // no inference variable in the query, therefore, the mapping should be
    // empty
    assert!(solution.mapping.is_empty());

    assert!(solver.next_solution().is_none());
}

#[test]
fn test_no_solution() {
    // fact: parent(alice, bob).
    let clause = Clause {
        head: Predicate {
            name: "parent".to_string(),
            arguments: vec![Term::atom("alice"), Term::atom("bob")],
        },
        body: vec![],
    };
    let mut kb = KnowledgeBase::new();
    kb.add_clause(clause);

    let goal = Goal {
        predicate: Predicate {
            name: "parent".to_string(),
            arguments: vec![Term::atom("bob"), Term::atom("alice")],
        },
    };
    let mut solver = Solver::new(goal, &kb);
    let solution = solver.next_solution();
    assert!(solution.is_none());
}
