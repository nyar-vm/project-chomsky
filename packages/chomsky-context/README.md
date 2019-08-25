# chomsky-context

Context and environment management for the Chomsky framework.

## Overview

`chomsky-context` handles the injection and management of execution contexts within the `chomsky-uir` E-Graph. It allows the compiler to explicitly reason about where and how a program fragment will run (e.g., on a GPU, asynchronously, or within a specific memory model).

## Features

- **Context Injection**: Utilities to wrap program fragments with specific contexts (`WithContext`).
- **Standard Contexts**: First-class support for common contexts:
  - **GPU**: For data-parallel kernels.
  - **CPU**: For general-purpose execution.
  - **Async**: For non-blocking/concurrent tasks.
  - **Spatial**: For hardware-aware layout or placement.
- **E-Graph Aware**: Designed to work directly with E-Graph `Id`s, facilitating seamless integration into the optimization pipeline.

## Core Concepts

### ContextInjector
A utility class providing static methods to inject context nodes into an E-Graph. It automatically adds the necessary `Context` node and wraps the target `Id` with a `WithContext` operator.

## Usage

```rust
use chomsky_context::ContextInjector;
use chomsky_uir::EGraph;

let mut egraph = EGraph::new();
let my_code_id = egraph.add(/* ... */);

// Explicitly mark this fragment as running on GPU
let gpu_code_id = ContextInjector::inject_gpu(&egraph, my_code_id);
```

