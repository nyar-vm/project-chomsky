use chomsky_cost::{Cost, CostModel};
use chomsky_extract::{IKunExtractor, IKunTree};
use chomsky_uir::{EGraph, IKun};

struct MockCostModel;
impl CostModel for MockCostModel {
    fn cost(&self, enode: &IKun) -> Cost {
        match enode {
            IKun::Constant(_) => Cost {
                latency: 1.0,
                ..Default::default()
            },
            IKun::Map(_, _) => Cost {
                latency: 10.0,
                ..Default::default()
            },
            _ => Cost::default(),
        }
    }
}

#[test]
fn test_extraction_basic() {
    let egraph = EGraph::<IKun, ()>::new();
    let c = egraph.add(IKun::Constant(42));

    let extractor = IKunExtractor::new(&egraph, MockCostModel);
    let tree = extractor.extract(c);

    if let IKunTree::Constant(val) = tree {
        assert_eq!(val, 42);
    } else {
        panic!("Expected Constant");
    }
}

#[test]
fn test_extraction_nested() {
    let egraph = EGraph::<IKun, ()>::new();
    let f = egraph.add(IKun::Symbol("f".to_string()));
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let map = egraph.add(IKun::Map(f, x));

    let extractor = IKunExtractor::new(&egraph, MockCostModel);
    let tree = extractor.extract(map);

    if let IKunTree::Map(f_tree, x_tree) = tree {
        if let IKunTree::Symbol(s) = *f_tree {
            assert_eq!(s, "f");
        } else {
            panic!("Expected Symbol f");
        }
        if let IKunTree::Symbol(s) = *x_tree {
            assert_eq!(s, "x");
        } else {
            panic!("Expected Symbol x");
        }
    } else {
        panic!("Expected Map");
    }
}
