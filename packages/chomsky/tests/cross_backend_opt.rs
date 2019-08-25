use chomsky_cost::JsCostModel;
use chomsky_extract::IKunTree;
use chomsky_full::optimizer::UniversalOptimizer;
use chomsky_uir::IKun;
use chomsky_uir::egraph::EGraph;

#[test]
fn test_constant_folding_optimization() {
    let egraph: EGraph<IKun, ()> = EGraph::new();

    // (1 + 2) * 3
    let v1 = egraph.add(IKun::Constant(1));
    let v2 = egraph.add(IKun::Constant(2));
    let add = egraph.add(IKun::Extension("add".to_string(), vec![v1, v2]));
    let v3 = egraph.add(IKun::Constant(3));
    let root = egraph.add(IKun::Extension("mul".to_string(), vec![add, v3]));

    let optimizer = UniversalOptimizer::new();
    let cost_model = JsCostModel::new(true);

    let optimized_tree = optimizer.optimize(&egraph, root, cost_model);

    // Expecting 9
    assert!(matches!(optimized_tree, IKunTree::Constant(9)));
}

#[test]
fn test_algebraic_simplification_optimization() {
    let egraph: EGraph<IKun, ()> = EGraph::new();

    // x * 1 + 0
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let one = egraph.add(IKun::Constant(1));
    let mul = egraph.add(IKun::Extension("mul".to_string(), vec![x, one]));
    let zero = egraph.add(IKun::Constant(0));
    let root = egraph.add(IKun::Extension("add".to_string(), vec![mul, zero]));

    let optimizer = UniversalOptimizer::new();
    let cost_model = JsCostModel::new(true);

    let optimized_tree = optimizer.optimize(&egraph, root, cost_model);

    // Expecting x
    assert!(matches!(optimized_tree, IKunTree::Symbol(s) if s == "x"));
}

#[test]
fn test_map_fusion_optimization() {
    let egraph: EGraph<IKun, ()> = EGraph::new();

    // map(f, map(g, x))
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let g = egraph.add(IKun::Symbol("g".to_string()));
    let map_g = egraph.add(IKun::Map(g, x));
    let f = egraph.add(IKun::Symbol("f".to_string()));
    let root = egraph.add(IKun::Map(f, map_g));

    let optimizer = UniversalOptimizer::new();
    let cost_model = JsCostModel::new(true);

    let optimized_tree = optimizer.optimize(&egraph, root, cost_model);

    // Expecting fused map (either as Map, loop_map, TiledMap, or VectorizedMap)
    let func = match optimized_tree {
        IKunTree::Map(func, _) => func,
        IKunTree::Extension(name, args) if name == "loop_map" && args.len() == 2 => {
            Box::new(args[0].clone())
        }
        IKunTree::TiledMap(_, func, _) => func,
        IKunTree::VectorizedMap(_, func, _) => func,
        _ => panic!("Expected fused variant, got {:?}", optimized_tree),
    };

    if let IKunTree::Seq(ops) = *func {
        assert_eq!(ops.len(), 2);
        // In MapFusion, we added seq![g, f]
        if let IKunTree::Symbol(s) = &ops[0] {
            assert_eq!(s, "g");
        }
        if let IKunTree::Symbol(s) = &ops[1] {
            assert_eq!(s, "f");
        }
    } else {
        panic!("Expected Seq as map function, got {:?}", func);
    }
}

#[test]
fn test_strength_reduction_optimization() {
    let egraph: EGraph<IKun, ()> = EGraph::new();

    // x * 8
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let eight = egraph.add(IKun::Constant(8));
    let root = egraph.add(IKun::Extension("mul".to_string(), vec![x, eight]));

    let optimizer = UniversalOptimizer::new();
    // Use CpuCostModel to ensure shl is preferred over mul
    use chomsky_cost::CpuCostModel;
    let cost_model = CpuCostModel::new(16);

    let optimized_tree = optimizer.optimize(&egraph, root, cost_model);

    // Expecting x << 3
    if let IKunTree::Extension(op, args) = optimized_tree {
        assert_eq!(op, "shl");
        assert_eq!(args.len(), 2);
        if let IKunTree::Symbol(s) = &args[0] {
            assert_eq!(s, "x");
        }
        if let IKunTree::Constant(v) = &args[1] {
            assert_eq!(*v, 3);
        }
    } else {
        panic!("Expected Extension(shl), got {:?}", optimized_tree);
    }
}

#[test]
fn test_peephole_optimization() {
    let egraph: EGraph<IKun, ()> = EGraph::new();

    // x + x
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let root = egraph.add(IKun::Extension("add".to_string(), vec![x, x]));

    let optimizer = UniversalOptimizer::new();
    let cost_model = JsCostModel::new(true);

    let optimized_tree = optimizer.optimize(&egraph, root, cost_model);

    // Expecting x * 2 (which might then be strength reduced to x << 1)
    match optimized_tree {
        IKunTree::Extension(op, args) => {
            assert!(op == "mul" || op == "shl");
            if op == "mul" {
                if let IKunTree::Constant(v) = &args[1] {
                    assert_eq!(*v, 2);
                }
            } else {
                if let IKunTree::Constant(v) = &args[1] {
                    assert_eq!(*v, 1);
                }
            }
        }
        _ => panic!("Expected Extension(mul or shl), got {:?}", optimized_tree),
    }
}

#[test]
fn test_cross_backend_cost_sensitivity() {
    let egraph: EGraph<IKun, ()> = EGraph::new();

    // map(f, x)
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let f = egraph.add(IKun::Symbol("f".to_string()));
    let root = egraph.add(IKun::Map(f, x));

    let optimizer = UniversalOptimizer::new();

    // 1. JS with prefer_loop = true
    let js_cost_model = JsCostModel::new(true);
    let js_tree = optimizer.optimize(&egraph, root, js_cost_model);
    // It should be one of the optimized versions
    let is_optimized = matches!(js_tree, IKunTree::Extension(ref name, _) if name == "loop_map")
        || matches!(js_tree, IKunTree::VectorizedMap(_, _, _))
        || matches!(js_tree, IKunTree::TiledMap(_, _, _));
    assert!(
        is_optimized,
        "Expected an optimized variant, got {:?}",
        js_tree
    );

    // 2. A hypothetical backend that hates loops but loves builtins (size sensitive)
    // We can simulate this by making everything else infinite cost or very expensive
    struct TinyBuiltinModel;
    impl chomsky_cost::CostModel for TinyBuiltinModel {
        fn cost(&self, enode: &IKun) -> chomsky_cost::Cost {
            match enode {
                IKun::Map(_, _) => chomsky_cost::Cost {
                    latency: 1.0,
                    throughput: 1.0,
                    size: 0.1,
                    energy: 0.1,
                },
                _ => chomsky_cost::Cost {
                    latency: 100.0,
                    throughput: 0.1,
                    size: 100.0,
                    energy: 100.0,
                },
            }
        }
    }

    let builtin_tree = optimizer.optimize(&egraph, root, TinyBuiltinModel);
    assert!(matches!(builtin_tree, IKunTree::Map(_, _)));
}

#[test]
fn test_layout_transformation_optimization() {
    let egraph: EGraph<IKun, ()> = EGraph::new();

    // spatial_context { map(f, x) }
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let f = egraph.add(IKun::Symbol("f".to_string()));
    let map = egraph.add(IKun::Map(f, x));
    let ctx = egraph.add(IKun::SpatialContext);
    let root = egraph.add(IKun::WithContext(ctx, map));

    let optimizer = UniversalOptimizer::new();
    let cost_model = JsCostModel::new(true);

    let optimized_tree = optimizer.optimize(&egraph, root, cost_model);

    // Expecting WithContext(SpatialContext, SoAMap(f, x))
    if let IKunTree::WithContext(_, body) = optimized_tree {
        assert!(matches!(*body, IKunTree::SoAMap(_, _)));
    } else {
        panic!("Expected WithContext, got {:?}", optimized_tree);
    }
}

#[test]
fn test_loop_tiling_optimization() {
    let egraph: EGraph<IKun, ()> = EGraph::new();

    // map(f, x)
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let f = egraph.add(IKun::Symbol("f".to_string()));
    let root = egraph.add(IKun::Map(f, x));

    let optimizer = UniversalOptimizer::new();

    // Create a cost model that specifically favors tiling (e.g., for large data sets)
    struct TilingFavoredModel;
    impl chomsky_cost::CostModel for TilingFavoredModel {
        fn cost(&self, enode: &IKun) -> chomsky_cost::Cost {
            match enode {
                IKun::TiledMap(32, _, _) => chomsky_cost::Cost {
                    latency: 1.0,
                    throughput: 100.0,
                    size: 2.0,
                    energy: 1.0,
                },
                _ => chomsky_cost::Cost {
                    latency: 10.0,
                    throughput: 1.0,
                    size: 1.0,
                    energy: 1.0,
                },
            }
        }
    }

    let optimized_tree = optimizer.optimize(&egraph, root, TilingFavoredModel);

    // Expecting TiledMap(32, f, x)
    assert!(matches!(optimized_tree, IKunTree::TiledMap(32, _, _)));
}

#[test]
fn test_auto_vectorization_optimization() {
    let egraph: EGraph<IKun, ()> = EGraph::new();

    // map(f, x)
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let f = egraph.add(IKun::Symbol("f".to_string()));
    let root = egraph.add(IKun::Map(f, x));

    let optimizer = UniversalOptimizer::new();

    // JS engine often benefits from vectorization (simulated via cost model)
    let cost_model = JsCostModel::new(true);

    let optimized_tree = optimizer.optimize(&egraph, root, cost_model);

    // Expecting VectorizedMap(8, f, x)
    assert!(matches!(optimized_tree, IKunTree::VectorizedMap(8, _, _)));
}
