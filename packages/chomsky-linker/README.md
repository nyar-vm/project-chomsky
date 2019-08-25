# chomsky-linker

Linker for E-Graphs, handling cross-module references and fusion in the Chomsky framework.

## Overview

`chomsky-linker` allows for merging multiple independent E-Graphs into a single global graph. It resolves `Import` and `Export` references across modules, enabling whole-program optimization and cross-module fusion.

## Features

- **E-Graph Merging**: Robust logic to copy nodes and e-classes from multiple source graphs into a global `IKunLinker`.
- **Symbol Resolution**: Automatically links `Import` nodes to their corresponding `Export` definitions across modules.
- **Cross-Module Fusion**: By merging graphs, the optimizer can reason about calls between modules as if they were in the same graph, unlocking further rewrite opportunities.
- **Recursive Node Mapping**: Handles complex graph structures and maintains canonical forms during the merge process.

## Core Concepts

### IKunLinker
The main linker implementation. It:
1. Maintains a `global_graph`.
2. Tracks `exports` from all added modules.
3. Performs symbol resolution by unifying `Import` e-classes with their target `Export` e-classes.

## Usage

```rust
use chomsky_linker::IKunLinker;
use chomsky_uir::{EGraph, IKun};

let mut linker = IKunLinker::<()>::new();

let mod1 = EGraph::new();
// ... add exports to mod1 ...
linker.add_module("module_a", &mod1);

let mod2 = EGraph::new();
// ... add imports to mod2 ...
linker.add_module("module_b", &mod2);

linker.link();
```
