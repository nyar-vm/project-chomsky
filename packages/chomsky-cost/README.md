# chomsky-cost

Multi-dimensional cost modeling for the Chomsky optimization framework.

## Overview

In the vast space of equivalent program forms managed by the E-Graph, `chomsky-cost` provides the "fitness function" to evaluate which form is best for a given target. It models multiple metrics like latency, throughput, code size, and energy consumption.

## Features

- **Multi-dimensional Cost**: The `Cost` struct tracks latency, throughput, size, and energy.
- **Cost Models**: The `CostModel` trait allows defining how each `IKun` operator impacts the total cost on different hardware.
- **Customizable Weighting**: Support for scalarizing multi-dimensional costs into a single score using weighted sums, allowing users to prioritize different optimization targets (e.g., optimize for size vs. optimize for speed).
- **Default Models**: Includes a `DefaultCostModel` and target-specific examples like `JsCostModel`.

## Cost Metrics

- **Latency**: Estimated execution time of a single operation.
- **Throughput**: Number of operations that can be processed in parallel or pipelined.
- **Size**: Estimated binary or source code size of the operation.
- **Energy**: Estimated energy consumption.

## Usage

```rust
use chomsky_cost::{Cost, CostModel, DefaultCostModel};
use chomsky_uir::IKun;

let model = DefaultCostModel;
let node = IKun::Constant(42);
let cost = model.cost(&node);

println!("Score: {}", cost.score());
```

