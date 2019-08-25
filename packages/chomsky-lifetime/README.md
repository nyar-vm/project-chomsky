# chomsky-lifetime

Lifetime analysis and resource management for the Chomsky framework.

## Overview

`chomsky-lifetime` provides tools for reasoning about resource ownership and lifetimes within the `chomsky-uir` E-Graph. It implements ownership analysis to determine where resources are created, shared, and consumed, enabling the compiler to insert appropriate resource management instructions (e.g., Reference Counting or explicit Drops).

## Features

- **Ownership Analysis**: An E-Graph analysis that infers the ownership state (Borrowed, Shared, Owned, Linear) of program nodes.
- **Resource Tracking**: Identifies resource creation points (Sources) and consumption points (Sinks).
- **Lattice-Based Merging**: Uses a lattice model to merge ownership states across equivalent program forms.
- **Explicit Management**: Provides a foundation for inserting lifecycle instructions into the optimized program.

## Core Concepts

### Ownership State
The analysis tracks the following states:
- **Borrowed**: Temporary access without ownership.
- **Shared**: Shared ownership (e.g., Reference Counted).
- **Owned**: Unique ownership that must be managed.
- **Linear**: Strict single-use ownership.

### OwnershipAnalysis
The core E-Graph analysis implementation that propagates ownership information through the graph nodes based on their semantics and explicit constraints.

## Usage

```rust
use chomsky_lifetime::OwnershipAnalysis;
use chomsky_uir::EGraph;

let mut egraph = EGraph::<_, OwnershipAnalysis>::new();
// ... optimize and analyze ...

let results = chomsky_lifetime::LifetimePass::analyze(&egraph);
```
