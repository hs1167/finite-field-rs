# finite-field-rust

A minimal and rigorous implementation of **Finite Field arithmetic** in Rust. 

## Scope
This repository is an implementation-oriented exercise connecting abstract algebra with cryptographic programming foundations.

- Arithmetic over prime fields ($\mathbb{F}_p$)
- Addition, subtraction, multiplication (handling overflows)
- Exponentiation and Inversion via [Fermat's Little Theorem](https://en.wikipedia.org/wiki/Fermat%27s_little_theorem)
- Compact algebraic tests (group axioms, field properties)

## Mathematical Context
The goal is to implement a safe wrapper for `u64` values to ensure all operations remain within the field. 

For the initial implementation, we use a prime $p$ suitable for 64-bit architectures, such as the Goldilocks prime: $p = 2^{64} - 2^{32} + 1$.

## Goal
The focus is on safety, performance, and a deep understanding of how mathematical invariants are preserved in systems programming.
