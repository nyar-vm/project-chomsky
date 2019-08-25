# chomsky-glue

Interoperability and FFI glue layers for the Chomsky framework.

## Overview

`chomsky-glue` facilitates seamless interaction between Project Chomsky and other programming languages. it provides tools for registering external functions, validating cross-language calls, and generating the necessary FFI (Foreign Function Interface) boilerplate.

## Features

- **Cross-Language Registry**: A central database for external function metadata, including language, name, and semantic constraints.
- **Constraint-Aware Linting**: The `CrossLangLinter` validates whether cross-language calls satisfy the required effects, ownership, and type constraints within the E-Graph.
- **Extensible Glue Generation**: A plugin-based system for generating adapter code for different languages (e.g., Python, TypeScript).
- **FFI Boilerplate**: Automatically generates `extern "C"` signatures and adapter logic.

## Core Concepts

### CrossLangRegistry
Stores `ExternalFuncMetadata`, which defines the requirements (e.g., must be pure, needs specific ownership) for calling an external function.

### GlueGenerator
Uses `GlueProvider`s to produce language-specific adapter code. For example, it can generate the C-compatible bridge for a Python function.

## Usage

```rust
use chomsky_glue::{CrossLangRegistry, ExternalFuncMetadata, GlueGenerator};

let mut registry = CrossLangRegistry::new();
registry.register(ExternalFuncMetadata {
    lang: "python".to_string(),
    name: "compute_expensive".to_string(),
    required_constraints: Default::default(),
    provided_constraints: Default::default(),
});

let generator = GlueGenerator::new();
let adapter = generator.generate_adapter("python", "compute_expensive");
```
