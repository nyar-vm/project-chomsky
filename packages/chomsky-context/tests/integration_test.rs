use chomsky_context::ContextInjector;
use chomsky_uir::{Analysis, EGraph, IKun};

#[test]
fn test_context_injector_gpu() {
    let egraph = EGraph::<IKun, ()>::new();
    let x = egraph.add(IKun::Symbol("x".to_string()));

    let with_ctx = ContextInjector::inject_gpu(&egraph, x);

    // Check if it's WithContext(GpuContext, x)
    let class = egraph.get_class(with_ctx);
    let node = &class.nodes[0];
    if let IKun::WithContext(ctx, body) = node {
        assert_eq!(*body, x);
        let ctx_class = egraph.get_class(*ctx);
        if let IKun::GpuContext = &ctx_class.nodes[0] {
            // OK
        } else {
            panic!("Expected GpuContext");
        }
    } else {
        panic!("Expected WithContext");
    }
}

#[test]
fn test_context_injector_async() {
    let egraph = EGraph::<IKun, ()>::new();
    let x = egraph.add(IKun::Symbol("x".to_string()));

    let with_ctx = ContextInjector::inject_async(&egraph, x);

    // Check if it's WithContext(AsyncContext, x)
    let class = egraph.get_class(with_ctx);
    let node = &class.nodes[0];
    if let IKun::WithContext(ctx, body) = node {
        assert_eq!(*body, x);
        let ctx_class = egraph.get_class(*ctx);
        if let IKun::AsyncContext = &ctx_class.nodes[0] {
            // OK
        } else {
            panic!("Expected AsyncContext");
        }
    } else {
        panic!("Expected WithContext");
    }
}
