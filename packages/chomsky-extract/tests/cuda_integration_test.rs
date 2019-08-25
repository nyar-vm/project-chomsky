use chomsky_cost::GpuCostModel;
use chomsky_extract::IKunExtractor;
use chomsky_uir::egraph::{Analysis, EGraph};
use chomsky_uir::{IKun, IKunTree};

struct NoAnalysis;
impl Analysis<IKun> for NoAnalysis {
    type Data = ();
    fn make(egraph: &EGraph<IKun, Self>, enode: &IKun) -> Self::Data {
        ()
    }
    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> bool {
        false
    }
}

#[test]
fn test_cuda_kernel_extraction() {
    let mut egraph: EGraph<IKun, NoAnalysis> = EGraph::default();

    // Create a program with a generic Map and a GpuContext
    let f = egraph.add(IKun::Symbol("my_kernel".to_string()));
    let x = egraph.add(IKun::Symbol("my_data".to_string()));
    let generic_map = egraph.add(IKun::Map(f, x));

    // Add an optimized GpuMap variant to the same e-class
    let gpu_map = egraph.add(IKun::GpuMap(f, x));
    egraph.union(generic_map, gpu_map);

    // Add a GpuContext wrapper
    let gpu_ctx = egraph.add(IKun::GpuContext);
    let root = egraph.add(IKun::WithContext(gpu_ctx, generic_map));

    egraph.rebuild();

    // Extract with GpuCostModel
    let extractor = IKunExtractor::new(&egraph, GpuCostModel);
    let tree = extractor.extract(root);

    // Verify that the extracted tree contains GpuMap instead of generic Map
    if let IKunTree::WithContext(_, body) = tree {
        match *body {
            IKunTree::GpuMap(_, _) => println!("Successfully extracted GpuMap kernel!"),
            _ => panic!("Expected GpuMap in GpuContext, but got {:?}", body),
        }
    } else {
        panic!("Expected WithContext at root, but got {:?}", tree);
    }
}
