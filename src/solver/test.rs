// Basic tests for the SLG solver
use crate::{
    clause::{Clause, Goal, KnowledgeBase, Predicate},
    solver::Solver,
    substitution::Substitution,
    term::Term,
};

#[test]
fn simple_fact() {
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

    let mut solver = Solver::new(&kb);
    let mut goal_state = solver.create_goal_state(goal);

    let solution = solver.pull_next_goal(&mut goal_state).unwrap();

    // no inference variable in the query, therefore, the mapping should be
    // empty
    assert!(solution.mapping.is_empty());

    assert!(solver.pull_next_goal(&mut goal_state).is_none());
}

#[test]
fn simple_rule() {
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
    let mut solver = Solver::new(&kb);
    let mut goal_state = solver.create_goal_state(goal);

    let solution = solver.pull_next_goal(&mut goal_state).unwrap();

    // no inference variable in the query, therefore, the mapping should be
    // empty
    assert!(dbg!(solution.mapping).is_empty());

    assert!(solver.pull_next_goal(&mut goal_state).is_none());
}

#[test]
fn enumerate_multiple_solution() {
    // rule:
    // parent(bob, carol).
    // parent(alice, dave).

    let fact1 = Clause {
        head: Predicate {
            name: "parent".to_string(),
            arguments: vec![Term::atom("alice"), Term::atom("dave")],
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
    kb.add_clause(fact1);
    kb.add_clause(fact2);

    let query = Goal {
        predicate: Predicate {
            name: "parent".to_string(),
            arguments: vec![Term::variable(0), Term::variable(1)],
        },
    };

    let mut solver = Solver::new(&kb);
    let mut goal_state = solver.create_goal_state(query);

    let queried_solution_1 = solver.pull_next_goal(&mut goal_state).unwrap();
    let queried_solution_2 = solver.pull_next_goal(&mut goal_state).unwrap();

    assert!(solver.pull_next_goal(&mut goal_state).is_none());

    let expecteds = [
        Substitution {
            mapping: [(0, Term::atom("alice")), (1, Term::atom("dave"))]
                .into_iter()
                .collect(),
        },
        Substitution {
            mapping: [(0, Term::atom("bob")), (1, Term::atom("carol"))]
                .into_iter()
                .collect(),
        },
    ];

    assert!(expecteds.contains(&queried_solution_1));
    assert!(expecteds.contains(&queried_solution_2));
}

/*

#[test]
fn inference_multiple_nested_solution() {
    // facts:
    // parent(bob, alice).
    // parent(alice, dave).
    // parent(dave, carol).
    let fact1 = Clause {
        head: Predicate {
            name: "parent".to_string(),
            arguments: vec![Term::atom("bob"), Term::atom("alice")],
        },
        body: vec![],
    };
    let fact2 = Clause {
        head: Predicate {
            name: "parent".to_string(),
            arguments: vec![Term::atom("alice"), Term::atom("dave")],
        },
        body: vec![],
    };
    let fact3 = Clause {
        head: Predicate {
            name: "parent".to_string(),
            arguments: vec![Term::atom("dave"), Term::atom("carol")],
        },
        body: vec![],
    };

    // rule: grandparent(X, Y) :- parent(X, Z), parent(Z, Y).
    let grandparent_rule = Clause {
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

    // rule: great_grandparent(X, Y) :- parent(X, Z), grandparent(Z, Y)
    let great_grandparent_rule = Clause {
        head: Predicate {
            name: "great_grandparent".to_string(),
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
                    name: "grandparent".to_string(),
                    arguments: vec![Term::variable(2), Term::variable(1)],
                },
            },
        ],
    };

    let mut kb = KnowledgeBase::new();
    kb.add_clause(fact1);
    kb.add_clause(fact2);
    kb.add_clause(fact3);
    kb.add_clause(grandparent_rule);
    kb.add_clause(great_grandparent_rule);

    // Test grandparent(?0, ?1) - should return 2 solutions
    let grandparent_query = Goal {
        predicate: Predicate {
            name: "grandparent".to_string(),
            arguments: vec![Term::variable(0), Term::variable(1)],
        },
    };

    let mut solver = Solver::new(grandparent_query, &kb);
    let solution1 = solver.next_solution().unwrap();
    let solution2 = solver.next_solution().unwrap();
    assert_eq!(solver.next_solution(), None);

    let expected_grandparent_solutions = [
        Substitution {
            mapping: [(0, Term::atom("bob")), (1, Term::atom("dave"))]
                .into_iter()
                .collect(),
        },
        Substitution {
            mapping: [(0, Term::atom("alice")), (1, Term::atom("carol"))]
                .into_iter()
                .collect(),
        },
    ];

    assert!(expected_grandparent_solutions.contains(&solution1));
    assert!(expected_grandparent_solutions.contains(&solution2));

    // Test great_grandparent(?0, ?1) - should return 1 solution
    let great_grandparent_query = Goal {
        predicate: Predicate {
            name: "great_grandparent".to_string(),
            arguments: vec![Term::variable(0), Term::variable(1)],
        },
    };

    let mut solver = Solver::new(great_grandparent_query, &kb);
    let solution = solver.next_solution().unwrap();
    assert!(solver.next_solution().is_none());

    let expected_great_grandparent_solution = Substitution {
        mapping: [(0, Term::atom("bob")), (1, Term::atom("carol"))]
            .into_iter()
            .collect(),
    };

    assert_eq!(solution, expected_great_grandparent_solution);
}

#[test]
fn no_solution() {
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

#[test]
fn recursive_query() {
    // Create facts
    let fact1 = Clause {
        head: Predicate {
            name: "over".to_string(),
            arguments: vec![Term::atom("a"), Term::atom("b")],
        },
        body: vec![],
    };
    let fact2 = Clause {
        head: Predicate {
            name: "over".to_string(),
            arguments: vec![Term::atom("b"), Term::atom("c")],
        },
        body: vec![],
    };
    let fact3 = Clause {
        head: Predicate {
            name: "over".to_string(),
            arguments: vec![Term::atom("c"), Term::atom("d")],
        },
        body: vec![],
    };

    // Create transitive rule: over(?0, ?1) :- over(?0, ?2), over(?2, ?1).
    let transitive_rule = Clause {
        head: Predicate {
            name: "over".to_string(),
            arguments: vec![Term::variable(0), Term::variable(1)],
        },
        body: vec![
            Goal {
                predicate: Predicate {
                    name: "over".to_string(),
                    arguments: vec![Term::variable(0), Term::variable(2)],
                },
            },
            Goal {
                predicate: Predicate {
                    name: "over".to_string(),
                    arguments: vec![Term::variable(2), Term::variable(1)],
                },
            },
        ],
    };

    let mut kb = KnowledgeBase::new();
    kb.add_clause(fact1);
    kb.add_clause(fact2);
    kb.add_clause(fact3);
    kb.add_clause(transitive_rule);

    let query = Goal {
        predicate: Predicate {
            name: "over".to_string(),
            arguments: vec![Term::atom("a"), Term::atom("d")],
        },
    };

    let mut solver = Solver::new(query.clone(), &kb);

    // Collect all solutions
    let solution1 = solver.next_solution().unwrap();
    assert_eq!(solver.next_solution(), None);

    assert!(solution1.mapping.is_empty());
}

#[test]
fn enumerate_recursive_query() {
    // Create facts
    let fact1 = Clause {
        head: Predicate {
            name: "over".to_string(),
            arguments: vec![Term::atom("a"), Term::atom("b")],
        },
        body: vec![],
    };
    let fact2 = Clause {
        head: Predicate {
            name: "over".to_string(),
            arguments: vec![Term::atom("b"), Term::atom("c")],
        },
        body: vec![],
    };
    let fact3 = Clause {
        head: Predicate {
            name: "over".to_string(),
            arguments: vec![Term::atom("c"), Term::atom("d")],
        },
        body: vec![],
    };

    // Create transitive rule: over(?0, ?1) :- over(?0, ?2), over(?2, ?1).
    let transitive_rule = Clause {
        head: Predicate {
            name: "over".to_string(),
            arguments: vec![Term::variable(0), Term::variable(1)],
        },
        body: vec![
            Goal {
                predicate: Predicate {
                    name: "over".to_string(),
                    arguments: vec![Term::variable(0), Term::variable(2)],
                },
            },
            Goal {
                predicate: Predicate {
                    name: "over".to_string(),
                    arguments: vec![Term::variable(2), Term::variable(1)],
                },
            },
        ],
    };

    let mut kb = KnowledgeBase::new();
    kb.add_clause(fact1);
    kb.add_clause(fact2);
    kb.add_clause(fact3);
    kb.add_clause(transitive_rule);

    // Query: over(a, ?0) - should return solutions where ?0 = b, c, d
    let query = Goal {
        predicate: Predicate {
            name: "over".to_string(),
            arguments: vec![Term::atom("a"), Term::variable(0)],
        },
    };

    let mut solver = Solver::new(query, &kb);

    // Collect all solutions
    let mut solutions = Vec::new();
    while let Some(solution) = solver.next_solution() {
        solutions.push(solution);
    }

    dbg!(&solutions);

    // Should have 3 solutions: ?0 = b, c, d
    assert_eq!(solutions.len(), 3);

    let expected_solutions = [
        Substitution { mapping: [(0, Term::atom("b"))].into_iter().collect() },
        Substitution { mapping: [(0, Term::atom("c"))].into_iter().collect() },
        Substitution { mapping: [(0, Term::atom("d"))].into_iter().collect() },
    ];

    // Check that all expected solutions are present
    for expected in &expected_solutions {
        assert!(
            solutions.contains(expected),
            "Missing expected solution: {expected:?}"
        );
    }
}
*/
