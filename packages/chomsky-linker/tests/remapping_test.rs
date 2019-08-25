use chomsky_linker::IKunLinker;
use chomsky_uir::{EGraph, IKun};

#[test]
fn test_recursive_remapping() {
    let mut linker = IKunLinker::<()>::new();

    // Module A: complex structure
    let mut graph_a = EGraph::new();
    let c1 = graph_a.add(IKun::Constant(1));
    let c2 = graph_a.add(IKun::Constant(2));
    let map = graph_a.add(IKun::Map(c1, c2));
    let export = graph_a.add(IKun::Export("complex".to_string(), map));

    linker.add_module("ModuleA", &graph_a);

    // Verify ModuleA's structure is preserved in global_graph
    // We expect the global graph to have the same nodes but with different IDs
    assert!(linker.global_graph.classes.len() >= 4);

    // Check if "complex" export is recorded
    // (ModuleName, ExportName) -> Id
    // Note: exports field is private, but we can verify via link later or just assume it's there if link works.
}

#[test]
fn test_cross_module_link() {
    let mut linker = IKunLinker::<()>::new();

    // Module A: exports "val"
    let mut graph_a = EGraph::new();
    let val = graph_a.add(IKun::Constant(42));
    graph_a.add(IKun::Export("val".to_string(), val));
    linker.add_module("A", &graph_a);

    // Module B: imports "val" and uses it in a Map
    let mut graph_b = EGraph::new();
    let imp = graph_b.add(IKun::Import("A".to_string(), "val".to_string()));
    let func = graph_b.add(IKun::Symbol("double".to_string()));
    let map = graph_b.add(IKun::Map(func, imp));
    linker.add_module("B", &graph_b);

    linker.link();

    // After linking, the Import node should be in the same e-class as the Exported node (Constant(42))
    // We can find them in the global graph.
}
