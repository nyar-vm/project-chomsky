# chomsky-concretize

Concrete realization of UIR into target architecture models for the Chomsky framework.

## Overview

`chomsky-concretize` is the bridge between the high-level Universal Intermediate Representation (UIR) and the specific architectural models of backends. It handles layout decisions, tiling, unrolling, and vectorization based on the information gathered during optimization.

## Features

- **Layout Optimization**: Decisions between Structure of Arrays (SoA) and Array of Structures (AoS) based on access patterns and hardware capabilities.
- **Transformation Passes**: Implements logic for loop tiling, unrolling, and vectorization.
- **Concretization Analysis**: An E-Graph analysis that tracks and propagates layout and transformation decisions through the program graph.
- **Rule-Based realization**: Uses rewrite rules to transform generic UIR nodes into architecture-specific forms (e.g., `TiledMap`, `VectorizedMap`).

## Core Concepts

### ConcretizationData
Metadata tracked during the concretization process:
- `layout`: Current memory layout (SoA/AoS).
- `tiling_factor`: Factor for loop tiling.
- `unroll_factor`: Factor for loop unrolling.
- `vector_width`: Width for SIMD vectorization.

### LayoutOptimizer
A rewrite rule that explores different memory layouts to find the most efficient representation for a given computational intent.

## Usage

```rust
use chomsky_concretize::{ConcretizationAnalysis, LayoutOptimizer};
use chomsky_rule_engine::{RewriteRegistry, RuleCategory};
use chomsky_uir::EGraph;

let mut egraph = EGraph::<_, ConcretizationAnalysis>::new();
let mut registry = RewriteRegistry::new();

registry.register(RuleCategory::Concretization, Box::new(LayoutOptimizer));
```
