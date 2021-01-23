// Copyright Sam Elliott
// Dual-Licensed under the MIT License or the Apache License, Version 2.0.
// See COPYRIGHT for details.
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::assumptions as a;
use super::clause as c;
use super::literal as lit;

use std::collections::BTreeMap;

/// "If a propositional variable appears with only one polarity in the formula,
/// it's called `pure`, and it can always be assigned in a way to make all
/// clauses containing it true".
///
/// This takes a list of clauses, and a set of assumptions, and adds assumptions
/// for each pure literal in the clauses.
///
/// This cannot generate conflicts, usually.
pub fn eliminate(clauses: &[c::Clause], assumptions: &mut a::AssumptionStore) {
    let mut var_info = BTreeMap::new();

    for c in clauses {
        // If this clause is satisfied, we don't have to examine it.
        if c.is_satisfied(assumptions) {
            continue;
        }

        for l in c.iter() {
            // We only care about unresolved literals.
            if assumptions.get_lit(*l) == a::Assumption::Unknown {
                var_info
                    .entry(l.variable())
                    .or_insert(SeenVariable::Never)
                    .observe(*l)
            }
        }
    }

    for (var, seen) in var_info {
        match seen {
            SeenVariable::Only(polarity) => {
                // This should not produce a conflict.
                assert!(assumptions.assume(lit::Literal::new(var, polarity)));
            }
            _ => (),
        }
    }
}

// This is a quick and dumb lattice to represent what we've seen so far.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum SeenVariable {
    Never,
    Only(bool),
    Both,
}

impl SeenVariable {
    fn observe(&mut self, lit: lit::Literal) {
        match *self {
            SeenVariable::Never => *self = SeenVariable::Only(lit.polarity()),
            SeenVariable::Only(prev) if prev != lit.polarity() => *self = SeenVariable::Both,
            _ => (),
        }
    }
}
