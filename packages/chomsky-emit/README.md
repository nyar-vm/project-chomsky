# chomsky-emit

Code emission layer for the Chomsky framework, bridging UIR to target assemblers.

## Overview

`chomsky-emit` is the final stage of the compilation pipeline. It takes the optimized and extracted `IKunTree` and translates it into target-specific code. Its primary target is Gaia IR, which provides a high-level assembly format that can be further compiled to various backends like JVM, WASI, or x86.

## Features

- **Unified Emission**: Provides a single `GaiaEmitter` to handle translation to Gaia IR.
- **Backend Integration**: Pluggable backend system, supporting `JvmBackend`, `WasiBackend`, and `X86Backend`.
- **JIT Support**: Specialized emission modes for Just-In-Time compilation, including prologue generation for NyarVM calling conventions.
- **Structure Reconstruction**: Reconstructs high-level module elements like classes, functions, and constants from the `IKunTree`.

## Core Concepts

### GaiaEmitter
The central component that:
1. Traverses the `IKunTree`.
2. Generates equivalent Gaia IR (`GaiaModule`).
3. Invokes specific Gaia backends to produce binary or assembly artifacts.

## Usage

```rust
use chomsky_emit::{GaiaEmitter, JvmBackend};
use chomsky_extract::IKunTree;
use std::sync::Arc;

let emitter = GaiaEmitter::new("jvm")
    .with_backend("jvm", Arc::new(JvmBackend::new()));

let tree = IKunTree::Symbol("my_func".to_string());
let artifact = emitter.emit(&tree).expect("Emission failed");
```
