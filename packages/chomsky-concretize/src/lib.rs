#![warn(missing_docs)]

use chomsky_rule_engine::RewriteRule;
use chomsky_uir::egraph::{Analysis, EGraph};
use chomsky_uir::{IKun, Language};
use serde::{Deserialize, Serialize};

/// Layout: 内存布局类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Layout {
    #[default]
    Unknown,
    SoA,
    AoS,
}

/// ConcretizationData: 具体化层分析数据
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ConcretizationData {
    pub layout: Layout,
    pub tiling_factor: Option<usize>,
    pub unroll_factor: Option<usize>,
    pub vector_width: Option<usize>,
}

impl ConcretizationData {
    pub fn merge(&mut self, other: &Self) -> bool {
        let mut changed = false;
        if self.layout == Layout::Unknown && other.layout != Layout::Unknown {
            self.layout = other.layout;
            changed = true;
        }
        if self.tiling_factor.is_none() && other.tiling_factor.is_some() {
            self.tiling_factor = other.tiling_factor;
            changed = true;
        }
        if self.unroll_factor.is_none() && other.unroll_factor.is_some() {
            self.unroll_factor = other.unroll_factor;
            changed = true;
        }
        if self.vector_width.is_none() && other.vector_width.is_some() {
            self.vector_width = other.vector_width;
            changed = true;
        }
        changed
    }
}

#[derive(Default)]
pub struct ConcretizationAnalysis;

impl Analysis<IKun> for ConcretizationAnalysis {
    type Data = ConcretizationData;

    fn make(egraph: &EGraph<IKun, Self>, enode: &IKun) -> Self::Data {
        let mut data = ConcretizationData::default();
        match enode {
            IKun::SoALayout(child) | IKun::SoAMap(child, _) => {
                data = egraph.get_class(*child).data.clone();
                data.layout = Layout::SoA;
            }
            IKun::AoSLayout(child) => {
                data = egraph.get_class(*child).data.clone();
                data.layout = Layout::AoS;
            }
            IKun::Tiled(factor, child) | IKun::TiledMap(factor, child, _) => {
                data = egraph.get_class(*child).data.clone();
                data.tiling_factor = Some(*factor);
            }
            IKun::Unrolled(factor, child) | IKun::UnrolledMap(factor, child, _) => {
                data = egraph.get_class(*child).data.clone();
                data.unroll_factor = Some(*factor);
            }
            IKun::Vectorized(width, child) | IKun::VectorizedMap(width, child, _) => {
                data = egraph.get_class(*child).data.clone();
                data.vector_width = Some(*width);
            }
            _ => {
                // 默认从子节点传播
                for &child in enode.children().iter() {
                    let child_data = &egraph.get_class(child).data;
                    data.merge(child_data);
                }
            }
        }
        data
    }

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> bool {
        to.merge(&from)
    }
}

/// LayoutOptimizer: 处理意图到指令之间的内存布局决策 (SoA vs AoS)
pub struct LayoutOptimizer;

impl<A: Analysis<IKun>> RewriteRule<A> for LayoutOptimizer {
    fn name(&self) -> &str {
        "layout-optimization"
    }

    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                match node {
                    IKun::Symbol(_) => {
                        matches.push((id, None));
                    }
                    IKun::Map(f, x) | IKun::GpuMap(f, x) | IKun::CpuMap(f, x) => {
                        matches.push((id, Some((*f, *x))));
                    }
                    _ => {}
                }
            }
        }

        for (id, map_parts) in matches {
            if let Some((f, x)) = map_parts {
                // 对于 Map 类型，使用 SoAMap 避免循环
                let soa_id = egraph.add(IKun::SoAMap(f, x));
                egraph.union(id, soa_id);
            } else {
                // 对于 Symbol，使用包装节点（虽然会产生循环，但目前没有更好的办法，后续靠 Cost 剪枝）
                let soa_id = egraph.add(IKun::SoALayout(id));
                let aos_id = egraph.add(IKun::AoSLayout(id));
                egraph.union(id, soa_id);
                egraph.union(id, aos_id);
            }
        }
    }
}

/// SchedulePlanner: 处理循环调度策略决策 (Tiling, Vectorization, Unrolling)
pub struct SchedulePlanner;

impl<A: Analysis<IKun>> RewriteRule<A> for SchedulePlanner {
    fn name(&self) -> &str {
        "schedule-planning"
    }

    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                match node {
                    IKun::Map(f, x) => {
                        matches.push((id, *f, *x, false));
                    }
                    IKun::GpuMap(f, x) => {
                        matches.push((id, *f, *x, true));
                    }
                    IKun::CpuMap(f, x) => {
                        matches.push((id, *f, *x, false));
                    }
                    _ => {}
                }
            }
        }

        for (id, f, x, is_gpu) in matches {
            if is_gpu {
                // GPU 优先尝试 Tiling
                let tiled_id = egraph.add(IKun::TiledMap(32, f, x));
                egraph.union(id, tiled_id);
            } else {
                // CPU 尝试 Vectorization 和 Unrolling
                let vec_id = egraph.add(IKun::VectorizedMap(8, f, x));
                let unroll_id = egraph.add(IKun::UnrolledMap(4, f, x));
                egraph.union(id, vec_id);
                egraph.union(id, unroll_id);
            }
        }
    }
}
