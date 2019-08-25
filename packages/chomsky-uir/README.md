# chomsky-uir

Universal Intermediate Representation (UIR) for the Chomsky framework, powered by E-Graphs.

## Overview

`chomsky-uir` is the core optimization engine of Project Chomsky. Unlike traditional linear IRs, UIR uses Equality Graphs (E-Graphs) to store and reason about multiple equivalent forms of a program simultaneously. This allows for exhaustive exploration of optimization opportunities without being limited by phase ordering.

## Features

- **Intent Capture**: Provides `IntentBuilder` to formalize computational intents into rewriteable expressions.
- **E-Graph Implementation**: High-performance E-Graph structure for managing equivalence classes of program fragments.
- **Constraint Analysis**: Built-in analysis framework to track hardware constraints, type information, and semantic properties on the E-Graph.
- **Lowering**: Robust logic to reduce high-level intents into UIR primitives (`IKun` operators).
- **Register Allocation**: Includes a `LinearScanAllocator` for mapping UIR virtual registers to physical registers or stack slots.

## Module Structure

- `egraph.rs`: The heart of the IR, managing e-classes and nodes.
- `intent.rs`: Defines the `IKun` operator set, the primary language of the UIR.
- `builder.rs`: Fluent API for constructing UIR from higher-level representations.
- `analysis.rs`: Framework for running monotonic analyses (like `ConstraintAnalysis`) over the E-Graph.
- `regalloc.rs`: Register allocation algorithms for backend emission.

## Usage

```rust
use chomsky_uir::{EGraph, IntentBuilder, IKun};

let mut egraph = EGraph::new();
let mut builder = IntentBuilder::new(&mut egraph);

// Construct a simple addition
let a = builder.constant(1);
let b = builder.constant(2);
let result = builder.add(a, b);
```

