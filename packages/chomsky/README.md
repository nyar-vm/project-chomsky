# Project Chomsky

Main entry point and facade for the Chomsky optimization framework.

## Overview

Project Chomsky is a next-generation compiler optimization framework based on the principle of "Universal Grammar." It uses Equality Graphs (E-Graphs) and Equality Saturation to perform exhaustive, non-destructive program transformations, targeting diverse hardware from CPUs to GPUs and specialized accelerators.

The `chomsky` crate serves as the primary entry point for users, re-exporting the core functionality of the entire framework and providing high-level APIs for compilation, optimization, and verification.

## Features

- **Unified Facade**: Easy access to all sub-packages (`chomsky-uir`, `chomsky-rules`, `chomsky-emit`, etc.).
- **High-Level API**: Simplified interfaces for the entire compilation pipeline.
- **Multi-Target Support**: Built-in support for various backends including JVM, WASI, x86_64, and more.
- **Verification Tools**: Utilities to check the correctness and equivalence of optimized programs.
- **Extensible Architecture**: Modular design allowing for custom rules, cost models, and backends.

## Sub-Packages

Project Chomsky is composed of several specialized crates:
- `chomsky-types`: Core definitions (Intent, Context).
- `chomsky-uir`: E-Graph based Intermediate Representation.
- `chomsky-rule-engine`: Saturation-based rewrite engine.
- `chomsky-rules`: Collection of optimization rules.
- `chomsky-cost`: Multi-dimensional cost modeling.
- `chomsky-extract`: Optimal program selection from E-Graphs.
- `chomsky-emit`: Code generation for target assemblers.
- `chomsky-glue`: Interoperability and FFI support.

## Usage

For most users, depending on the `chomsky` crate is sufficient to build and run the optimization pipeline.

```rust
use chomsky::prelude::*;

// High-level compilation pipeline example
let mut pipeline = Pipeline::new();
pipeline.add_optimization_pass(DefaultRules);
let artifact = pipeline.compile(my_intent)?;
```

## Documentation

For more detailed information, please visit the [Official Documentation](https://gaia-assembly.netlify.app/chomsky/).
