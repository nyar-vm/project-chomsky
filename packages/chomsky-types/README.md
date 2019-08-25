# chomsky-types

Core type definitions and formal descriptions for the Chomsky optimization framework.

## Overview

`chomsky-types` provides the foundational building blocks for Project Chomsky, defining the "Universal Grammar" that allows the framework to represent diverse computational intents and their execution contexts.

## Key Components

### Intent Atoms
Represent the core computational patterns (Universal Grammar) of the framework:
- **Map**: Data transformation and filtering.
- **Reduce**: Accumulation and aggregation.
- **StateUpdate**: Side effects and state transitions.
- **Branch**: Conditional control flow.
- **Loop**: Iteration and recursion.
- **LifeCycle**: Resource management and cleanup (defer, etc.).
- **Meta**: Compile-time metaprogramming.
- **Trap**: Non-local control flow (exceptions, panics).

### Contexts
Define the environment and constraints under which intents are executed:
- **Linear/RefCounting**: Memory management models.
- **Async**: Asynchronous execution and coroutines.
- **GPU/SIMD**: Hardware-specific acceleration.
- **Safe**: Safety constraints (e.g., non-null).
- **Comptime**: Static evaluation environment.

### Source Management
- **SourceManager**: Global registry for source files.
- **Span & Loc**: Precise location tracking within source files, supporting multi-file projects.
- **LineMap**: Efficient mapping between byte offsets and line/column numbers.

## Usage

This crate is a low-level dependency used by almost all other `chomsky-*` crates. It contains no heavy logic, only data structures and basic utilities.

```rust
use chomsky_types::{Intent, Context, IntentNode};

let node = IntentNode::new(Intent::Map)
    .with_context(Context::GPU);
```

