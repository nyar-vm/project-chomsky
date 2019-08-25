# chomsky-extract

Optimal program extraction from E-Graphs for the Chomsky framework.

## Overview

After `chomsky-rule-engine` has expanded the E-Graph with equivalent forms and `chomsky-cost` has assigned costs, `chomsky-extract` is responsible for selecting the "best" representative program. It finds the lowest-cost tree from the complex cyclic E-Graph structure.

## Features

- **Optimal Extraction**: Uses a greedy but effective extraction algorithm to find the lowest-cost `IKunTree` for a given E-Class.
- **Backend Trait**: Defines the standard interface for all backends that consume extracted trees and produce artifacts (source, binary, or assembly).
- **Artifact Management**: Supports multiple output formats via `BackendArtifact`, including binary blobs, source strings, and collections of files.
- **Recursive Reconstruction**: Efficiently reconstructs the program tree while respecting the cost-based decisions.

## Core Concepts

### IKunExtractor
The main engine for extraction. It:
1. Calculates the best cost for every E-Class in the graph based on a `CostModel`.
2. Recursively selects the best nodes to form an `IKunTree`.

### Backend
The interface for code generation. A backend provides:
- A `name`.
- A `generate` method to transform an `IKunTree` into a `BackendArtifact`.
- A target-specific `CostModel`.

## Usage

```rust
use chomsky_extract::IKunExtractor;
use chomsky_cost::DefaultCostModel;
use chomsky_uir::EGraph;

let egraph = EGraph::new();
// ... fill egraph ...

let extractor = IKunExtractor::new(&egraph, DefaultCostModel);
let best_tree = extractor.extract(root_id);
```

