// Copyright Sam Elliott
// Dual-Licensed under the MIT License or the Apache License, Version 2.0.
// See COPYRIGHT for details.
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Solver Types
//
// - Variables
// - Literals
// - Whatever Else

/// A Variable in a SAT problem.
///
/// This is a symbolic variable that will stand for true or false in a solution.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct Variable(u32);

/// A Literal in a SAT clause.
///
/// A literal is either a Variable or a Negated Variable. This negation is
/// represented by `polarity`.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Literal {
    variable: Variable,
    polarity: bool,
}

impl Literal {
    pub fn new(variable: Variable, polarity: bool) -> Literal {
        Literal { variable, polarity }
    }

    /// This turns a DIMACS-representation of a literal (where 0 is the "end of
    /// clause", and "-1" means variable 1, but negated) into a literal as we
    /// represent it in this sytem.
    pub fn from_dimacs(dimacs: i32) -> Option<Literal> {
        let is_positive = dimacs > 0;
        dimacs.checked_abs().filter(|v| *v != 0).map(|v| Literal {
            polarity: is_positive,
            variable: Variable(v as u32),
        })
    }

    pub fn to_dimacs(&self) -> String {
        match self.polarity() {
            true => format!("{}", self.variable().0),
            false => format!("-{}", self.variable().0),
        }
    }

    pub fn negate(&self) -> Literal {
        Literal {
            variable: self.variable,
            polarity: !self.polarity,
        }
    }

    pub fn variable(&self) -> Variable {
        self.variable
    }

    pub fn polarity(&self) -> bool {
        self.polarity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_var() {
        assert_eq!(
            Literal {
                variable: Variable(1),
                polarity: true
            }
            .variable(),
            Variable(1)
        );
    }

    #[test]
    fn get_polarity() {
        assert_eq!(
            Literal {
                variable: Variable(1),
                polarity: false
            }
            .polarity(),
            false
        );
    }

    #[test]
    fn zero_lit() {
        assert_eq!(None, Literal::from_dimacs(0));
    }

    #[test]
    fn pos_lit() {
        assert_eq!(
            Some(Literal {
                polarity: true,
                variable: Variable(1),
            }),
            Literal::from_dimacs(1)
        )
    }

    #[test]
    fn neg_lit() {
        assert_eq!(
            Some(Literal {
                polarity: false,
                variable: Variable(2),
            }),
            Literal::from_dimacs(-2)
        )
    }

    #[test]
    fn negate() {
        let lit = Literal {
            polarity: true,
            variable: Variable(3),
        };
        assert_eq!(
            Literal {
                polarity: false,
                variable: Variable(3),
            },
            lit.negate()
        )
    }
}
