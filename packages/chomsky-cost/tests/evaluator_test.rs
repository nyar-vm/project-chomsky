use chomsky_cost::{Backend, CostEvaluator};
use chomsky_uir::IKun;

#[test]
fn test_cost_evaluator() {
    let evaluator = CostEvaluator::new();

    // Constant should be cheap on all backends
    let constant = IKun::Constant(42);
    let (best_backend, _) = evaluator.best_backend(&constant);
    // On our current model, constants are cheap everywhere, but let's see
    println!("Best backend for constant: {:?}", best_backend);

    // GpuMap should be best on GPU
    let f = chomsky_uir::Id::from(0);
    let x = chomsky_uir::Id::from(1);
    let gpu_map = IKun::GpuMap(f, x);
    let (best_gpu, _) = evaluator.best_backend(&gpu_map);
    assert_eq!(best_gpu, Backend::Gpu);

    // CpuMap should be best on CPU
    let cpu_map = IKun::CpuMap(f, x);
    let (best_cpu, _) = evaluator.best_backend(&cpu_map);
    assert_eq!(best_cpu, Backend::Cpu);
}
