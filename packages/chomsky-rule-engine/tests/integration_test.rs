use chomsky_rule_engine::{RewriteRegistry, RewriteRule, RuleCategory, SaturationScheduler};
use chomsky_uir::{EGraph, IKun};

struct TestRule;

// We use () as Analysis, which is already implemented in chomsky-uir

#[derive(Default)]
struct TestAnalysis;
impl chomsky_uir::Analysis<IKun> for TestAnalysis {
    type Data = ();
    fn make(_: &EGraph<IKun, Self>, _: &IKun) -> Self::Data {
        ()
    }
    fn merge(&mut self, _: &mut Self::Data, _: Self::Data) -> bool {
        false
    }
}

impl RewriteRule<TestAnalysis> for TestRule {
    fn name(&self) -> &str {
        "test-rule"
    }
    fn apply(&self, egraph: &EGraph<IKun, TestAnalysis>) {
        let mut matches = Vec::new();
        for entry in egraph.classes.iter() {
            let (&id, eclass) = entry.pair();
            for node in &eclass.nodes {
                if let IKun::Symbol(s) = node {
                    if s == "x" {
                        matches.push(id);
                    }
                }
            }
        }
        for id in matches {
            let y = egraph.add(IKun::Symbol("y".to_string()));
            egraph.union(id, y);
        }
    }
}

#[test]
fn test_registry() {
    let mut registry = RewriteRegistry::<TestAnalysis>::new();
    registry.register(RuleCategory::Algebraic, Box::new(TestRule));

    assert_eq!(registry.all_rules().count(), 1);
    assert_eq!(registry.get_rules(RuleCategory::Algebraic).count(), 1);
    assert_eq!(registry.get_rules(RuleCategory::Architectural).count(), 0);
}

#[test]
fn test_saturation_scheduler() {
    let egraph = EGraph::<IKun, TestAnalysis>::new();
    let mut registry = RewriteRegistry::<TestAnalysis>::new();
    registry.register(RuleCategory::Algebraic, Box::new(TestRule));

    let x = egraph.add(IKun::Symbol("x".to_string()));
    let y = egraph.add(IKun::Symbol("y".to_string()));

    assert_ne!(egraph.union_find.find(x), egraph.union_find.find(y));

    let scheduler = SaturationScheduler::default();
    scheduler.run(&egraph, &registry);

    assert_eq!(egraph.union_find.find(x), egraph.union_find.find(y));
}
