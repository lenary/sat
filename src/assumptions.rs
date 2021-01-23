// Copyright Sam Elliott
// Dual-Licensed under the MIT License or the Apache License, Version 2.0.
// See COPYRIGHT for details.
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::literal::*;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Assumption {
    Unknown,
    Assume(bool),
}

pub struct AssumptionStore {
    // A list of assumptions in order, so we can roll them back when we run into
    // an incorrect guess.
    assumptions: Vec<Literal>,

    // A list of indexes into `vars`, which are used as rollback boundaries.
    // Currently we only rollback one transaction at a time, later we might go
    // further.
    rollback_boundaries: Vec<usize>,
}

impl AssumptionStore {
    pub fn new() -> AssumptionStore {
        AssumptionStore {
            assumptions: vec![],
            rollback_boundaries: vec![],
        }
    }

    // We iterate backwards, because we're most likely to have added the
    // variable recently.
    fn iter(&self) -> impl Iterator<Item = Literal> + '_ {
        self.assumptions.iter().rev().copied()
    }

    pub fn get_var(&self, var: Variable) -> Assumption {
        match self.iter().find(|l| l.variable() == var) {
            Some(l) => Assumption::Assume(l.polarity()),
            None => Assumption::Unknown,
        }
    }

    // Get the assumption for a literal. This respects the polarity of the literal,
    // if an assumption is present.
    pub fn get_lit(&self, lit: Literal) -> Assumption {
        match self.iter().find(|l| l.variable() == lit.variable()) {
            Some(l) => Assumption::Assume(l.polarity() == lit.polarity()),
            None => Assumption::Unknown,
        }
    }

    // assume `lit` is true. returns `false` if there's a conflict by making this assumption.
    pub fn assume(&mut self, lit: Literal) -> bool {
        match self.iter().find(|l| l.variable() == lit.variable()) {
            Some(prev) if prev.polarity() != lit.polarity() => {
                // This conflicts with a previous assumption.
                return false;
            }
            // No conflict
            _ => (),
        }

        self.assumptions.push(lit);
        true
    }

    pub fn new_inference(&mut self) {
        self.rollback_boundaries.push(self.assumptions.len())
    }

    pub fn rollback_inference(&mut self) {
        match self.rollback_boundaries.pop() {
            Some(idx) => {
                self.assumptions.truncate(idx);
            }
            None => {
                panic!("You can only rollback as many times as you started a new inference.");
            }
        }
    }

    pub fn get_solution(self) -> Vec<Literal> {
        self.assumptions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(l: i32) -> Literal {
        Literal::from_dimacs(l).unwrap()
    }

    #[test]
    fn get_empty() {
        let one = lit(1);
        let assumptions = AssumptionStore::new();

        assert_eq!(assumptions.get_var(one.variable()), Assumption::Unknown);
        assert_eq!(assumptions.get_lit(one), Assumption::Unknown);
    }

    #[test]
    fn get_lit_true() {
        let one = lit(1);
        let mut assumptions = AssumptionStore::new();

        assumptions.assume(one);

        assert_eq!(
            assumptions.get_var(one.variable()),
            Assumption::Assume(true)
        );
        assert_eq!(assumptions.get_lit(one), Assumption::Assume(true));
    }

    #[test]
    fn get_lit_false() {
        let one = lit(1);
        let mut assumptions = AssumptionStore::new();

        assumptions.assume(one.negate());

        assert_eq!(
            assumptions.get_var(one.variable()),
            Assumption::Assume(false)
        );

        assert_eq!(assumptions.get_lit(one.negate()), Assumption::Assume(true));
        assert_eq!(assumptions.get_lit(one), Assumption::Assume(false));
    }

    #[test]
    fn test_rollback() {
        let mut assumptions = AssumptionStore::new();

        assumptions.new_inference();

        assert!(assumptions.assume(lit(1)));

        assumptions.new_inference();

        assert!(assumptions.assume(lit(2)));

        assumptions.new_inference();

        assert!(assumptions.assume(lit(3)));
        assert!(assumptions.assume(lit(4)));

        assumptions.rollback_inference();

        assert!(assumptions.assume(lit(-3)));

        assert_eq!(assumptions.get_lit(lit(1)), Assumption::Assume(true));
        assert_eq!(assumptions.get_lit(lit(2)), Assumption::Assume(true));
        assert_eq!(assumptions.get_lit(lit(3)), Assumption::Assume(false));
        assert_eq!(assumptions.get_lit(lit(4)), Assumption::Unknown);

        assumptions.rollback_inference();

        assert_eq!(assumptions.get_lit(lit(1)), Assumption::Assume(true));
        assert_eq!(assumptions.get_lit(lit(2)), Assumption::Unknown);

        assumptions.rollback_inference();

        assert_eq!(assumptions.get_lit(lit(1)), Assumption::Unknown);
    }

    #[test]
    fn test_solution() {
        let mut assumptions = AssumptionStore::new();

        assumptions.new_inference();

        assert!(assumptions.assume(lit(1)));

        assumptions.new_inference();

        assert!(assumptions.assume(lit(2)));

        assumptions.rollback_inference();

        let soln = assumptions.get_solution();

        assert!(soln.contains(&lit(1)));
        assert!(!soln.contains(&lit(2)));
    }
}
