use chomsky_rule_engine::RewriteRule;
use chomsky_rules::{FilterFusion, MapFusion, MapToLoop};
use chomsky_uir::{EGraph, IKun};

#[test]
fn test_map_fusion() {
    let egraph = EGraph::<IKun>::new();

    // Construct Map(f, Map(g, x))
    let f = egraph.add(IKun::Symbol("f".to_string()));
    let g = egraph.add(IKun::Symbol("g".to_string()));
    let x = egraph.add(IKun::Symbol("x".to_string()));

    let map_inner = egraph.add(IKun::Map(g, x));
    let map_outer = egraph.add(IKun::Map(f, map_inner));

    let rule = MapFusion;
    rule.apply(&egraph);

    // Expected: Map(Seq(g, f), x)
    let seq_gf = egraph.add(IKun::Seq(vec![g, f]));
    let map_fused = egraph.add(IKun::Map(seq_gf, x));

    assert_eq!(
        egraph.union_find.find(map_outer),
        egraph.union_find.find(map_fused)
    );
}

#[test]
fn test_filter_fusion() {
    let egraph = EGraph::<IKun>::new();

    let p1 = egraph.add(IKun::Symbol("p1".to_string()));
    let p2 = egraph.add(IKun::Symbol("p2".to_string()));
    let x = egraph.add(IKun::Symbol("x".to_string()));

    let filter_inner = egraph.add(IKun::Filter(p2, x));
    let filter_outer = egraph.add(IKun::Filter(p1, filter_inner));

    let rule = FilterFusion;
    rule.apply(&egraph);

    // Expected: Filter(Extension("and_predicate", [p2, p1]), x)
    let combined_p = egraph.add(IKun::Extension("and_predicate".to_string(), vec![p2, p1]));
    let filter_fused = egraph.add(IKun::Filter(combined_p, x));

    assert_eq!(
        egraph.union_find.find(filter_outer),
        egraph.union_find.find(filter_fused)
    );
}

#[test]
fn test_map_to_loop() {
    let egraph = EGraph::<IKun>::new();

    let f = egraph.add(IKun::Symbol("f".to_string()));
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let map_node = egraph.add(IKun::Map(f, x));

    let rule = MapToLoop;
    rule.apply(&egraph);

    // Expected: Extension("loop_map", [f, x])
    let loop_node = egraph.add(IKun::Extension("loop_map".to_string(), vec![f, x]));

    assert_eq!(
        egraph.union_find.find(map_node),
        egraph.union_find.find(loop_node)
    );
}
