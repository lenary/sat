// Copyright Sam Elliott
// Dual-Licensed under the MIT License or the Apache License, Version 2.0.
// See COPYRIGHT for details.
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A SAT solver, written in pure Rust.
//!
//! It won't be that good, go look somewhere else.

// Core data structures (this may change)
pub mod clause;
pub mod literal;

// Utilities
pub mod assumptions;

// Formats
pub mod dimacs;

// Free Algorithms
pub mod dpll;
pub mod pure_literal_elimination;
pub mod unit_propagation;

#[cfg(test)]
mod tests {}
