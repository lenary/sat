// Copyright Sam Elliott
// Dual-Licensed under the MIT License or the Apache License, Version 2.0.
// See COPYRIGHT for details.
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::assumptions as ac;
use super::literal as lit;

/// A Clause is a *disjunction* of literals, i.e. `x OR y OR z`.
#[derive(Debug)]
pub struct Clause(Vec<lit::Literal>);

impl Clause {
    pub fn new() -> Clause {
        Clause(vec![])
    }

    pub fn iter(&self) -> std::slice::Iter<'_, lit::Literal> {
        self.0.iter()
    }

    /// Searches for `var` in self, returning the index it appears at, and the
    /// literal containing the variable because different algorithms need both
    /// or either of course.
    fn get_this_var(&self, var: lit::Variable) -> Option<(usize, lit::Literal)> {
        self.0
            .iter()
            .copied()
            .enumerate()
            .find(|(_idx, l)| l.variable() == var)
    }

    /// Adds `lit` to the clause
    pub fn add_literal(&mut self, lit: lit::Literal) {
        if let Some((_, l)) = self.get_this_var(lit.variable()) {
            // `lit` is in self, but `-lit` is not, so add it.
            //
            // We canot delete `-lit` because `a OR ~a` is not equivalent to the
            // empty clause.
            if l.polarity() != lit.polarity() {
                self.0.push(lit)
            }
        } else {
            // `lit` or `-lit` is not in self, so add it.
            self.0.push(lit)
        }
    }

    /// Under the given `assumptions`, is there a possibility of satisfying this clause?
    ///
    /// Returns `true` for empty clauses, or clauses where all literals are false.
    pub fn is_unsatisfiable(&self, assumptions: &ac::AssumptionStore) -> bool {
        self.0
            .iter()
            .all(|lit| assumptions.get_lit(*lit) == ac::Assumption::Assume(false))
    }

    /// Under the given `assumptions`, is this clause satisfied?
    pub fn is_satisfied(&self, assumptions: &ac::AssumptionStore) -> bool {
        self.0
            .iter()
            .any(|lit| assumptions.get_lit(*lit) == ac::Assumption::Assume(true))
    }

    /// Under the given `assumptions` is this clause a unit clause? If it is, return the literal
    /// that is a Unit.
    pub fn get_unit(&self, assumptions: &ac::AssumptionStore) -> Option<lit::Literal> {
        // We're going to treat this as a one-element array for storing a possible unknown unit literal.
        let mut unit = None;

        for lit in self.0.iter() {
            match assumptions.get_lit(*lit) {
                // A literal we don't have a value for, keep track of it.
                ac::Assumption::Unknown => {
                    match unit {
                        // First unknown seen, store it.
                        None => unit = Some(*lit),
                        // Seen one unknown, plus this one, not a unit-clause
                        Some(_) => return None,
                    }
                }
                // A literal we know is `true`, so the clause is satisfied, not a unit-clause.
                ac::Assumption::Assume(true) => return None,
                // A literal we know is `false`, keep looking for literals that might be satisfied.
                ac::Assumption::Assume(false) => (),
            }
        }

        return unit;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(l: i32) -> lit::Literal {
        lit::Literal::from_dimacs(l).unwrap()
    }

    fn assumptions() -> ac::AssumptionStore {
        ac::AssumptionStore::new()
    }

    #[test]
    fn empty() {
        let c = Clause::new();
        let assumptions = assumptions();

        assert!(!c.is_satisfied(&assumptions));
        assert!(c.is_unsatisfiable(&assumptions));

        assert_eq!(c.get_unit(&assumptions), None);
    }

    #[test]
    fn unit() {
        let mut c = Clause::new();
        let assumptions = assumptions();

        c.add_literal(lit(1));

        assert!(!c.is_satisfied(&assumptions));
        assert!(!c.is_unsatisfiable(&assumptions));

        assert_eq!(c.get_unit(&assumptions), Some(lit(1)));
    }

    #[test]
    fn two() {
        let mut c = Clause::new();
        let assumptions = assumptions();

        c.add_literal(lit(1));
        c.add_literal(lit(-2));

        assert!(!c.is_satisfied(&assumptions));
        assert!(!c.is_unsatisfiable(&assumptions));

        assert_eq!(c.get_unit(&assumptions), None);
    }

    #[test]
    fn two_satisfied() {
        let mut c = Clause::new();
        let mut assumptions = assumptions();

        c.add_literal(lit(1));
        c.add_literal(lit(-2));

        // adding an assumption that ensures X is satisfied
        assumptions.assume(lit(1));

        assert!(c.is_satisfied(&assumptions));
        assert!(!c.is_unsatisfiable(&assumptions));

        assert_eq!(c.get_unit(&assumptions), None);
    }

    #[test]
    fn two_to_unit() {
        let mut c = Clause::new();
        let mut assumptions = assumptions();

        c.add_literal(lit(1));
        c.add_literal(lit(-2));

        // Adding an assumption in the *wrong* direction.
        assumptions.assume(lit(-1));

        assert!(!c.is_satisfied(&assumptions));
        assert!(!c.is_unsatisfiable(&assumptions));

        assert_eq!(c.get_unit(&assumptions), Some(lit(-2)));
    }

    #[test]
    fn two_unsatisfiable() {
        let mut c = Clause::new();
        let mut assumptions = assumptions();

        c.add_literal(lit(1));
        c.add_literal(lit(-2));

        // Adding an assumption in the *wrong* direction.
        assumptions.assume(lit(-1));
        assumptions.assume(lit(2));

        assert!(!c.is_satisfied(&assumptions));
        assert!(c.is_unsatisfiable(&assumptions));

        assert_eq!(c.get_unit(&assumptions), None);
    }
}
