use chomsky_cost::CostModel;
use chomsky_extract::{IKunExtractor, IKunTree};
use chomsky_rule_engine::{RewriteRegistry, SaturationScheduler};
use chomsky_rules::register_standard_rules;
use chomsky_uir::IKun;
use chomsky_uir::egraph::{Analysis, EGraph, Id};

/// A universal optimizer that integrates saturation search and cost-based extraction.
/// This optimizer is designed to be backend-agnostic, using standard rules and cost models
/// to find the best program variant in the E-Graph.
pub struct UniversalOptimizer<A: Analysis<IKun> + 'static>
where
    A::Data: chomsky_uir::egraph::HasDebugInfo,
{
    pub registry: RewriteRegistry<A>,
    pub scheduler: SaturationScheduler,
    pub egraph: EGraph<IKun, A>,
}

impl<A: Analysis<IKun> + 'static> Default for UniversalOptimizer<A>
where
    A: Default,
    A::Data: chomsky_uir::egraph::HasDebugInfo,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Analysis<IKun> + 'static> UniversalOptimizer<A>
where
    A::Data: chomsky_uir::egraph::HasDebugInfo,
{
    /// Creates a new UniversalOptimizer with standard rules registered.
    pub fn new() -> Self
    where
        A: Default,
    {
        let mut registry = RewriteRegistry::new();
        register_standard_rules(&mut registry);

        Self {
            registry,
            scheduler: SaturationScheduler::default(),
            egraph: EGraph::new(),
        }
    }

    /// Adds an intent to the internal EGraph.
    pub fn add_intent(&mut self, ikun: &IKun) -> Id {
        self.egraph.add(ikun.clone())
    }

    /// Runs saturation search on the internal EGraph.
    pub fn saturate(&mut self) {
        self.scheduler.run(&self.egraph, &self.registry);
    }

    /// Extracts the best candidate from the internal EGraph using a cost model.
    pub fn extract<C: CostModel>(&self, root_id: Id, cost_model: C) -> IKunTree {
        let extractor = IKunExtractor::new(&self.egraph, cost_model);
        extractor.extract(root_id)
    }

    /// Optimizes the program starting from the given root ID.
    /// 1. Runs saturation search using the registered rules.
    /// 2. Extracts the best candidate based on the provided cost model.
    pub fn optimize<C: CostModel>(
        &self,
        egraph: &EGraph<IKun, A>,
        root_id: Id,
        cost_model: C,
    ) -> IKunTree {
        // Run saturation search
        self.scheduler.run(egraph, &self.registry);

        // Extract the best variant from the saturated E-Graph
        let extractor = IKunExtractor::new(egraph, cost_model);
        extractor.extract(root_id)
    }

    /// Registers a custom rule to the optimizer.
    pub fn register_rule(
        &mut self,
        category: chomsky_rule_engine::RuleCategory,
        rule: Box<dyn chomsky_rule_engine::RewriteRule<A>>,
    ) {
        self.registry.register(category, rule);
    }
}
