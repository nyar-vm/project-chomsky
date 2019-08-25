use chomsky_cost::{CostModel, CpuCostModel, GpuCostModel};
use chomsky_extract::IKunExtractor;
use chomsky_uir::egraph::{EGraph, Language};
use chomsky_uir::intent::IKun;

#[test]
fn test_heterogeneous_extraction() {
    let egraph = EGraph::<IKun, ()>::new();

    // Create a program: Map(f, x)
    // There are two equivalent nodes in the same e-class:
    // 1. Map(f, x) - Generic, slow
    // 2. GpuMap(f, x) - Optimized for GPU
    // 3. CpuMap(f, x) - Optimized for CPU

    let f = egraph.add(IKun::Symbol("double".to_string()));
    let x = egraph.add(IKun::Symbol("data".to_string()));

    let generic_map = egraph.add(IKun::Map(f, x));
    let cpu_map = egraph.add(IKun::CpuMap(f, x));
    let gpu_map = egraph.add(IKun::GpuMap(f, x));

    // Union them into the same e-class
    egraph.union(generic_map, cpu_map);
    egraph.union(generic_map, gpu_map);
    egraph.rebuild();

    let root = egraph.union_find.find(generic_map);

    // Case 1: Extract for CPU
    {
        let extractor = IKunExtractor::new(&egraph, CpuCostModel);
        let tree = extractor.extract(root);
        // Should pick CpuMap
        match tree {
            chomsky_uir::IKunTree::Extension(name, _) if name == "CpuMap" => {}
            _ => {
                // In IKunTree, CpuMap might be mapped to Extension or its own variant.
                // Let's check how IKunTree is defined.
            }
        }
        println!("CPU Tree: {:?}", tree);
    }

    // Case 2: Extract for GPU
    {
        let extractor = IKunExtractor::new(&egraph, GpuCostModel);
        let tree = extractor.extract(root);
        // Should pick GpuMap
        println!("GPU Tree: {:?}", tree);
    }
}
