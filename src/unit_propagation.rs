// Copyright Sam Elliott
// Dual-Licensed under the MIT License or the Apache License, Version 2.0.
// See COPYRIGHT for details.
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::assumptions as a;
use super::clause as c;
use super::literal as lit;

use std::collections::VecDeque;

/// Takes a set of clauses, a literal to propagate, and a set of assumptions;
/// updates assumptions, returns `false` if a conflict was found
pub fn propagate(
    clauses: &[c::Clause],
    lit: lit::Literal,
    assumptions: &mut a::AssumptionStore,
) -> bool {
    let mut worklist: VecDeque<lit::Literal> = VecDeque::new();
    worklist.push_back(lit);

    while let Some(current_lit) = worklist.pop_front() {
        if !assumptions.assume(current_lit) {
            // assuming `current_lit` generates a conflict, making the system unsatisfiable.
            return false;
        }

        for cls in clauses {
            if cls.is_unsatisfiable(assumptions) {
                // `cls` is now unsatisfiable, so the whole system is unsatisfiable.
                return false;
            }

            match cls.get_unit(assumptions) {
                // new unit clause, push the literal onto the worklist.
                Some(lit) => worklist.push_back(lit),
                // no conflict but also no additional information we can use.
                None => (),
            }
        }
    }

    // the loop will have been broken if we generated a conflict
    return true;
}

#[cfg(test)]
mod tests {

    use super::*;

    fn lit(l: i32) -> lit::Literal {
        lit::Literal::from_dimacs(l).unwrap()
    }

    macro_rules! c {
        [] => ( c::Clause::new() );
        [$($e:expr),+ $(,)?] => ({
            let mut clause = c!();
            for l in [$($e),+].iter().copied() {
                clause.add_literal(lit(l));
            }
            clause
        })
    }

    macro_rules! a {
        [] => ( a::AssumptionStore::new() );
        [$($e:expr),+ $(,)?] => ({
            let mut assumptions = a!();
            for l in [$($e),+].iter().copied() {
                assert!(assumptions.assume(lit(l)));
            }
            assumptions
        })
    }

    // Tests that we correctly detect conflicts.
    #[test]
    fn prop_zero() {
        let clauses = vec![c![]];
        let mut assumptions = a![];

        assert!(!propagate(&clauses, lit(1), &mut assumptions));
    }

    // Tests that we can resolve clauses simply.
    #[test]
    fn prop_one() {
        let clauses = vec![c![1, 2, 3]];
        let mut assumptions = a![];

        // propagation introduces no conflicts
        assert!(propagate(&clauses, lit(1), &mut assumptions));

        // Propagation only introduced one literal.
        assert_eq!(assumptions.get_lit(lit(1)), a::Assumption::Assume(true));
        assert_eq!(assumptions.get_lit(lit(2)), a::Assumption::Unknown);
        assert_eq!(assumptions.get_lit(lit(3)), a::Assumption::Unknown);

        // the only clause is now satisfied
        assert!(clauses[0].is_satisfied(&assumptions));
    }

    // Tests propagation of a single literal that doesn't resolve the clauses.
    #[test]
    fn prop_one_neg() {
        let clauses = vec![c![1, 2, 3]];
        let mut assumptions = a![];

        assert!(propagate(&clauses, lit(-1), &mut assumptions));

        // propagation only introduced one assumption.
        assert_eq!(assumptions.get_lit(lit(1)), a::Assumption::Assume(false));
        assert_eq!(assumptions.get_lit(lit(2)), a::Assumption::Unknown);
        assert_eq!(assumptions.get_lit(lit(3)), a::Assumption::Unknown);

        // the only clause is still unresolved
        assert!(!clauses[0].is_satisfied(&assumptions));
        assert!(!clauses[0].is_unsatisfiable(&assumptions));
    }

    // Tests that we can propagate to completion.
    #[test]
    fn prop_two() {
        let clauses = vec![c![1, 2, 3]];
        let mut assumptions = a![-1];

        assert!(propagate(&clauses, lit(-2), &mut assumptions));

        // propagation only introduced two assumptions
        assert_eq!(assumptions.get_lit(lit(1)), a::Assumption::Assume(false));
        assert_eq!(assumptions.get_lit(lit(2)), a::Assumption::Assume(false));
        assert_eq!(assumptions.get_lit(lit(3)), a::Assumption::Assume(true));

        // the clause is satisfied
        assert!(clauses[0].is_satisfied(&assumptions));
    }

    // Tests propagations actually propagate through multiple clauses.
    #[test]
    fn prop_multi_clause() {
        let clauses = vec![c![-1, 2], c![-2, 3], c![-3, 4]];
        let mut assumptions = a![];

        assert!(propagate(&clauses, lit(1), &mut assumptions));

        // propagation only introduced one assumption.
        assert_eq!(assumptions.get_lit(lit(1)), a::Assumption::Assume(true));
        assert_eq!(assumptions.get_lit(lit(2)), a::Assumption::Assume(true));
        assert_eq!(assumptions.get_lit(lit(3)), a::Assumption::Assume(true));
        assert_eq!(assumptions.get_lit(lit(4)), a::Assumption::Assume(true));

        // all clauses are now satisfied.
        assert!(clauses[0].is_satisfied(&assumptions));
        assert!(clauses[1].is_satisfied(&assumptions));
        assert!(clauses[2].is_satisfied(&assumptions));
    }

    // Tests that existing unit clauses will be propagated, even if they don't
    // contain the requested literal.
    #[test]
    fn prop_external_lit() {
        let clauses = vec![c![1], c![2]];
        let mut assumptions = a![];

        assert!(propagate(&clauses, lit(3), &mut assumptions));

        assert_eq!(assumptions.get_lit(lit(1)), a::Assumption::Assume(true));
        assert_eq!(assumptions.get_lit(lit(2)), a::Assumption::Assume(true));

        assert!(clauses[0].is_satisfied(&assumptions));
        assert!(clauses[1].is_satisfied(&assumptions));
    }

    // Tests that we detect a conflict *during* propagation
    #[test]
    fn prop_detect_conflict() {
        let clauses = vec![c![-1, 2], c![-2, 3]];
        let mut assumptions = a![-3];

        assert!(!propagate(&clauses, lit(1), &mut assumptions));

        // Conflicts don't affect existing assumptions
        assert_eq!(assumptions.get_lit(lit(3)), a::Assumption::Assume(false));

        assert!(clauses[1].is_unsatisfiable(&assumptions));
    }

    // Tests an example found on wikipedia: https://en.wikipedia.org/wiki/Unit_propagation
    #[test]
    fn prop_example_wikipedia() {
        // 1 = a, 2 = b, 3 = c, 4 = d
        let clauses = vec![c![1, 2], c![-1, 3], c![-3, 4], c![1]];
        let mut assumptions = a![];

        assert!(propagate(&clauses, lit(1), &mut assumptions));

        assert!(clauses[0].is_satisfied(&assumptions));
        assert!(clauses[1].is_satisfied(&assumptions));
        assert!(clauses[2].is_satisfied(&assumptions));
        assert!(clauses[3].is_satisfied(&assumptions));

        // 1 implies 3; 2 may get set, but it's not necessary.
        assert_eq!(assumptions.get_lit(lit(1)), a::Assumption::Assume(true));
        assert_eq!(assumptions.get_lit(lit(3)), a::Assumption::Assume(true));
    }
}
