# 深度学习领域 (Deep Learning)

在深度学习领域，Chomsky 充当计算图优化器，主要对接 [dxo-rs](../../dxo-rs) 框架。

## 核心用法
1. **算子导入**：利用 `Extension` 机制将 `MatMul`, `Conv2D`, `Softmax` 等高层算子导入 E-Graph。
2. **等价代数重写**：
   - 应用数学等价律（如结合律、分配律）寻找计算量更小的路径。
   - **FlashAttention 提取**：识别特定的 `Softmax` 与 `MatMul` 组合模式，并重写为单一的 `FlashAttention` 扩展节点。
3. **布局寻优**：在 E-Graph 中同时保留 `NCHW` 和 `NHWC` 的路径，由成本模型根据目标硬件的 Cache 特性选择。

## 侧重点
- **数据流宽度**：侧重于张量维度的推导与内存连续性的保持。
- **算子融合**：寻找将多个逐元素 (Elementwise) 算子合并为一个 Kernel 的最优方案，以减少内存读写。
- **异构加速**：重点关注如何将 IKun 节点高效映射到 NPU 或 GPU 的特定加速单元。
