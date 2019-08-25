#![warn(missing_docs)]

use chomsky_rule_engine::{RewriteRegistry, RewriteRule, RuleCategory};
use chomsky_uir::egraph::{Analysis, EGraph};
use chomsky_uir::{IKun, Id};

pub struct ConstantFolding;

impl<A: Analysis<IKun>> RewriteRule<A> for ConstantFolding {
    fn name(&self) -> &str {
        "constant-folding"
    }

    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::Extension(op, args) = node {
                    if args.len() == 2 {
                        let arg1_root = egraph.union_find.find(args[0]);
                        let arg2_root = egraph.union_find.find(args[1]);

                        let arg1_const = get_const(egraph, arg1_root);
                        let arg2_const = get_const(egraph, arg2_root);

                        if let (Some(v1), Some(v2)) = (arg1_const, arg2_const) {
                            match op.as_str() {
                                "add" => matches.push((id, v1 + v2)),
                                "sub" => matches.push((id, v1 - v2)),
                                "mul" => matches.push((id, v1 * v2)),
                                "div" if v2 != 0 => matches.push((id, v1 / v2)),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        for (id, result) in matches {
            let const_id = egraph.add(IKun::Constant(result));
            egraph.union(id, const_id);
        }
    }
}

fn get_bool<A: Analysis<IKun>>(egraph: &EGraph<IKun, A>, id: Id) -> Option<bool> {
    let root = egraph.union_find.find(id);
    let eclass = egraph.classes.get(&root)?;
    for node in &eclass.nodes {
        if let IKun::BooleanConstant(v) = node {
            return Some(*v);
        }
    }
    None
}

pub struct AlgebraicSimplification;

impl<A: Analysis<IKun>> RewriteRule<A> for AlgebraicSimplification {
    fn name(&self) -> &str {
        "algebraic-simplification"
    }

    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let id = *entry.key();
            let eclass = entry.value();
            for node in &eclass.nodes {
                if let IKun::Extension(op, args) = node {
                    if args.len() == 2 {
                        let arg1_root = egraph.union_find.find(args[0]);
                        let arg2_root = egraph.union_find.find(args[1]);

                        match op.as_str() {
                            "add" => {
                                // x + 0 = x
                                if is_const(egraph, arg2_root, 0) {
                                    matches.push((id, args[0]));
                                }
                                // 0 + x = x
                                else if is_const(egraph, arg1_root, 0) {
                                    matches.push((id, args[1]));
                                }
                            }
                            "sub" => {
                                // x - 0 = x
                                if is_const(egraph, arg2_root, 0) {
                                    matches.push((id, args[0]));
                                }
                                // x - x = 0
                                else if arg1_root == arg2_root {
                                    let zero_id = egraph.add(IKun::Constant(0));
                                    matches.push((id, zero_id));
                                }
                            }
                            "mul" => {
                                // x * 1 = x
                                if is_const(egraph, arg2_root, 1) {
                                    matches.push((id, args[0]));
                                }
                                // 1 * x = x
                                else if is_const(egraph, arg1_root, 1) {
                                    matches.push((id, args[1]));
                                }
                                // x * 0 = 0
                                else if is_const(egraph, arg2_root, 0) {
                                    matches.push((id, arg2_root));
                                }
                                // 0 * x = 0
                                else if is_const(egraph, arg1_root, 0) {
                                    matches.push((id, arg1_root));
                                }
                            }
                            "div" => {
                                // x / 1 = x
                                if is_const(egraph, arg2_root, 1) {
                                    matches.push((id, args[0]));
                                }
                                // x / x = 1 (if x != 0)
                                else if arg1_root == arg2_root {
                                    // In a real compiler we'd check if x can be 0.
                                    // For simplicity in this demo, we assume safety or handle it via analysis.
                                    let one_id = egraph.add(IKun::Constant(1));
                                    matches.push((id, one_id));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        for (id, target) in matches {
            egraph.union(id, target);
        }
    }
}

pub struct TrapSimplification;

impl<A: Analysis<IKun>> RewriteRule<A> for TrapSimplification {
    fn name(&self) -> &str {
        "trap-simplification"
    }

    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::Trap(inner) = node {
                    let inner_root = egraph.union_find.find(*inner);

                    // Trap(Trap(x)) -> Trap(x)
                    if let Some(inner_eclass) = egraph.classes.get(&inner_root) {
                        for inner_node in &inner_eclass.nodes {
                            if let IKun::Trap(_) = inner_node {
                                matches.push((id, *inner));
                            }
                        }
                    }
                }
            }
        }

        for (id, target) in matches {
            egraph.union(id, target);
        }
    }
}

pub struct UniversalSemanticOptimization;

impl<A: Analysis<IKun>> RewriteRule<A> for UniversalSemanticOptimization {
    fn name(&self) -> &str {
        "universal-semantic-optimization"
    }

    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                match node {
                    // 1. Redundant Context elimination: WithContext(ctx, WithContext(ctx, x)) -> WithContext(ctx, x)
                    IKun::WithContext(ctx_id, inner_id) => {
                        let ctx_root = egraph.union_find.find(*ctx_id);
                        let inner_root = egraph.union_find.find(*inner_id);

                        if let Some(inner_class) = egraph.classes.get(&inner_root) {
                            for inner_node in &inner_class.nodes {
                                if let IKun::WithContext(nested_ctx_id, _nested_inner_id) =
                                    inner_node
                                {
                                    if egraph.union_find.find(*nested_ctx_id) == ctx_root {
                                        // Found redundant context
                                        matches.push((id, *inner_id));
                                    }
                                }
                            }
                        }
                    }

                    // 2. Comptime evaluation: WithContext(ComptimeContext, x) -> x (once evaluated)
                    // In a real system, this would trigger the actual comptime engine.
                    // Here we just model the fact that it simplifies if evaluated.

                    // 3. Defer normalization: WithContext(DeferContext, List(a, b, Defer(c)))
                    // This is more complex and depends on how we lower defer.
                    _ => {}
                }
            }
        }

        for (id, result) in matches {
            egraph.union(id, result);
        }
    }
}

fn get_const<A: Analysis<IKun>>(egraph: &EGraph<IKun, A>, id: Id) -> Option<i64> {
    let root = egraph.union_find.find(id);
    egraph.classes.get(&root).and_then(|c| {
        c.nodes.iter().find_map(|n| {
            if let IKun::Constant(v) = n {
                Some(*v)
            } else {
                None
            }
        })
    })
}

fn is_const<A: Analysis<IKun>>(egraph: &EGraph<IKun, A>, id: Id, val: i64) -> bool {
    get_const(egraph, id) == Some(val)
}

pub struct StrengthReduction;

impl<A: Analysis<IKun>> RewriteRule<A> for StrengthReduction {
    fn name(&self) -> &str {
        "strength-reduction"
    }
    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::Extension(op, args) = node {
                    if args.len() == 2 {
                        if let Some(val) = get_const(egraph, args[1]) {
                            match op.as_str() {
                                "mul" if val > 0 && (val & (val - 1)) == 0 => {
                                    let n = (val as f64).log2() as i64;
                                    matches.push((id, "shl", args[0], n));
                                }
                                "div" if val > 0 && (val & (val - 1)) == 0 => {
                                    let n = (val as f64).log2() as i64;
                                    matches.push((id, "shr", args[0], n));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        for (id, op, arg, n) in matches {
            let n_id = egraph.add(IKun::Constant(n));
            let new_id = egraph.add(IKun::Extension(op.to_string(), vec![arg, n_id]));
            egraph.union(id, new_id);
        }
    }
}

pub struct Peephole;

impl<A: Analysis<IKun>> RewriteRule<A> for Peephole {
    fn name(&self) -> &str {
        "peephole"
    }
    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::Extension(op, args) = node {
                    if args.len() == 2 {
                        let arg1_root = egraph.union_find.find(args[0]);
                        let arg2_root = egraph.union_find.find(args[1]);
                        match op.as_str() {
                            "add" if arg1_root == arg2_root => {
                                matches.push((id, "mul", args[0], 2));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        for (id, op, arg, val) in matches {
            let val_id = egraph.add(IKun::Constant(val));
            let new_id = egraph.add(IKun::Extension(op.to_string(), vec![arg, val_id]));
            egraph.union(id, new_id);
        }
    }
}

pub struct MapFusion;

impl<A: Analysis<IKun>> RewriteRule<A> for MapFusion {
    fn name(&self) -> &str {
        "map-fusion"
    }

    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::Map(f, inner_id) = node {
                    let inner_root = egraph.union_find.find(*inner_id);
                    if let Some(inner_class) = egraph.classes.get(&inner_root) {
                        for inner_node in &inner_class.nodes {
                            if let IKun::Map(g, x) = inner_node {
                                matches.push((id, *f, *g, *x));
                            }
                        }
                    }
                }
            }
        }

        for (id, f, g, x) in matches {
            let seq_id = egraph.add(IKun::Compose(f, g));
            let new_map_id = egraph.add(IKun::Map(seq_id, x));
            egraph.union(id, new_map_id);
        }
    }
}

pub struct FilterFusion;

impl<A: Analysis<IKun>> RewriteRule<A> for FilterFusion {
    fn name(&self) -> &str {
        "filter-fusion"
    }

    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::Filter(p1, inner_id) = node {
                    let inner_root = egraph.union_find.find(*inner_id);
                    if let Some(inner_class) = egraph.classes.get(&inner_root) {
                        for inner_node in &inner_class.nodes {
                            if let IKun::Filter(p2, x) = inner_node {
                                matches.push((id, *p1, *p2, *x));
                            }
                        }
                    }
                }
            }
        }

        for (id, p1, p2, x) in matches {
            let combined_p = egraph.add(IKun::Extension("and_predicate".to_string(), vec![p2, p1]));
            let new_filter_id = egraph.add(IKun::Filter(combined_p, x));
            egraph.union(id, new_filter_id);
        }
    }
}

pub struct FilterMapFusion;

impl<A: Analysis<IKun>> RewriteRule<A> for FilterMapFusion {
    fn name(&self) -> &str {
        "filter-map-fusion"
    }

    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::Map(f, inner_id) = node {
                    let inner_root = egraph.union_find.find(*inner_id);
                    if let Some(inner_class) = egraph.classes.get(&inner_root) {
                        for inner_node in &inner_class.nodes {
                            if let IKun::Filter(p, x) = inner_node {
                                matches.push((id, *f, *p, *x));
                            }
                        }
                    }
                }
            }
        }

        for (id, f, p, x) in matches {
            let fm_node = egraph.add(IKun::Extension("filter_map".to_string(), vec![f, p, x]));
            egraph.union(id, fm_node);
        }
    }
}

pub struct MapFilterFusion;

impl<A: Analysis<IKun>> RewriteRule<A> for MapFilterFusion {
    fn name(&self) -> &str {
        "map-filter-fusion"
    }

    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::Filter(p, inner_id) = node {
                    let inner_root = egraph.union_find.find(*inner_id);
                    if let Some(inner_class) = egraph.classes.get(&inner_root) {
                        for inner_node in &inner_class.nodes {
                            if let IKun::Map(f, x) = inner_node {
                                matches.push((id, *p, *f, *x));
                            }
                        }
                    }
                }
            }
        }

        for (id, p, f, x) in matches {
            let mf_node = egraph.add(IKun::Extension("map_filter".to_string(), vec![p, f, x]));
            egraph.union(id, mf_node);
        }
    }
}

pub struct MapReduceFusion;

impl<A: Analysis<IKun>> RewriteRule<A> for MapReduceFusion {
    fn name(&self) -> &str {
        "map-reduce-fusion"
    }

    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::Reduce(g, init, inner_id) = node {
                    let inner_root = egraph.union_find.find(*inner_id);
                    if let Some(inner_class) = egraph.classes.get(&inner_root) {
                        for inner_node in &inner_class.nodes {
                            if let IKun::Map(f, x) = inner_node {
                                matches.push((id, *f, *g, *init, *x));
                            }
                        }
                    }
                }
            }
        }

        for (id, f, g, init, x) in matches {
            let fused_node = egraph.add(IKun::Extension(
                "loop_map_reduce".to_string(),
                vec![f, g, init, x],
            ));
            egraph.union(id, fused_node);
        }
    }
}

pub struct FilterReduceFusion;

impl<A: Analysis<IKun>> RewriteRule<A> for FilterReduceFusion {
    fn name(&self) -> &str {
        "filter-reduce-fusion"
    }

    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::Reduce(f, init, inner_id) = node {
                    let inner_root = egraph.union_find.find(*inner_id);
                    if let Some(inner_class) = egraph.classes.get(&inner_root) {
                        for inner_node in &inner_class.nodes {
                            if let IKun::Filter(p, x) = inner_node {
                                matches.push((id, *p, *f, *init, *x));
                            }
                        }
                    }
                }
            }
        }

        for (id, p, f, init, x) in matches {
            let fused_node = egraph.add(IKun::Extension(
                "loop_filter_reduce".to_string(),
                vec![p, f, init, x],
            ));
            egraph.union(id, fused_node);
        }
    }
}

pub struct LayoutTransformation;

impl<A: Analysis<IKun>> RewriteRule<A> for LayoutTransformation {
    fn name(&self) -> &str {
        "layout-transformation"
    }
    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let eclass = entry.value();
            for node in &eclass.nodes {
                if let IKun::WithContext(ctx_id, body_id) = node {
                    let is_spatial = egraph.classes.get(ctx_id).map_or(false, |c| {
                        c.nodes
                            .iter()
                            .any(|n| matches!(n, IKun::SpatialContext | IKun::GpuContext))
                    });

                    if is_spatial {
                        if let Some(body_class) = egraph.classes.get(body_id) {
                            for body_node in &body_class.nodes {
                                if let IKun::Map(f, x) = body_node {
                                    matches.push((*body_id, *f, *x));
                                }
                            }
                        }
                    }
                }
            }
        }
        for (id, f, x) in matches {
            let soa_map = egraph.add(IKun::SoAMap(f, x));
            egraph.union(id, soa_map);
        }
    }
}

pub struct LoopTiling;

impl<A: Analysis<IKun>> RewriteRule<A> for LoopTiling {
    fn name(&self) -> &str {
        "loop-tiling"
    }
    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let eclass = entry.value();
            for node in &eclass.nodes {
                if let IKun::Map(f, x) = node {
                    matches.push((*entry.key(), *f, *x));
                }
            }
        }
        for (id, f, x) in matches {
            let tiled_map = egraph.add(IKun::TiledMap(32, f, x));
            egraph.union(id, tiled_map);
        }
    }
}

pub struct AutoVectorization;

impl<A: Analysis<IKun>> RewriteRule<A> for AutoVectorization {
    fn name(&self) -> &str {
        "auto-vectorization"
    }
    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let eclass = entry.value();
            for node in &eclass.nodes {
                if let IKun::Map(f, x) = node {
                    matches.push((*entry.key(), *f, *x));
                }
            }
        }
        for (id, f, x) in matches {
            let vectorized_map = egraph.add(IKun::VectorizedMap(8, f, x));
            egraph.union(id, vectorized_map);
        }
    }
}

pub struct GpuSpecialization;

impl<A: Analysis<IKun>> RewriteRule<A> for GpuSpecialization {
    fn name(&self) -> &str {
        "gpu-specialization"
    }

    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::WithContext(ctx_id, body_id) = node {
                    let is_gpu = egraph.classes.get(ctx_id).map_or(false, |c| {
                        c.nodes.iter().any(|n| matches!(n, IKun::GpuContext))
                    });

                    if is_gpu {
                        if let Some(body_class) = egraph.classes.get(body_id) {
                            for body_node in &body_class.nodes {
                                if let IKun::Map(f, x) = body_node {
                                    matches.push((id, *f, *x));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct CpuSpecialization;

impl<A: Analysis<IKun>> RewriteRule<A> for CpuSpecialization {
    fn name(&self) -> &str {
        "cpu-specialization"
    }

    fn apply(&self, _egraph: &EGraph<IKun, A>) {}
}

pub struct MapToLoop;

impl<A: Analysis<IKun>> RewriteRule<A> for MapToLoop {
    fn name(&self) -> &str {
        "map-to-loop"
    }

    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::Map(f, x) = node {
                    matches.push((id, *f, *x));
                }
            }
        }

        for (id, f, x) in matches {
            let loop_node = egraph.add(IKun::Extension("loop_map".to_string(), vec![f, x]));
            egraph.union(id, loop_node);
        }

        let mut filter_matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::Filter(p, x) = node {
                    filter_matches.push((id, *p, *x));
                }
            }
        }

        for (id, p, x) in filter_matches {
            let loop_node = egraph.add(IKun::Extension("loop_filter".to_string(), vec![p, x]));
            egraph.union(id, loop_node);
        }

        let mut reduce_matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::Reduce(f, init, x) = node {
                    reduce_matches.push((id, *f, *init, *x));
                }
            }
        }

        for (id, f, init, x) in reduce_matches {
            let loop_node =
                egraph.add(IKun::Extension("loop_reduce".to_string(), vec![f, init, x]));
            egraph.union(id, loop_node);
        }
    }
}

/// 注册所有标准优化规则。这些规则涵盖了通用的代数优化、
/// 函数式融合以及编程语言（PL）相关的通用语义优化。
pub fn register_standard_rules<A: Analysis<IKun> + 'static>(registry: &mut RewriteRegistry<A>) {
    // 1. 代数规则 (Algebraic Rules)
    registry.register(RuleCategory::Algebraic, Box::new(ConstantFolding));
    registry.register(RuleCategory::Algebraic, Box::new(AlgebraicSimplification));
    registry.register(RuleCategory::Algebraic, Box::new(StrengthReduction));
    registry.register(RuleCategory::Algebraic, Box::new(Peephole));
    registry.register(RuleCategory::Algebraic, Box::new(TrapSimplification));
    registry.register(RuleCategory::Algebraic, Box::new(MetaSimplification));
    registry.register(RuleCategory::Algebraic, Box::new(ContextSimplification));
    registry.register(RuleCategory::Algebraic, Box::new(LifeCycleSimplification));
    registry.register(RuleCategory::Algebraic, Box::new(SafeElimination));

    // 2. 架构与融合规则 (Architectural & Fusion Rules)
    registry.register(RuleCategory::Architectural, Box::new(MapFusion));
    registry.register(RuleCategory::Architectural, Box::new(FilterFusion));
    registry.register(RuleCategory::Architectural, Box::new(FilterMapFusion));
    registry.register(RuleCategory::Architectural, Box::new(MapFilterFusion));
    registry.register(RuleCategory::Architectural, Box::new(MapReduceFusion));
    registry.register(RuleCategory::Architectural, Box::new(FilterReduceFusion));
    registry.register(RuleCategory::Architectural, Box::new(LayoutTransformation));
    registry.register(RuleCategory::Architectural, Box::new(LoopTiling));
    registry.register(RuleCategory::Architectural, Box::new(AutoVectorization));

    // 3. 特化规则 (Specialization Rules)
    registry.register(RuleCategory::Architectural, Box::new(GpuSpecialization));
    registry.register(RuleCategory::Architectural, Box::new(CpuSpecialization));

    // 4. 具体化规则 (Concretization Rules)
    registry.register(RuleCategory::Concretization, Box::new(MapToLoop));
}

pub struct MetaSimplification;

impl<A: Analysis<IKun>> RewriteRule<A> for MetaSimplification {
    fn name(&self) -> &str {
        "meta-simplification"
    }

    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::Meta(inner) = node {
                    let inner_root = egraph.union_find.find(*inner);

                    // 1. Meta(Meta(x)) -> Meta(x)
                    if let Some(inner_eclass) = egraph.classes.get(&inner_root) {
                        for inner_node in &inner_eclass.nodes {
                            if let IKun::Meta(_) = inner_node {
                                matches.push((id, *inner));
                            }
                        }
                    }

                    // 2. Meta(Constant(x)) -> Constant(x)
                    if let Some(inner_eclass) = egraph.classes.get(&inner_root) {
                        for inner_node in &inner_eclass.nodes {
                            if let IKun::Constant(_) = inner_node {
                                matches.push((id, *inner));
                            }
                        }
                    }
                }
            }
        }

        for (id, target) in matches {
            egraph.union(id, target);
        }
    }
}

pub struct ContextSimplification;

impl<A: Analysis<IKun>> RewriteRule<A> for ContextSimplification {
    fn name(&self) -> &str {
        "context-simplification"
    }

    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::WithContext(ctx, inner) = node {
                    let inner_root = egraph.union_find.find(*inner);

                    // 1. Redundant Context: WithContext(ctx, WithContext(ctx, x)) -> WithContext(ctx, x)
                    if let Some(inner_eclass) = egraph.classes.get(&inner_root) {
                        for inner_node in &inner_eclass.nodes {
                            if let IKun::WithContext(inner_ctx, _) = inner_node {
                                if ctx == inner_ctx {
                                    matches.push((id, *inner));
                                }
                            }
                        }
                    }
                }
            }
        }

        for (id, target) in matches {
            egraph.union(id, target);
        }
    }
}

pub struct LifeCycleSimplification;

impl<A: Analysis<IKun>> RewriteRule<A> for LifeCycleSimplification {
    fn name(&self) -> &str {
        "lifecycle-simplification"
    }
    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::LifeCycle(setup, cleanup) = node {
                    let setup_root = egraph.union_find.find(*setup);

                    // 1. LifeCycle(LifeCycle(s1, c1), c2) -> LifeCycle(s1, Seq([c1, c2]))
                    if let Some(setup_eclass) = egraph.classes.get(&setup_root) {
                        for setup_node in &setup_eclass.nodes {
                            if let IKun::LifeCycle(s1, c1) = setup_node {
                                matches.push((id, *s1, *c1, *cleanup));
                            }
                        }
                    }

                    // 2. LifeCycle(Constant(x), c) -> Constant(x) if we can prove c is not needed or side-effect free.
                    // For now, let's just do a simple version: LifeCycle(x, EmptySeq) -> x
                    let cleanup_root = egraph.union_find.find(*cleanup);
                    if let Some(cleanup_eclass) = egraph.classes.get(&cleanup_root) {
                        for cleanup_node in &cleanup_eclass.nodes {
                            if let IKun::Seq(items) = cleanup_node {
                                if items.is_empty() {
                                    matches.push((id, *setup, *setup, *setup)); // dummy target to signal simplification
                                }
                            }
                        }
                    }
                }
            }
        }

        for (id, s1, c1, c2) in matches {
            if s1 == c1 && c1 == c2 {
                // Simplification case 2
                egraph.union(id, s1);
            } else {
                // Simplification case 1
                let new_cleanup = egraph.add(IKun::Seq(vec![c1, c2]));
                let new_lifecycle = egraph.add(IKun::LifeCycle(s1, new_cleanup));
                egraph.union(id, new_lifecycle);
            }
        }
    }
}

pub struct SafeElimination;

impl<A: Analysis<IKun>> RewriteRule<A> for SafeElimination {
    fn name(&self) -> &str {
        "safe-elimination"
    }
    fn apply(&self, egraph: &EGraph<IKun, A>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::Trap(inner) = node {
                    let inner_root = egraph.union_find.find(*inner);
                    if let Some(inner_eclass) = egraph.classes.get(&inner_root) {
                        for inner_node in &inner_eclass.nodes {
                            if let IKun::WithContext(ctx_id, body_id) = inner_node {
                                let ctx_root = egraph.union_find.find(*ctx_id);
                                if let Some(ctx_eclass) = egraph.classes.get(&ctx_root) {
                                    if ctx_eclass
                                        .nodes
                                        .iter()
                                        .any(|n| matches!(n, IKun::SafeContext))
                                    {
                                        matches.push((id, *body_id));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        for (id, target) in matches {
            egraph.union(id, target);
        }
    }
}
