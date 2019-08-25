# chomsky-rules

Optimization rules collection for the Chomsky framework.

## Overview

`chomsky-rules` is a library of predefined rewrite rules for the Chomsky framework. It contains common algebraic simplifications, constant folding logic, and architectural transformations that can be plugged into the `chomsky-rule-engine`.

## Features

- **Constant Folding**: Pre-calculates constant expressions (e.g., `1 + 2 => 3`).
- **Algebraic Simplification**: Implements common identities (e.g., `x + 0 => x`, `x * 1 => x`, `x - x => 0`).
- **Boolean Logic**: Simplifies boolean expressions and conditional branches.
- **Architectural Rules**: Provides rules for mapping generic operations to optimized hardware-specific forms.

## Core Rules

- **ConstantFolding**: Handles basic arithmetic operations on constant values.
- **AlgebraicSimplification**: Applies mathematical identities to reduce program complexity.

## Usage

```rust
use chomsky_rules::{ConstantFolding, AlgebraicSimplification};
use chomsky_rule_engine::{RewriteRegistry, RuleCategory};
use chomsky_uir::EGraph;

let mut registry = RewriteRegistry::new();
registry.register(RuleCategory::Algebraic, Box::new(ConstantFolding));
registry.register(RuleCategory::Algebraic, Box::new(AlgebraicSimplification));
```
