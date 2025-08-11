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
fn enumerate_single_solution() {
    // rule:
    // parent(alice, bob).
    // parent(bob, dave).
    // grandparent(X, Y) :- parent(X, Z), parent(Z, Y).

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
            arguments: vec![Term::atom("bob"), Term::atom("dave")],
        },
        body: vec![],
    };

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

    let mut kb = KnowledgeBase::new();
    kb.add_clause(fact1);
    kb.add_clause(fact2);
    kb.add_clause(grandparent_rule);

    let query = Goal {
        predicate: Predicate {
            name: "grandparent".to_string(),
            arguments: vec![Term::atom("alice"), Term::variable(0)],
        },
    };

    let mut solver = Solver::new(&kb);
    let mut goal_state = solver.create_goal_state(query);

    let queried_solution = solver.pull_next_goal(&mut goal_state).unwrap();

    assert!(solver.pull_next_goal(&mut goal_state).is_none());

    assert_eq!(queried_solution.mapping.len(), 1);
    assert_eq!(queried_solution.mapping.get(&0), Some(&Term::atom("dave")));
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

#[test]
fn enumerate_multiple_nested_solution() {
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

    let mut solver = Solver::new(&kb);
    let mut goal_state = solver.create_goal_state(grandparent_query);

    let solution1 = solver.pull_next_goal(&mut goal_state).unwrap();
    let solution2 = solver.pull_next_goal(&mut goal_state).unwrap();

    assert_eq!(solver.pull_next_goal(&mut goal_state), None);

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

    let mut goal_state = solver.create_goal_state(great_grandparent_query);

    let solution = solver.pull_next_goal(&mut goal_state).unwrap();

    assert!(solver.pull_next_goal(&mut goal_state).is_none());

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
    let mut solver = Solver::new(&kb);
    let mut goal_state = solver.create_goal_state(goal);

    let solution = solver.pull_next_goal(&mut goal_state);

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

    let mut solver = Solver::new(&kb);
    let mut goal_state = solver.create_goal_state(query);

    // Collect all solutions
    let solution1 = solver.pull_next_goal(&mut goal_state).unwrap();

    assert_eq!(solver.pull_next_goal(&mut goal_state), None);

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

    let mut solver = Solver::new(&kb);
    let mut goal_state = solver.create_goal_state(query);

    // Collect all solutions
    let mut solutions = Vec::new();
    while let Some(solution) = solver.pull_next_goal(&mut goal_state) {
        solutions.push(solution);
    }

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

#[test]
fn mutual_recursion_odd_even() {
    // Test mutual recursion with odd/even predicates
    // Facts:
    // even(0).
    // Rules:
    // odd(X) :- even(Y), succ(Y, X).
    // even(X) :- odd(Y), succ(Y, X).
    // succ(0, 1).
    // succ(1, 2).
    // succ(2, 3).
    // succ(3, 4).

    let even_fact = Clause {
        head: Predicate {
            name: "even".to_string(),
            arguments: vec![Term::atom("0")],
        },
        body: vec![],
    };

    let succ_facts = vec![
        Clause {
            head: Predicate {
                name: "succ".to_string(),
                arguments: vec![Term::atom("0"), Term::atom("1")],
            },
            body: vec![],
        },
        Clause {
            head: Predicate {
                name: "succ".to_string(),
                arguments: vec![Term::atom("1"), Term::atom("2")],
            },
            body: vec![],
        },
        Clause {
            head: Predicate {
                name: "succ".to_string(),
                arguments: vec![Term::atom("2"), Term::atom("3")],
            },
            body: vec![],
        },
        Clause {
            head: Predicate {
                name: "succ".to_string(),
                arguments: vec![Term::atom("3"), Term::atom("4")],
            },
            body: vec![],
        },
    ];

    let odd_rule = Clause {
        head: Predicate {
            name: "odd".to_string(),
            arguments: vec![Term::variable(0)],
        },
        body: vec![
            Goal {
                predicate: Predicate {
                    name: "even".to_string(),
                    arguments: vec![Term::variable(1)],
                },
            },
            Goal {
                predicate: Predicate {
                    name: "succ".to_string(),
                    arguments: vec![Term::variable(1), Term::variable(0)],
                },
            },
        ],
    };

    let even_rule = Clause {
        head: Predicate {
            name: "even".to_string(),
            arguments: vec![Term::variable(0)],
        },
        body: vec![
            Goal {
                predicate: Predicate {
                    name: "odd".to_string(),
                    arguments: vec![Term::variable(1)],
                },
            },
            Goal {
                predicate: Predicate {
                    name: "succ".to_string(),
                    arguments: vec![Term::variable(1), Term::variable(0)],
                },
            },
        ],
    };

    let mut kb = KnowledgeBase::new();
    kb.add_clause(even_fact);
    for fact in succ_facts {
        kb.add_clause(fact);
    }
    kb.add_clause(odd_rule);
    kb.add_clause(even_rule);

    // Test odd(?0) - should return solutions for 1, 3
    let odd_query = Goal {
        predicate: Predicate {
            name: "odd".to_string(),
            arguments: vec![Term::variable(0)],
        },
    };

    let mut solver = Solver::new(&kb);
    let mut goal_state = solver.create_goal_state(odd_query);

    let mut odd_solutions = Vec::new();
    while let Some(solution) = solver.pull_next_goal(&mut goal_state) {
        odd_solutions.push(solution);
    }

    assert_eq!(odd_solutions.len(), 2);
    let expected_odd = [
        Substitution { mapping: [(0, Term::atom("1"))].into_iter().collect() },
        Substitution { mapping: [(0, Term::atom("3"))].into_iter().collect() },
    ];

    for expected in &expected_odd {
        assert!(
            odd_solutions.contains(expected),
            "Missing odd solution: {expected:?}"
        );
    }

    // Test even(?0) - should return solutions for 0, 2, 4
    let even_query = Goal {
        predicate: Predicate {
            name: "even".to_string(),
            arguments: vec![Term::variable(0)],
        },
    };

    let mut goal_state = solver.create_goal_state(even_query);

    let mut even_solutions = Vec::new();
    while let Some(solution) = solver.pull_next_goal(&mut goal_state) {
        even_solutions.push(solution);
    }

    assert_eq!(even_solutions.len(), 3);
    let expected_even = [
        Substitution { mapping: [(0, Term::atom("0"))].into_iter().collect() },
        Substitution { mapping: [(0, Term::atom("2"))].into_iter().collect() },
        Substitution { mapping: [(0, Term::atom("4"))].into_iter().collect() },
    ];

    for expected in &expected_even {
        assert!(
            even_solutions.contains(expected),
            "Missing even solution: {expected:?}"
        );
    }
}

#[test]
fn graph_reachability_with_multiple_edge_types() {
    // Test recursive reachability in a graph with multiple edge types
    // Facts for different edge types:
    // road(a, b).
    // road(b, c).
    // rail(c, d).
    // rail(d, e).
    // boat(e, f).

    // connected(X, Y) :- road(X, Y).
    // connected(X, Y) :- rail(X, Y).
    // connected(X, Y) :- boat(X, Y).
    // reachable(X, Y) :- connected(X, Y).
    // reachable(X, Y) :- connected(X, Z), reachable(Z, Y).

    let road_facts = vec![
        Clause {
            head: Predicate {
                name: "road".to_string(),
                arguments: vec![Term::atom("a"), Term::atom("b")],
            },
            body: vec![],
        },
        Clause {
            head: Predicate {
                name: "road".to_string(),
                arguments: vec![Term::atom("b"), Term::atom("c")],
            },
            body: vec![],
        },
    ];

    let rail_facts = vec![
        Clause {
            head: Predicate {
                name: "rail".to_string(),
                arguments: vec![Term::atom("c"), Term::atom("d")],
            },
            body: vec![],
        },
        Clause {
            head: Predicate {
                name: "rail".to_string(),
                arguments: vec![Term::atom("d"), Term::atom("e")],
            },
            body: vec![],
        },
    ];

    let boat_fact = Clause {
        head: Predicate {
            name: "boat".to_string(),
            arguments: vec![Term::atom("e"), Term::atom("f")],
        },
        body: vec![],
    };

    let connected_rules = vec![
        Clause {
            head: Predicate {
                name: "connected".to_string(),
                arguments: vec![Term::variable(0), Term::variable(1)],
            },
            body: vec![Goal {
                predicate: Predicate {
                    name: "road".to_string(),
                    arguments: vec![Term::variable(0), Term::variable(1)],
                },
            }],
        },
        Clause {
            head: Predicate {
                name: "connected".to_string(),
                arguments: vec![Term::variable(0), Term::variable(1)],
            },
            body: vec![Goal {
                predicate: Predicate {
                    name: "rail".to_string(),
                    arguments: vec![Term::variable(0), Term::variable(1)],
                },
            }],
        },
        Clause {
            head: Predicate {
                name: "connected".to_string(),
                arguments: vec![Term::variable(0), Term::variable(1)],
            },
            body: vec![Goal {
                predicate: Predicate {
                    name: "boat".to_string(),
                    arguments: vec![Term::variable(0), Term::variable(1)],
                },
            }],
        },
    ];

    let reachable_base = Clause {
        head: Predicate {
            name: "reachable".to_string(),
            arguments: vec![Term::variable(0), Term::variable(1)],
        },
        body: vec![Goal {
            predicate: Predicate {
                name: "connected".to_string(),
                arguments: vec![Term::variable(0), Term::variable(1)],
            },
        }],
    };

    let reachable_recursive = Clause {
        head: Predicate {
            name: "reachable".to_string(),
            arguments: vec![Term::variable(0), Term::variable(1)],
        },
        body: vec![
            Goal {
                predicate: Predicate {
                    name: "connected".to_string(),
                    arguments: vec![Term::variable(0), Term::variable(2)],
                },
            },
            Goal {
                predicate: Predicate {
                    name: "reachable".to_string(),
                    arguments: vec![Term::variable(2), Term::variable(1)],
                },
            },
        ],
    };

    let mut kb = KnowledgeBase::new();
    for fact in road_facts {
        kb.add_clause(fact);
    }
    for fact in rail_facts {
        kb.add_clause(fact);
    }
    kb.add_clause(boat_fact);
    for rule in connected_rules {
        kb.add_clause(rule);
    }
    kb.add_clause(reachable_base);
    kb.add_clause(reachable_recursive);

    // Test reachable(a, ?0) - should find all nodes reachable from 'a'
    let query = Goal {
        predicate: Predicate {
            name: "reachable".to_string(),
            arguments: vec![Term::atom("a"), Term::variable(0)],
        },
    };

    let mut solver = Solver::new(&kb);
    let mut goal_state = solver.create_goal_state(query);

    let mut solutions = Vec::new();
    while let Some(solution) = solver.pull_next_goal(&mut goal_state) {
        solutions.push(solution);
    }

    // Should be able to reach b, c, d, e, f from a
    assert_eq!(solutions.len(), 5);
    let expected_destinations = ["b", "c", "d", "e", "f"];

    for dest in expected_destinations {
        let expected = Substitution {
            mapping: [(0, Term::atom(dest))].into_iter().collect(),
        };
        assert!(
            solutions.contains(&expected),
            "Missing reachable destination: {dest}"
        );
    }
}

#[test]
fn family_relationships_complex_recursion() {
    // Test complex family relationships with multiple recursive predicates
    // Facts:
    // parent(adam, bob).
    // parent(adam, carol).
    // parent(bob, dan).
    // parent(bob, eve).
    // parent(carol, frank).
    // parent(dan, grace).

    // Rules:
    // ancestor(X, Y) :- parent(X, Y).
    // ancestor(X, Y) :- parent(X, Z), ancestor(Z, Y).
    // sibling(X, Y) :- parent(Z, X), parent(Z, Y), X != Y.
    // cousin(X, Y) :- parent(A, X), parent(B, Y), sibling(A, B).
    // relative(X, Y) :- ancestor(X, Y).
    // relative(X, Y) :- ancestor(Y, X).
    // relative(X, Y) :- sibling(X, Y).
    // relative(X, Y) :- cousin(X, Y).

    let parent_facts = vec![
        Clause {
            head: Predicate {
                name: "parent".to_string(),
                arguments: vec![Term::atom("adam"), Term::atom("bob")],
            },
            body: vec![],
        },
        Clause {
            head: Predicate {
                name: "parent".to_string(),
                arguments: vec![Term::atom("adam"), Term::atom("carol")],
            },
            body: vec![],
        },
        Clause {
            head: Predicate {
                name: "parent".to_string(),
                arguments: vec![Term::atom("bob"), Term::atom("dan")],
            },
            body: vec![],
        },
        Clause {
            head: Predicate {
                name: "parent".to_string(),
                arguments: vec![Term::atom("bob"), Term::atom("eve")],
            },
            body: vec![],
        },
        Clause {
            head: Predicate {
                name: "parent".to_string(),
                arguments: vec![Term::atom("carol"), Term::atom("frank")],
            },
            body: vec![],
        },
        Clause {
            head: Predicate {
                name: "parent".to_string(),
                arguments: vec![Term::atom("dan"), Term::atom("grace")],
            },
            body: vec![],
        },
    ];

    let ancestor_base = Clause {
        head: Predicate {
            name: "ancestor".to_string(),
            arguments: vec![Term::variable(0), Term::variable(1)],
        },
        body: vec![Goal {
            predicate: Predicate {
                name: "parent".to_string(),
                arguments: vec![Term::variable(0), Term::variable(1)],
            },
        }],
    };

    let ancestor_recursive = Clause {
        head: Predicate {
            name: "ancestor".to_string(),
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
                    name: "ancestor".to_string(),
                    arguments: vec![Term::variable(2), Term::variable(1)],
                },
            },
        ],
    };

    let sibling_rule = Clause {
        head: Predicate {
            name: "sibling".to_string(),
            arguments: vec![Term::variable(0), Term::variable(1)],
        },
        body: vec![
            Goal {
                predicate: Predicate {
                    name: "parent".to_string(),
                    arguments: vec![Term::variable(2), Term::variable(0)],
                },
            },
            Goal {
                predicate: Predicate {
                    name: "parent".to_string(),
                    arguments: vec![Term::variable(2), Term::variable(1)],
                },
            },
            // Note: We can't directly express X != Y in this test framework,
            // so this will include reflexive pairs
        ],
    };

    let cousin_rule = Clause {
        head: Predicate {
            name: "cousin".to_string(),
            arguments: vec![Term::variable(0), Term::variable(1)],
        },
        body: vec![
            Goal {
                predicate: Predicate {
                    name: "parent".to_string(),
                    arguments: vec![Term::variable(2), Term::variable(0)],
                },
            },
            Goal {
                predicate: Predicate {
                    name: "parent".to_string(),
                    arguments: vec![Term::variable(3), Term::variable(1)],
                },
            },
            Goal {
                predicate: Predicate {
                    name: "sibling".to_string(),
                    arguments: vec![Term::variable(2), Term::variable(3)],
                },
            },
        ],
    };

    let relative_rules = vec![
        Clause {
            head: Predicate {
                name: "relative".to_string(),
                arguments: vec![Term::variable(0), Term::variable(1)],
            },
            body: vec![Goal {
                predicate: Predicate {
                    name: "ancestor".to_string(),
                    arguments: vec![Term::variable(0), Term::variable(1)],
                },
            }],
        },
        Clause {
            head: Predicate {
                name: "relative".to_string(),
                arguments: vec![Term::variable(0), Term::variable(1)],
            },
            body: vec![Goal {
                predicate: Predicate {
                    name: "ancestor".to_string(),
                    arguments: vec![Term::variable(1), Term::variable(0)],
                },
            }],
        },
        Clause {
            head: Predicate {
                name: "relative".to_string(),
                arguments: vec![Term::variable(0), Term::variable(1)],
            },
            body: vec![Goal {
                predicate: Predicate {
                    name: "sibling".to_string(),
                    arguments: vec![Term::variable(0), Term::variable(1)],
                },
            }],
        },
        Clause {
            head: Predicate {
                name: "relative".to_string(),
                arguments: vec![Term::variable(0), Term::variable(1)],
            },
            body: vec![Goal {
                predicate: Predicate {
                    name: "cousin".to_string(),
                    arguments: vec![Term::variable(0), Term::variable(1)],
                },
            }],
        },
    ];

    let mut kb = KnowledgeBase::new();
    for fact in parent_facts {
        kb.add_clause(fact);
    }
    kb.add_clause(ancestor_base);
    kb.add_clause(ancestor_recursive);
    kb.add_clause(sibling_rule);
    kb.add_clause(cousin_rule);
    for rule in relative_rules {
        kb.add_clause(rule);
    }

    // Test ancestor(adam, ?0) - should find all descendants of adam
    let ancestor_query = Goal {
        predicate: Predicate {
            name: "ancestor".to_string(),
            arguments: vec![Term::atom("adam"), Term::variable(0)],
        },
    };

    let mut solver = Solver::new(&kb);
    let mut goal_state = solver.create_goal_state(ancestor_query);

    let mut ancestor_solutions = Vec::new();
    while let Some(solution) = solver.pull_next_goal(&mut goal_state) {
        ancestor_solutions.push(solution);
    }

    // Adam is ancestor of: bob, carol, dan, eve, frank, grace
    assert_eq!(ancestor_solutions.len(), 6);
    let expected_descendants = ["bob", "carol", "dan", "eve", "frank", "grace"];

    for descendant in expected_descendants {
        let expected = Substitution {
            mapping: [(0, Term::atom(descendant))].into_iter().collect(),
        };
        assert!(
            ancestor_solutions.contains(&expected),
            "Missing descendant: {descendant}"
        );
    }

    // Test cousin(dan, ?0) - should find cousins of dan
    let cousin_query = Goal {
        predicate: Predicate {
            name: "cousin".to_string(),
            arguments: vec![Term::atom("dan"), Term::variable(0)],
        },
    };

    let mut goal_state = solver.create_goal_state(cousin_query);

    let mut cousin_solutions = Vec::new();
    while let Some(solution) = solver.pull_next_goal(&mut goal_state) {
        cousin_solutions.push(solution);
    }

    // Dan's cousin should be frank (both have grandparent adam through
    // different parents) Note: The sibling rule will also match reflexively
    // (X, X) since we can't express X != Y So we might get more solutions
    // than expected
    assert!(!cousin_solutions.is_empty());
    let expected_cousin = Substitution {
        mapping: [(0, Term::atom("frank"))].into_iter().collect(),
    };
    assert!(cousin_solutions.contains(&expected_cousin));
}

#[test]
fn circular_dependency_with_multiple_predicates() {
    // Test handling of circular dependencies across multiple predicates
    // Facts:
    // depends(a, b).
    // depends(b, c).
    // depends(c, a).  // Creates a cycle
    // depends(d, e).

    // Rules:
    // indirect_depends(X, Y) :- depends(X, Y).
    // indirect_depends(X, Y) :- depends(X, Z), indirect_depends(Z, Y).
    // related(X, Y) :- indirect_depends(X, Y).
    // related(X, Y) :- indirect_depends(Y, X).

    let depend_facts = vec![
        Clause {
            head: Predicate {
                name: "depends".to_string(),
                arguments: vec![Term::atom("a"), Term::atom("b")],
            },
            body: vec![],
        },
        Clause {
            head: Predicate {
                name: "depends".to_string(),
                arguments: vec![Term::atom("b"), Term::atom("c")],
            },
            body: vec![],
        },
        Clause {
            head: Predicate {
                name: "depends".to_string(),
                arguments: vec![Term::atom("c"), Term::atom("a")],
            },
            body: vec![],
        },
        Clause {
            head: Predicate {
                name: "depends".to_string(),
                arguments: vec![Term::atom("d"), Term::atom("e")],
            },
            body: vec![],
        },
    ];

    let indirect_depends_base = Clause {
        head: Predicate {
            name: "indirect_depends".to_string(),
            arguments: vec![Term::variable(0), Term::variable(1)],
        },
        body: vec![Goal {
            predicate: Predicate {
                name: "depends".to_string(),
                arguments: vec![Term::variable(0), Term::variable(1)],
            },
        }],
    };

    let indirect_depends_recursive = Clause {
        head: Predicate {
            name: "indirect_depends".to_string(),
            arguments: vec![Term::variable(0), Term::variable(1)],
        },
        body: vec![
            Goal {
                predicate: Predicate {
                    name: "depends".to_string(),
                    arguments: vec![Term::variable(0), Term::variable(2)],
                },
            },
            Goal {
                predicate: Predicate {
                    name: "indirect_depends".to_string(),
                    arguments: vec![Term::variable(2), Term::variable(1)],
                },
            },
        ],
    };

    let related_rules = vec![
        Clause {
            head: Predicate {
                name: "related".to_string(),
                arguments: vec![Term::variable(0), Term::variable(1)],
            },
            body: vec![Goal {
                predicate: Predicate {
                    name: "indirect_depends".to_string(),
                    arguments: vec![Term::variable(0), Term::variable(1)],
                },
            }],
        },
        Clause {
            head: Predicate {
                name: "related".to_string(),
                arguments: vec![Term::variable(0), Term::variable(1)],
            },
            body: vec![Goal {
                predicate: Predicate {
                    name: "indirect_depends".to_string(),
                    arguments: vec![Term::variable(1), Term::variable(0)],
                },
            }],
        },
    ];

    let mut kb = KnowledgeBase::new();
    for fact in depend_facts {
        kb.add_clause(fact);
    }
    kb.add_clause(indirect_depends_base);
    kb.add_clause(indirect_depends_recursive);
    for rule in related_rules {
        kb.add_clause(rule);
    }

    // Test indirect_depends(a, ?0) - should handle the cycle properly
    let query = Goal {
        predicate: Predicate {
            name: "indirect_depends".to_string(),
            arguments: vec![Term::atom("a"), Term::variable(0)],
        },
    };

    let mut solver = Solver::new(&kb);
    let mut goal_state = solver.create_goal_state(query);

    let mut solutions = Vec::new();
    while let Some(solution) = solver.pull_next_goal(&mut goal_state) {
        solutions.push(solution);
    }

    // Due to the cycle a->b->c->a, 'a' should indirectly depend on b and c
    // The solver should handle this without infinite loops
    assert!(solutions.len() >= 2); // At least b and c

    let expected_minimal = [
        Substitution { mapping: [(0, Term::atom("b"))].into_iter().collect() },
        Substitution { mapping: [(0, Term::atom("c"))].into_iter().collect() },
    ];

    for expected in &expected_minimal {
        assert!(
            solutions.contains(expected),
            "Missing expected dependency: {expected:?}"
        );
    }

    // Test that 'd' only depends on 'e' (no cycle)
    let query_d = Goal {
        predicate: Predicate {
            name: "indirect_depends".to_string(),
            arguments: vec![Term::atom("d"), Term::variable(0)],
        },
    };

    let mut goal_state_d = solver.create_goal_state(query_d);

    let mut solutions_d = Vec::new();
    while let Some(solution) = solver.pull_next_goal(&mut goal_state_d) {
        solutions_d.push(solution);
    }

    assert_eq!(solutions_d.len(), 1);
    let expected_d =
        Substitution { mapping: [(0, Term::atom("e"))].into_iter().collect() };
    assert!(solutions_d.contains(&expected_d));
}

#[test]
fn deep_recursive_chain_with_branching() {
    // Test deep recursive chains with branching paths
    // This creates a tree-like structure to test deep recursion
    // Facts:
    // connects(root, a1).
    // connects(root, a2).
    // connects(a1, b1).
    // connects(a1, b2).
    // connects(a2, b3).
    // connects(b1, c1).
    // connects(b2, c2).
    // connects(b3, c3).
    // connects(c1, d1).
    // connects(c2, d2).
    // connects(c3, d3).

    // Rules:
    // path(X, Y) :- connects(X, Y).
    // path(X, Y) :- connects(X, Z), path(Z, Y).
    // reachable_from_root(X) :- path(root, X).
    // depth_2(X) :- connects(Y, X), connects(root, Y).
    // depth_3(X) :- connects(Y, X), depth_2(Y).

    let connect_facts = vec![
        // Level 1
        ("root", "a1"),
        ("root", "a2"),
        // Level 2
        ("a1", "b1"),
        ("a1", "b2"),
        ("a2", "b3"),
        // Level 3
        ("b1", "c1"),
        ("b2", "c2"),
        ("b3", "c3"),
        // Level 4
        ("c1", "d1"),
        ("c2", "d2"),
        ("c3", "d3"),
    ]
    .into_iter()
    .map(|(from, to)| Clause {
        head: Predicate {
            name: "connects".to_string(),
            arguments: vec![Term::atom(from), Term::atom(to)],
        },
        body: vec![],
    })
    .collect::<Vec<_>>();

    let path_base = Clause {
        head: Predicate {
            name: "path".to_string(),
            arguments: vec![Term::variable(0), Term::variable(1)],
        },
        body: vec![Goal {
            predicate: Predicate {
                name: "connects".to_string(),
                arguments: vec![Term::variable(0), Term::variable(1)],
            },
        }],
    };

    let path_recursive = Clause {
        head: Predicate {
            name: "path".to_string(),
            arguments: vec![Term::variable(0), Term::variable(1)],
        },
        body: vec![
            Goal {
                predicate: Predicate {
                    name: "connects".to_string(),
                    arguments: vec![Term::variable(0), Term::variable(2)],
                },
            },
            Goal {
                predicate: Predicate {
                    name: "path".to_string(),
                    arguments: vec![Term::variable(2), Term::variable(1)],
                },
            },
        ],
    };

    let reachable_from_root = Clause {
        head: Predicate {
            name: "reachable_from_root".to_string(),
            arguments: vec![Term::variable(0)],
        },
        body: vec![Goal {
            predicate: Predicate {
                name: "path".to_string(),
                arguments: vec![Term::atom("root"), Term::variable(0)],
            },
        }],
    };

    let depth_2_rule = Clause {
        head: Predicate {
            name: "depth_2".to_string(),
            arguments: vec![Term::variable(0)],
        },
        body: vec![
            Goal {
                predicate: Predicate {
                    name: "connects".to_string(),
                    arguments: vec![Term::variable(1), Term::variable(0)],
                },
            },
            Goal {
                predicate: Predicate {
                    name: "connects".to_string(),
                    arguments: vec![Term::atom("root"), Term::variable(1)],
                },
            },
        ],
    };

    let depth_3_rule = Clause {
        head: Predicate {
            name: "depth_3".to_string(),
            arguments: vec![Term::variable(0)],
        },
        body: vec![
            Goal {
                predicate: Predicate {
                    name: "connects".to_string(),
                    arguments: vec![Term::variable(1), Term::variable(0)],
                },
            },
            Goal {
                predicate: Predicate {
                    name: "depth_2".to_string(),
                    arguments: vec![Term::variable(1)],
                },
            },
        ],
    };

    let mut kb = KnowledgeBase::new();
    for fact in connect_facts {
        kb.add_clause(fact);
    }
    kb.add_clause(path_base);
    kb.add_clause(path_recursive);
    kb.add_clause(reachable_from_root);
    kb.add_clause(depth_2_rule);
    kb.add_clause(depth_3_rule);

    // Test reachable_from_root(?0) - should find all nodes reachable from root
    let reachable_query = Goal {
        predicate: Predicate {
            name: "reachable_from_root".to_string(),
            arguments: vec![Term::variable(0)],
        },
    };

    let mut solver = Solver::new(&kb);
    let mut goal_state = solver.create_goal_state(reachable_query);

    let mut reachable_solutions = Vec::new();
    while let Some(solution) = solver.pull_next_goal(&mut goal_state) {
        reachable_solutions.push(solution);
    }

    // Should find all 11 nodes (excluding root itself):
    // a1,a2,b1,b2,b3,c1,c2,c3,d1,d2,d3
    assert_eq!(reachable_solutions.len(), 11);

    let expected_reachable =
        ["a1", "a2", "b1", "b2", "b3", "c1", "c2", "c3", "d1", "d2", "d3"];

    for node in expected_reachable {
        let expected = Substitution {
            mapping: [(0, Term::atom(node))].into_iter().collect(),
        };
        assert!(
            reachable_solutions.contains(&expected),
            "Missing reachable node: {node}"
        );
    }

    // Test depth_2(?0) - should find b1, b2, b3
    let depth_2_query = Goal {
        predicate: Predicate {
            name: "depth_2".to_string(),
            arguments: vec![Term::variable(0)],
        },
    };

    let mut goal_state = solver.create_goal_state(depth_2_query);

    let mut depth_2_solutions = Vec::new();
    while let Some(solution) = solver.pull_next_goal(&mut goal_state) {
        depth_2_solutions.push(solution);
    }

    assert_eq!(depth_2_solutions.len(), 3);
    let expected_depth_2 = ["b1", "b2", "b3"];

    for node in expected_depth_2 {
        let expected = Substitution {
            mapping: [(0, Term::atom(node))].into_iter().collect(),
        };
        assert!(
            depth_2_solutions.contains(&expected),
            "Missing depth-2 node: {node}"
        );
    }

    // Test depth_3(?0) - should find c1, c2, c3
    let depth_3_query = Goal {
        predicate: Predicate {
            name: "depth_3".to_string(),
            arguments: vec![Term::variable(0)],
        },
    };

    let mut goal_state = solver.create_goal_state(depth_3_query);

    let mut depth_3_solutions = Vec::new();
    while let Some(solution) = solver.pull_next_goal(&mut goal_state) {
        depth_3_solutions.push(solution);
    }

    assert_eq!(depth_3_solutions.len(), 3);
    let expected_depth_3 = ["c1", "c2", "c3"];

    for node in expected_depth_3 {
        let expected = Substitution {
            mapping: [(0, Term::atom(node))].into_iter().collect(),
        };
        assert!(
            depth_3_solutions.contains(&expected),
            "Missing depth-3 node: {node}"
        );
    }
}
