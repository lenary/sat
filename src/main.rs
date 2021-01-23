// Copyright Sam Elliott
// Dual-Licensed under the MIT License or the Apache License, Version 2.0.
// See COPYRIGHT for details.
// SPDX-License-Identifier: MIT OR Apache-2.0

use sat::dimacs;
use sat::dpll;

use std::io;

// This is written to be as stupid-simple as possible.
pub fn main() -> std::io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    if let Some(clauses) = dimacs::parse(stdin.lock()) {
        let soln = dpll::satisfiable(&clauses);

        dimacs::print(&mut stdout.lock(), soln)?;
    } else {
        println!("c No Input Received");
    }

    Ok(())
}
