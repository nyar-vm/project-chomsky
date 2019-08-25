use chomsky_rule_engine::{RewriteRegistry, RuleCategory, SaturationScheduler};
use chomsky_rules::{
    FilterFusion, FilterMapFusion, FilterReduceFusion, MapFilterFusion, MapFusion, MapReduceFusion,
};
use chomsky_uir::{EGraph, IKun, Id};

#[test]
fn test_filter_reduce_fusion() {
    let mut egraph = EGraph::<IKun, ()>::new();

    let x = egraph.add(IKun::Symbol("x".to_string()));
    let p = egraph.add(IKun::Symbol("p".to_string()));
    let f = egraph.add(IKun::Symbol("f".to_string()));
    let init = egraph.add(IKun::Constant(0));

    // filter(p, x)
    let filter_node = egraph.add(IKun::Filter(p, x));
    // reduce(f, init, filter(p, x))
    let reduce_node = egraph.add(IKun::Reduce(f, init, filter_node));

    let mut registry = RewriteRegistry::new();
    registry.register(RuleCategory::Algebraic, Box::new(FilterReduceFusion));

    let scheduler = SaturationScheduler::default();
    scheduler.run(&mut egraph, &registry);

    // Verify that reduce_node is unified with loop_filter_reduce(p, f, init, x)
    let fused_node = egraph.add(IKun::Extension(
        "loop_filter_reduce".to_string(),
        vec![p, f, init, x],
    ));

    assert_eq!(
        egraph.union_find.find(reduce_node),
        egraph.union_find.find(fused_node)
    );
}

#[test]
fn test_map_filter_fusion() {
    let mut egraph = EGraph::<IKun, ()>::new();

    let x = egraph.add(IKun::Symbol("x".to_string()));
    let f = egraph.add(IKun::Symbol("f".to_string()));
    let p = egraph.add(IKun::Symbol("p".to_string()));

    // map(f, x)
    let map_node = egraph.add(IKun::Map(f, x));
    // filter(p, map(f, x))
    let filter_node = egraph.add(IKun::Filter(p, map_node));

    let mut registry = RewriteRegistry::new();
    registry.register(RuleCategory::Algebraic, Box::new(MapFilterFusion));

    let scheduler = SaturationScheduler::default();
    scheduler.run(&mut egraph, &registry);

    // Verify that filter_node is unified with map_filter(p, f, x)
    let fused_node = egraph.add(IKun::Extension("map_filter".to_string(), vec![p, f, x]));

    assert_eq!(
        egraph.union_find.find(filter_node),
        egraph.union_find.find(fused_node)
    );
}

#[test]
fn test_map_fusion() {
    let mut egraph = EGraph::<IKun, ()>::new();
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let f = egraph.add(IKun::Symbol("f".to_string()));
    let g = egraph.add(IKun::Symbol("g".to_string()));

    // map(g, x)
    let inner_map = egraph.add(IKun::Map(g, x));
    // map(f, map(g, x))
    let outer_map = egraph.add(IKun::Map(f, inner_map));

    let mut registry = RewriteRegistry::new();
    registry.register(RuleCategory::Algebraic, Box::new(MapFusion));

    let scheduler = SaturationScheduler::default();
    scheduler.run(&mut egraph, &registry);

    // Verify that outer_map is unified with map(compose(f, g), x)
    let compose = egraph.add(IKun::Compose(f, g));
    let fused_node = egraph.add(IKun::Map(compose, x));

    assert_eq!(
        egraph.union_find.find(outer_map),
        egraph.union_find.find(fused_node)
    );
}

#[test]
fn test_filter_fusion() {
    let mut egraph = EGraph::<IKun, ()>::new();
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let p1 = egraph.add(IKun::Symbol("p1".to_string()));
    let p2 = egraph.add(IKun::Symbol("p2".to_string()));

    // filter(p2, x)
    let inner_filter = egraph.add(IKun::Filter(p2, x));
    // filter(p1, filter(p2, x))
    let outer_filter = egraph.add(IKun::Filter(p1, inner_filter));

    let mut registry = RewriteRegistry::new();
    registry.register(RuleCategory::Algebraic, Box::new(FilterFusion));

    let scheduler = SaturationScheduler::default();
    scheduler.run(&mut egraph, &registry);

    // Verify that outer_filter is unified with filter(and_predicate(p2, p1), x)
    let combined_p = egraph.add(IKun::Extension("and_predicate".to_string(), vec![p2, p1]));
    let fused_node = egraph.add(IKun::Filter(combined_p, x));

    assert_eq!(
        egraph.union_find.find(outer_filter),
        egraph.union_find.find(fused_node)
    );
}

#[test]
fn test_filter_map_fusion() {
    let mut egraph = EGraph::<IKun, ()>::new();
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let f = egraph.add(IKun::Symbol("f".to_string()));
    let p = egraph.add(IKun::Symbol("p".to_string()));

    // filter(p, x)
    let filter_node = egraph.add(IKun::Filter(p, x));
    // map(f, filter(p, x))
    let map_node = egraph.add(IKun::Map(f, filter_node));

    let mut registry = RewriteRegistry::new();
    registry.register(RuleCategory::Algebraic, Box::new(FilterMapFusion));

    let scheduler = SaturationScheduler::default();
    scheduler.run(&mut egraph, &registry);

    // Verify that map_node is unified with filter_map(f, p, x)
    let fused_node = egraph.add(IKun::Extension("filter_map".to_string(), vec![f, p, x]));

    assert_eq!(
        egraph.union_find.find(map_node),
        egraph.union_find.find(fused_node)
    );
}

#[test]
fn test_map_reduce_fusion() {
    let mut egraph = EGraph::<IKun, ()>::new();
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let f = egraph.add(IKun::Symbol("f".to_string()));
    let g = egraph.add(IKun::Symbol("g".to_string()));
    let init = egraph.add(IKun::Constant(0));

    // map(f, x)
    let map_node = egraph.add(IKun::Map(f, x));
    // reduce(g, init, map(f, x))
    let reduce_node = egraph.add(IKun::Reduce(g, init, map_node));

    let mut registry = RewriteRegistry::new();
    registry.register(RuleCategory::Algebraic, Box::new(MapReduceFusion));

    let scheduler = SaturationScheduler::default();
    scheduler.run(&mut egraph, &registry);

    // Verify that reduce_node is unified with loop_map_reduce(f, g, init, x)
    let fused_node = egraph.add(IKun::Extension(
        "loop_map_reduce".to_string(),
        vec![f, g, init, x],
    ));

    assert_eq!(
        egraph.union_find.find(reduce_node),
        egraph.union_find.find(fused_node)
    );
}
