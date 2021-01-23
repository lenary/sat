// Copyright Sam Elliott
// Dual-Licensed under the MIT License or the Apache License, Version 2.0.
// See COPYRIGHT for details.
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::clause as c;
use super::literal as l;

use std::io::{BufRead, Result, Write};
use std::mem::swap;

/// A DIMACS-CNF File Parser
///
/// The format is supposed to be stupid simple:
/// - `c ...` is a comment
/// - `p cnf 3 4` means a problem with 3 variables and 4 clauses
/// - `34 -2 83 0` means a clause. Negated ints are negated literals. 0 means end
///   of clause.
pub fn parse<R: BufRead>(buf: R) -> Option<Vec<c::Clause>> {
    let mut found_problem = false;
    // let mut variables_len = 0;
    let mut clauses_left = 0;
    let mut current_clause = c::Clause::new();
    let mut all_clauses = vec![];

    for res in buf.lines() {
        if let Ok(mut line) = res {
            line.make_ascii_lowercase();

            if line.starts_with("c") {
                // Comment line
                continue;
            } else if line.starts_with("p") {
                // This parser bails out if we have multiple `p` lines in an input.
                if found_problem {
                    return None;
                }

                let parts: Vec<_> = line.split_ascii_whitespace().collect();
                if parts.len() != 4 {
                    return None;
                }
                if parts[0] != "p" || parts[1] != "cnf" {
                    return None;
                }

                // We don't really care about number of variables.
                // if let Ok(len) = parts[2].parse() {
                //     variables_len = len
                // } else {
                //     return None;
                // }

                if let Ok(len) = parts[3].parse() {
                    clauses_left = len
                } else {
                    return None;
                }

                found_problem = true;
            } else {
                // We can only parse lines that are entirely digits, `-` or space.
                if !line
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '-' || c.is_ascii_whitespace())
                    || clauses_left == 0
                {
                    return None;
                }

                for dimacs_lit_str in line.split_ascii_whitespace() {
                    if let Ok(dimacs_lit) = dimacs_lit_str.parse::<i32>() {
                        if let Some(lit) = l::Literal::from_dimacs(dimacs_lit) {
                            current_clause.add_literal(lit)
                        } else {
                            // `from_dimacs` returns None if we pass it `0`, which
                            // DIMACS uses to signal end of clause.

                            // Create a new clause for future literals
                            let mut clause = c::Clause::new();
                            // Replace current clause
                            swap(&mut current_clause, &mut clause);
                            // Ensure we add the current clause to the result
                            all_clauses.push(clause);
                            // Note that we parsed another clause.
                            clauses_left -= 1;
                        }
                    } else {
                        return None;
                    }
                }
            }
        } else {
            return None;
        }
    }

    // If we get to here with one clause left, it likely means the current
    // clause was never terminated with 0. If so, finish it now.
    if clauses_left == 1 {
        all_clauses.push(current_clause);
        clauses_left -= 1;
    }

    if clauses_left > 0 {
        return None;
    }

    return Some(all_clauses);
}

/// A DIMACS Solution Printer
///
/// The format is supposed to be stupid simple:
/// - `c ...` is a comment
/// - `s <RESULT>` says whether you managed
/// - `v 34 -2 83 0` gives the resulting values of variables, terminated by 0.
pub fn print<W: Write>(buf: &mut W, soln: Option<Vec<l::Literal>>) -> Result<()> {
    match soln {
        None => writeln!(buf, "s UNSATISFIABLE"),
        Some(soln) => {
            writeln!(buf, "s SATISFIABLE")?;
            write!(buf, "v ")?;
            for l in soln {
                write!(buf, "{} ", l.to_dimacs())?;
            }
            writeln!(buf, "0")
        }
    }
}
