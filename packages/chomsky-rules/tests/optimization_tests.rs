use chomsky_rule_engine::{RewriteRegistry, RuleCategory, SaturationScheduler};
use chomsky_rules::{ContextSimplification, MetaSimplification, TrapSimplification};
use chomsky_uir::{EGraph, IKun};

#[test]
fn test_meta_simplification() {
    let mut egraph = EGraph::<IKun, ()>::new();

    // Constant(10)
    let c10 = egraph.add(IKun::Constant(10));
    // Meta(Constant(10))
    let meta_c10 = egraph.add(IKun::Meta(c10));
    // Meta(Meta(Constant(10)))
    let meta_meta_c10 = egraph.add(IKun::Meta(meta_c10));

    let mut registry = RewriteRegistry::new();
    registry.register(RuleCategory::Algebraic, Box::new(MetaSimplification));

    let scheduler = SaturationScheduler::default();
    scheduler.run(&mut egraph, &registry);

    assert_eq!(
        egraph.union_find.find(meta_c10),
        egraph.union_find.find(c10)
    );
    assert_eq!(
        egraph.union_find.find(meta_meta_c10),
        egraph.union_find.find(c10)
    );
}

#[test]
fn test_context_simplification() {
    let mut egraph = EGraph::<IKun, ()>::new();

    let c1 = egraph.add(IKun::Constant(1));
    let async_ctx = egraph.add(IKun::AsyncContext);

    // WithContext(Async, Constant(1))
    let async_c1 = egraph.add(IKun::WithContext(async_ctx, c1));
    // WithContext(Async, WithContext(Async, Constant(1)))
    let nested_async_c1 = egraph.add(IKun::WithContext(async_ctx, async_c1));

    let mut registry = RewriteRegistry::new();
    registry.register(RuleCategory::Algebraic, Box::new(ContextSimplification));

    let scheduler = SaturationScheduler::default();
    scheduler.run(&mut egraph, &registry);

    assert_eq!(
        egraph.union_find.find(nested_async_c1),
        egraph.union_find.find(async_c1)
    );
}

#[test]
fn test_trap_simplification() {
    let mut egraph = EGraph::<IKun, ()>::new();

    let c1 = egraph.add(IKun::Constant(1));
    // Trap(Constant(1))
    let trap_c1 = egraph.add(IKun::Trap(c1));
    // Trap(Trap(Constant(1)))
    let nested_trap_c1 = egraph.add(IKun::Trap(trap_c1));

    let mut registry = RewriteRegistry::new();
    registry.register(RuleCategory::Algebraic, Box::new(TrapSimplification));

    let scheduler = SaturationScheduler::default();
    scheduler.run(&mut egraph, &registry);

    assert_eq!(
        egraph.union_find.find(nested_trap_c1),
        egraph.union_find.find(trap_c1)
    );
}
