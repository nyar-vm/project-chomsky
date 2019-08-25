#![warn(missing_docs)]

use chomsky_uir::Analysis;
use chomsky_uir::egraph::EGraph;
use chomsky_uir::intent::IKun;

pub trait RewriteRule<A: Analysis<IKun>>: Send + Sync {
    fn name(&self) -> &str;
    fn apply(&self, egraph: &EGraph<IKun, A>);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleCategory {
    Algebraic,
    Architectural,
    Aggressive,
    Concretization,
}

pub struct RewriteRegistry<A: Analysis<IKun>> {
    rules: Vec<(RuleCategory, Box<dyn RewriteRule<A>>)>,
}

impl<A: Analysis<IKun>> RewriteRegistry<A> {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn register(&mut self, category: RuleCategory, rule: Box<dyn RewriteRule<A>>) {
        self.rules.push((category, rule));
    }

    pub fn get_rules(
        &self,
        category: RuleCategory,
    ) -> impl Iterator<Item = &Box<dyn RewriteRule<A>>> {
        self.rules
            .iter()
            .filter(move |(cat, _)| *cat == category)
            .map(|(_, rule)| rule)
    }

    pub fn all_rules(&self) -> impl Iterator<Item = &Box<dyn RewriteRule<A>>> {
        self.rules.iter().map(|(_, rule)| rule)
    }

    pub fn into_rules(self) -> Vec<(RuleCategory, Box<dyn RewriteRule<A>>)> {
        self.rules
    }
}

pub struct SaturationScheduler {
    pub fuel: usize,
    pub timeout: std::time::Duration,
}

impl Default for SaturationScheduler {
    fn default() -> Self {
        Self {
            fuel: 10,
            timeout: std::time::Duration::from_secs(5),
        }
    }
}

impl SaturationScheduler {
    pub fn run<A: Analysis<IKun>>(&self, egraph: &EGraph<IKun, A>, registry: &RewriteRegistry<A>) {
        let start_time = std::time::Instant::now();

        for i in 0..self.fuel {
            if start_time.elapsed() > self.timeout {
                println!("Saturation timeout reached at iteration {}", i);
                break;
            }

            let prev_nodes_count = egraph.memo.len();
            let prev_classes_count = egraph.classes.len();

            // Apply all rules
            for rule in registry.all_rules() {
                rule.apply(egraph);
            }

            egraph.rebuild();

            let current_nodes_count = egraph.memo.len();
            let current_classes_count = egraph.classes.len();

            if current_nodes_count == prev_nodes_count
                && current_classes_count == prev_classes_count
            {
                println!("Saturation reached at iteration {}", i);
                break;
            }
        }
    }
}

pub fn ikun_registry<A: Analysis<IKun> + 'static>() -> RewriteRegistry<A> {
    RewriteRegistry::new()
}

pub fn default_rules<A: Analysis<IKun> + 'static>() -> Vec<Box<dyn RewriteRule<A>>> {
    vec![]
}
