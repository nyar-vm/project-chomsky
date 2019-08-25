# chomsky-rule-engine

A powerful rewrite rule engine for the Chomsky optimization framework.

## Overview

`chomsky-rule-engine` is responsible for applying algebraic transformations and optimizations to the `chomsky-uir` E-Graph. It enables "Equality Saturation," where rules are applied repeatedly until no new equivalent forms are discovered or a resource limit is reached.

## Features

- **Rewrite Rules**: A flexible `RewriteRule` trait for defining transformations on `IKun` operators.
- **Equality Saturation**: The `SaturationScheduler` manages the execution of rules, ensuring convergence and preventing infinite expansion.
- **Rule Categories**: Support for categorizing rules (e.g., Algebraic, Architectural, Aggressive) for fine-grained optimization control.
- **Performance Focused**: Efficient rule matching and application designed to work with large E-Graphs.

## Core Concepts

### RewriteRule
A rule defines a pattern to match in the E-Graph and a way to add an equivalent form. For example:
`x + 0 => x`

### SaturationScheduler
Controls the optimization process using:
- **Fuel**: Limits the number of iterations to prevent excessive compilation time.
- **Timeout**: Hard time limit for the saturation process.
- **Rebuilding**: Automatically handles E-Graph rebuilding after rule applications to maintain canonical forms.

## Usage

```rust
use chomsky_rule_engine::{RewriteRegistry, SaturationScheduler, RuleCategory};
use chomsky_uir::EGraph;

let egraph = EGraph::new();
let mut registry = RewriteRegistry::new();

// Register rules...
// registry.register(RuleCategory::Algebraic, Box::new(MyRule));

let scheduler = SaturationScheduler::default();
scheduler.run(&egraph, &registry);
```

