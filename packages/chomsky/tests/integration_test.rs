use chomsky_full::uir::{EGraph, IKun};

#[test]
fn test_full_reexport_check() {
    let egraph = EGraph::<IKun, ()>::new();
    let x = egraph.add(IKun::Symbol("x".to_string()));
    // If this compiles, re-exports are working
    assert_eq!(egraph.union_find.find(x), x);
}
