use chomsky_uir::{ConstraintAnalysis, EGraph, IKun, Id};

#[test]
fn test_egraph_basic_add_and_find() {
    let egraph = EGraph::<IKun, ConstraintAnalysis>::new();

    let a = egraph.add(IKun::Symbol("a".to_string()));
    let b = egraph.add(IKun::Symbol("b".to_string()));

    assert_ne!(a, b);

    let a_again = egraph.add(IKun::Symbol("a".to_string()));
    assert_eq!(egraph.union_find.find(a), egraph.union_find.find(a_again));
}

#[test]
fn test_egraph_union() {
    let egraph = EGraph::<IKun, ConstraintAnalysis>::new();

    let a = egraph.add(IKun::Symbol("a".to_string()));
    let b = egraph.add(IKun::Symbol("b".to_string()));

    egraph.union(a, b);

    assert_eq!(egraph.union_find.find(a), egraph.union_find.find(b));

    // Check if the eclass contains both nodes
    let root = egraph.union_find.find(a);
    let eclass = egraph.classes.get(&root).unwrap();
    assert_eq!(eclass.nodes.len(), 2);
}

#[test]
fn test_egraph_complex_nodes() {
    let egraph = EGraph::<IKun, ConstraintAnalysis>::new();

    let f = egraph.add(IKun::Symbol("f".to_string()));
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let y = egraph.add(IKun::Symbol("y".to_string()));

    let map_fx = egraph.add(IKun::Map(f, x));
    let map_fy = egraph.add(IKun::Map(f, y));

    assert_ne!(map_fx, map_fy);

    // Union x and y
    egraph.union(x, y);

    // In E-graphs, we need to rebuild to restore congruence after union
    egraph.rebuild();

    // After union and rebuild, adding map(f, y) should give the same id as map(f, x)
    let map_fy_canonical = egraph.add(IKun::Map(f, y));
    assert_eq!(
        egraph.union_find.find(map_fx),
        egraph.union_find.find(map_fy_canonical)
    );
}

#[test]
fn test_context_handling() {
    let egraph = EGraph::<IKun, ConstraintAnalysis>::new();

    let ctx = egraph.add(IKun::GpuContext);
    let body = egraph.add(IKun::Symbol("kernel".to_string()));
    let with_ctx = egraph.add(IKun::WithContext(ctx, body));

    let root = egraph.union_find.find(with_ctx);
    let eclass = egraph.classes.get(&root).unwrap();

    assert!(
        eclass
            .nodes
            .iter()
            .any(|n| matches!(n, IKun::WithContext(_, _)))
    );
}
