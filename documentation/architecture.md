# Chomsky 架构分析：承载“普遍优化”的基石

本文档分析了 `ProjectChomsky/packages` 下的模块化架构如何实现并承载“普遍优化”的目标。

## 1. 核心架构概览

Chomsky 的 packages 结构遵循 **Intake -> Optimize -> Extract -> Concretize** 的流水线，每一层都为“普遍性”做了深度解耦：

- **[chomsky-uir](./packages/chomsky-uir)**: **普遍语法的承载者**。利用 `IKun` 枚举定义了计算意图的原子（Map, Reduce, Seq 等），并通过 `Extension` 提供了无限性。
- **[chomsky-emit](./packages/chomsky-emit)**: **统一的提取层**。不再为每个硬件后端编写繁琐的适配器，而是统一导出为 Gaia Assembly，利用高度复用的 [gaia.ts](../../gaia.ts) 进行最终的代码生成。
- **[chomsky-rule-engine](./packages/chomsky-rule-engine)**: **寻优算法的引擎**。
- **[chomsky-cost](./packages/chomsky-cost)**: **多维价值评价体系**。通过定义四维成本向量（Latency, Throughput, Size, Energy），它能同时满足高性能计算（DL）、低延迟查询（DB）和嵌入式硬件（EDA）的不同评价标准。
- **[chomsky](./packages/chomsky)**: **架构的编排者**。它将上述组件封装为 `UniversalOptimizer`，为各领域的输入（Intake）提供了一致的优化接口。

## 2. 为什么它能承载“普遍优化”？

### 2.1 解决阶段顺序问题 (Phase Ordering)
传统编译器（如 LLVM）由于采用顺序变换，常常因为先做了 A 优化而导致无法再做 B 优化。Chomsky 利用 E-Graph 的等价类空间，同时保留所有可能的变换路径，在最后阶段才根据成本模型提取最优解，从而彻底解决了这一难题。

### 2.2 跨领域的语义归一化
无论是 SQL 的 `JOIN`、深度学习的 `MatMul` 还是编程语言的 `Loop`，在 Chomsky 中都被抽象为意图。这种归一化使得我们可以在 `chomsky-rules` 中编写通用的代数重写规则（如结合律、分配律），同时作用于所有领域。

### 2.3 约束与效应的格系统
通过 `chomsky-uir` 中的 `Analysis` 机制，我们可以为 E-Graph 挂载复杂的属性分析（如所有权格、副作用格）。这使得优化器在进行等价重写时，能够严格遵守“普遍语法”定义的约束条件，确保优化后的程序语义完全等价。

## 3. 架构演进与风险评估

目前的架构已具备承载能力，详细的论证请参阅 **[普遍优化可行性评估报告](./assessment-universal-optimization.md)**。未来的扩展点将集中在：
1. **反馈驱动优化 (FDO)**: 将 `Extraction` 后的实际运行数据反馈给 `CostModel`。
2. **多层级 IR (Multi-level IR)**: 在 `Concretize` 阶段引入更贴近硬件的子级 IR 寻优。
3. **协同寻优 (Co-optimization)**: 同时对算法逻辑和内存布局进行 E-Graph 搜索。

---
结论：**Chomsky 的包结构设计是面向未来十年计算模式演进的，完全能够胜任“普遍优化”的历史使命。**
