#![warn(missing_docs)]

use chomsky_uir::constraint::Ownership;
use chomsky_uir::egraph::{Analysis, EGraph};
use chomsky_uir::intent::IKun;
use std::collections::HashMap;

/// OwnershipAnalysis: 推导每个节点的资源所有权状态
#[derive(Default, Clone)]
pub struct OwnershipAnalysis;

impl Analysis<IKun> for OwnershipAnalysis {
    type Data = Ownership;

    fn make(egraph: &EGraph<IKun, Self>, enode: &IKun) -> Self::Data {
        match enode {
            // 显式约束节点传播所有权
            IKun::OwnershipConstraint(o) => *o,
            IKun::WithConstraint(_, c) => egraph.get_class(*c).data,

            // 资源创建点 (Source)
            IKun::Map(_, _) | IKun::Reduce(_, _, _) | IKun::Closure(_, _) => Ownership::Owned,

            // 资源消费点 (Sink)
            IKun::ResourceDrop(_) => Ownership::Owned,

            // 默认传播：如果子节点是 Linear/Owned，通常父节点继承或转换为 Owned
            // 这里简化处理：默认为 Shared，除非显式指定
            _ => Ownership::Shared,
        }
    }

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> bool {
        // Lattice: Borrowed < Shared < Owned < Linear
        if to == &from {
            return false;
        }
        // 简单合并逻辑：取“更强”的所有权
        let mut changed = false;
        if from == Ownership::Linear && *to != Ownership::Linear {
            *to = Ownership::Linear;
            changed = true;
        } else if from == Ownership::Owned && *to != Ownership::Linear && *to != Ownership::Owned {
            *to = Ownership::Owned;
            changed = true;
        }
        changed
    }
}

pub struct LifetimePass;

impl LifetimePass {
    /// 运行生命周期分析并插入显式资源管理指令
    /// 注意：由于 E-Graph 是纯函数式的，这里的 "Pass" 实际上是生成一个新的、包含 RC 指令的图，
    /// 或者在提取阶段（Extraction）应用。
    /// 为了演示，我们提供一个简单的分析入口。
    pub fn analyze(egraph: &EGraph<IKun, OwnershipAnalysis>) -> HashMap<u32, Ownership> {
        let mut results = HashMap::new();
        for r in egraph.classes.iter() {
            let id = r.key();
            let class = r.value();
            results.insert(*id as u32, class.data);
        }
        results
    }
}
