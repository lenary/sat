// Copyright Sam Elliott
// Dual-Licensed under the MIT License or the Apache License, Version 2.0.
// See COPYRIGHT for details.
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::assumptions as a;
use super::clause as cls;
use super::literal as lit;
use super::pure_literal_elimination;
use super::unit_propagation;

use std::collections::BTreeSet;

type Problem<'a> = &'a [cls::Clause];
type Solution = Vec<lit::Literal>;

/// Check a set of clauses are satisfiable, using the DPLL algorithm.
///
/// The DPLL Algorihm does:
/// - Checks for sat/unsat
/// - Performs unit propagation
/// - Performs pure literal elimination
/// - Makes guesses, and rolls them back only one step if they are wrong.
///
/// It does not:
/// - "Backjump" multiple levels on conflicts.
/// - "Learn" new clauses based on conflicting assumptions.
///
/// This makes it marginally smarter than just guessing and rolling back on an
/// incorrect guess.
///
/// Returns:
/// - None if `unsat`
/// - Some(Solution) if `sat`
pub fn satisfiable(clauses: Problem) -> Option<Solution> {
    let mut assumptions = a::AssumptionStore::new();

    let mut var_set: BTreeSet<lit::Variable> = BTreeSet::new();
    for c in clauses {
        for l in c.iter() {
            var_set.insert(l.variable());
        }
    }

    loop {
        // `assumptions` is a consistent set of literals
        if clauses.iter().all(|c| c.is_satisfied(&assumptions)) {
            break;
        }

        // `assumptions` generates an unsatisfiable clause
        if clauses.iter().any(|c| c.is_unsatisfiable(&assumptions)) {
            return None;
        }

        // Choose a literal for the next two steps.
        if let Some(next_var) = get_next_variable(&var_set, &assumptions) {
            // At this point, we need to do unit propagation then pure literal
            // assignment under `v` or `~v`. This is called "guessing", as we don't
            // know which of `v` or `~v` will be correct.

            match make_guess(next_var, clauses, &mut assumptions) {
                // No conflicts, keep guess and see if we're done or we need to
                // continue.
                true => continue,

                // Guess generated a conflict, fallthrough to the code below which
                // makes the opposite guess
                false => (),
            }

            match make_guess(next_var.negate(), clauses, &mut assumptions) {
                // No conflicts, keep guess, and see if we're done or we need to
                // continue.
                true => continue,

                // Guess generated a conflict. So did the previous one, so
                // there's no evaluation which can give us a correct assumption.
                false => return None,
            }
        } else {
            // No more unknown variables, finished!
            break;
        }
    }

    return Some(assumptions.get_solution());
}

fn get_next_variable(
    candidates: &BTreeSet<lit::Variable>,
    assumptions: &a::AssumptionStore,
) -> Option<lit::Literal> {
    // We've done absolutely zero tuning of the selection order here
    candidates
        .iter()
        .copied()
        .find(|v| assumptions.get_var(*v) == a::Assumption::Unknown)
        .map(|var| lit::Literal::new(var, true))
}

fn make_guess(
    new_lit: lit::Literal,
    clauses: Problem,
    assumptions: &mut a::AssumptionStore,
) -> bool {
    assumptions.new_inference();

    // Try unit propagation.
    if !unit_propagation::propagate(clauses, new_lit, assumptions) {
        // There's a conflict, rollback and try other guess
        assumptions.rollback_inference();
        return false;
    }

    // Eliminate pure literals (this cannot generate conflicts).
    pure_literal_elimination::eliminate(clauses, assumptions);

    // No conflicts found in those two steps, continue.
    true
}
