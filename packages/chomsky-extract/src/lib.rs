#![warn(missing_docs)]

use chomsky_cost::{Cost, CostModel};
use chomsky_uir::egraph::{Analysis, EGraph, HasDebugInfo, Id, Language};
pub use chomsky_uir::{IKun, IKunTree};
use std::collections::HashMap;

use chomsky_types::ChomskyResult;
use chomsky_uir::intent::CrossLanguageCall;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendArtifact {
    Source(String),
    Binary(Vec<u8>),
    Assembly(String),
    Collection(HashMap<String, Vec<u8>>),
}

/// The standard trait for all backends.
/// Backends consume an extracted IKunTree and produce an artifact.
pub trait Backend {
    fn name(&self) -> &str;
    fn generate(&self, tree: &IKunTree) -> ChomskyResult<BackendArtifact>;

    /// 获取该后端的成本模型
    fn get_model(&self) -> &dyn CostModel {
        &chomsky_cost::DEFAULT_COST_MODEL
    }
}

pub struct IKunExtractor<'a, A, C>
where
    A: Analysis<IKun>,
    A::Data: HasDebugInfo,
    C: CostModel,
{
    pub egraph: &'a EGraph<IKun, A>,
    pub cost_model: C,
    pub costs: HashMap<Id, (Cost, IKun)>,
}

impl<'a, A, C> IKunExtractor<'a, A, C>
where
    A: Analysis<IKun>,
    A::Data: HasDebugInfo,
    C: CostModel,
{
    pub fn new(egraph: &'a EGraph<IKun, A>, cost_model: C) -> Self {
        let mut extractor = Self {
            egraph,
            cost_model,
            costs: HashMap::new(),
        };
        extractor.find_best();
        extractor
    }

    fn find_best(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;
            for entry in self.egraph.classes.iter() {
                let id = *entry.key();
                let eclass = entry.value();

                for node in &eclass.nodes {
                    let mut node_cost = self.cost_model.cost(node);
                    let mut can_compute = true;

                    for child in node.children() {
                        let child_id = self.egraph.union_find.find(child);
                        if let Some((child_cost, _)) = self.costs.get(&child_id) {
                            node_cost = node_cost.add(child_cost);
                        } else {
                            can_compute = false;
                            break;
                        }
                    }

                    if can_compute {
                        let current_best = self.costs.get(&id);

                        if current_best.is_none()
                            || node_cost.score() < current_best.unwrap().0.score()
                        {
                            self.costs.insert(id, (node_cost, node.clone()));
                            changed = true;
                        }
                    }
                }
            }
        }
    }

    pub fn extract(&self, id: Id) -> IKunTree {
        let root = self.egraph.union_find.find(id);
        let (_, node) = self.costs.get(&root).expect("No cost found for eclass");

        let mut tree = match node {
            IKun::Constant(v) => IKunTree::Constant(*v),
            IKun::FloatConstant(v) => IKunTree::FloatConstant(*v),
            IKun::BooleanConstant(v) => IKunTree::BooleanConstant(*v),
            IKun::StringConstant(s) => IKunTree::StringConstant(s.clone()),
            IKun::Symbol(s) => IKunTree::Symbol(s.clone()),
            IKun::Map(f, x) => {
                IKunTree::Map(Box::new(self.extract(*f)), Box::new(self.extract(*x)))
            }
            IKun::Filter(f, x) => {
                IKunTree::Filter(Box::new(self.extract(*f)), Box::new(self.extract(*x)))
            }
            IKun::Reduce(f, init, list) => IKunTree::Reduce(
                Box::new(self.extract(*f)),
                Box::new(self.extract(*init)),
                Box::new(self.extract(*list)),
            ),
            IKun::StateUpdate(var, val) => {
                IKunTree::StateUpdate(Box::new(self.extract(*var)), Box::new(self.extract(*val)))
            }
            IKun::Choice(cond, t, f) => IKunTree::Choice(
                Box::new(self.extract(*cond)),
                Box::new(self.extract(*t)),
                Box::new(self.extract(*f)),
            ),
            IKun::Repeat(cond, body) => {
                IKunTree::Repeat(Box::new(self.extract(*cond)), Box::new(self.extract(*body)))
            }
            IKun::LifeCycle(setup, cleanup) => IKunTree::LifeCycle(
                Box::new(self.extract(*setup)),
                Box::new(self.extract(*cleanup)),
            ),
            IKun::Meta(body) => IKunTree::Meta(Box::new(self.extract(*body))),
            IKun::Trap(body) => IKunTree::Trap(Box::new(self.extract(*body))),
            IKun::Return(val) => IKunTree::Return(Box::new(self.extract(*val))),
            IKun::Seq(ids) => IKunTree::Seq(ids.iter().map(|&id| self.extract(id)).collect()),
            IKun::Compose(a, b) => {
                IKunTree::Compose(Box::new(self.extract(*a)), Box::new(self.extract(*b)))
            }
            IKun::WithContext(ctx, body) => {
                IKunTree::WithContext(Box::new(self.extract(*ctx)), Box::new(self.extract(*body)))
            }
            IKun::WithConstraint(constraint, body) => IKunTree::WithConstraint(
                Box::new(self.extract(*constraint)),
                Box::new(self.extract(*body)),
            ),
            IKun::CpuContext => IKunTree::CpuContext,
            IKun::GpuContext => IKunTree::GpuContext,
            IKun::AsyncContext => IKunTree::AsyncContext,
            IKun::SpatialContext => IKunTree::SpatialContext,
            IKun::ComptimeContext => IKunTree::ComptimeContext,
            IKun::ResourceContext => IKunTree::ResourceContext,
            IKun::SafeContext => IKunTree::SafeContext,
            IKun::EffectConstraint(e) => IKunTree::EffectConstraint(e.clone()),
            IKun::OwnershipConstraint(o) => IKunTree::OwnershipConstraint(o.clone()),
            IKun::TypeConstraint(t) => IKunTree::TypeConstraint(t.clone()),
            IKun::AtomicConstraint => IKunTree::AtomicConstraint,
            IKun::Extension(name, args) => IKunTree::Extension(
                name.clone(),
                args.iter().map(|&id| self.extract(id)).collect(),
            ),
            IKun::CrossLangCall(CrossLanguageCall {
                language: lang,
                module_path: group,
                function_name: func,
                arguments: args,
            }) => IKunTree::CrossLangCall {
                language: lang.clone(),
                module_path: group.clone(),
                function_name: func.clone(),
                arguments: args.iter().map(|&id| self.extract(id)).collect(),
            },
            IKun::GpuMap(f, x) => {
                IKunTree::GpuMap(Box::new(self.extract(*f)), Box::new(self.extract(*x)))
            }
            IKun::CpuMap(f, x) => {
                IKunTree::CpuMap(Box::new(self.extract(*f)), Box::new(self.extract(*x)))
            }
            IKun::TiledMap(size, f, x) => IKunTree::TiledMap(
                *size,
                Box::new(self.extract(*f)),
                Box::new(self.extract(*x)),
            ),
            IKun::VectorizedMap(width, f, x) => IKunTree::VectorizedMap(
                *width,
                Box::new(self.extract(*f)),
                Box::new(self.extract(*x)),
            ),
            IKun::UnrolledMap(factor, f, x) => IKunTree::UnrolledMap(
                *factor,
                Box::new(self.extract(*f)),
                Box::new(self.extract(*x)),
            ),
            IKun::SoAMap(f, x) => {
                IKunTree::SoAMap(Box::new(self.extract(*f)), Box::new(self.extract(*x)))
            }
            IKun::SoALayout(x) => IKunTree::SoALayout(Box::new(self.extract(*x))),
            IKun::AoSLayout(x) => IKunTree::AoSLayout(Box::new(self.extract(*x))),
            IKun::Tiled(size, x) => IKunTree::Tiled(*size, Box::new(self.extract(*x))),
            IKun::Unrolled(factor, x) => IKunTree::Unrolled(*factor, Box::new(self.extract(*x))),
            IKun::Vectorized(width, x) => IKunTree::Vectorized(*width, Box::new(self.extract(*x))),
            IKun::Pipe(a, b) => {
                IKunTree::Pipe(Box::new(self.extract(*a)), Box::new(self.extract(*b)))
            }
            IKun::Reg(x) => IKunTree::Reg(Box::new(self.extract(*x))),
            IKun::Lambda(params, body) => {
                IKunTree::Lambda(params.clone(), Box::new(self.extract(*body)))
            }
            IKun::Apply(func, args) => IKunTree::Apply(
                Box::new(self.extract(*func)),
                args.iter().map(|&id| self.extract(id)).collect(),
            ),
            IKun::Closure(body, captured) => IKunTree::Closure(
                Box::new(self.extract(*body)),
                captured.iter().map(|&id| self.extract(id)).collect(),
            ),
            IKun::ResourceClone(x) => IKunTree::ResourceClone(Box::new(self.extract(*x))),
            IKun::ResourceDrop(x) => IKunTree::ResourceDrop(Box::new(self.extract(*x))),
            IKun::Import(m, s) => IKunTree::Import(m.clone(), s.clone()),
            IKun::Export(s, body) => IKunTree::Export(s.clone(), Box::new(self.extract(*body))),
            IKun::Module(m, items) => IKunTree::Module(
                m.clone(),
                items.iter().map(|&id| self.extract(id)).collect(),
            ),
        };

        if let Some(loc) = self.egraph.get_class(root).data.get_locs().first() {
            tree = IKunTree::Source(*loc, Box::new(tree));
        }
        tree
    }

    pub fn get_best_node(&self, id: Id) -> Option<&IKun> {
        let root = self.egraph.union_find.find(id);
        self.costs.get(&root).map(|(_, node)| node)
    }

    pub fn get_best_cost(&self, id: Id) -> Option<Cost> {
        let root = self.egraph.union_find.find(id);
        self.costs.get(&root).map(|(cost, _)| *cost)
    }
}

/// A DAG-aware extractor that attempts to find the optimal extraction
/// considering node sharing. This is a placeholder for a full ILP solver.
pub struct IKunIlpExtractor<'a, A: Analysis<IKun>, C: CostModel> {
    pub egraph: &'a EGraph<IKun, A>,
    pub cost_model: C,
}

impl<'a, A: Analysis<IKun>, C: CostModel + Clone> IKunIlpExtractor<'a, A, C> {
    pub fn new(egraph: &'a EGraph<IKun, A>, cost_model: C) -> Self {
        Self { egraph, cost_model }
    }

    /// Performs extraction. Currently uses the greedy extractor as a fallback.
    /// In a full implementation, this would use an ILP solver to find the
    /// minimum cost subgraph that covers the root e-class.
    pub fn extract(&self, id: Id) -> IKunTree
    where
        A::Data: chomsky_uir::egraph::HasDebugInfo,
    {
        let extractor = IKunExtractor::new(self.egraph, self.cost_model.clone());
        extractor.extract(id)
    }
}
