# finite-field-rust

A minimal and rigorous implementation of **Finite Field arithmetic** in Rust.

## Scope

This repository is an implementation-oriented exercise connecting abstract algebra with cryptographic programming foundations.

* Arithmetic over the Goldilocks prime field ($F_p$)
* Branchless addition and subtraction (managing CPU carry/borrow flags)
* Optimized 128-bit multiplication leveraging affine projection (inspired by Plonky2), avoiding hardware Euclidean division
* Exponentiation, Inversion (via Fermat's Little Theorem), and Field Division
* Safe byte serialization (`to_bytes`/`from_bytes`) for external cryptographic use
* Comprehensive algebraic tests (ring/field axioms, zero-division, hardware boundary cases)

> **Note on Documentation:** The codebase is heavily commented by design. It serves as an educational resource, explicitly detailing the mathematical proofs and algebraic tricks (such as the 128-bit reduction logic and affine projections) directly alongside the Rust implementation.

## Mathematical Context

The goal is to implement a safe wrapper for `u64` values to ensure all operations remain strictly within the field.

For this implementation, we use the Goldilocks prime, which is highly optimized for 64-bit architectures:
$p = 2^{64} - 2^{32} + 1$.

Because $2^{64} \equiv 2^{32} - 1 \pmod p$, we can optimize reductions (such as in 128-bit multiplication) using simple bitwise shifts and additions rather than costly modulo operations.

## Goal & Disclaimer

The focus is on safety, performance, and a deep understanding of how mathematical invariants are preserved in systems programming.

**Disclaimer:** This project is primarily a personal learning exercise. While I strive for mathematical rigor, this codebase has not been audited for production use and may contain bugs or edge-case oversights. If you spot any vulnerabilities, mathematical inaccuracies, or have suggestions for improvement, please don't hesitate to open an issue or a PR. Feedback and corrections are highly appreciated!
