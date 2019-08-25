use chomsky_concretize::{ConcretizationAnalysis, ConcretizationData, Layout};
use chomsky_uir::{Analysis, EGraph, IKun};

#[test]
fn test_concretize_analysis_basic() {
    let egraph = EGraph::<IKun, ConcretizationAnalysis>::new();
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let data = egraph.get_class(x).data.clone();

    assert_eq!(data.layout, Layout::Unknown);
    assert_eq!(data.tiling_factor, None);
}

#[test]
fn test_concretize_analysis_layout() {
    let egraph = EGraph::<IKun, ConcretizationAnalysis>::new();
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let soa_x = egraph.add(IKun::SoALayout(x));

    let data = egraph.get_class(soa_x).data.clone();
    assert_eq!(data.layout, Layout::SoA);
}

#[test]
fn test_concretize_analysis_tiling() {
    let egraph = EGraph::<IKun, ConcretizationAnalysis>::new();
    let x = egraph.add(IKun::Symbol("x".to_string()));
    let tiled_x = egraph.add(IKun::Tiled(16, x));

    let data = egraph.get_class(tiled_x).data.clone();
    assert_eq!(data.tiling_factor, Some(16));
}

#[test]
fn test_concretize_merge() {
    let mut d1 = ConcretizationData::default();
    let mut d2 = ConcretizationData::default();

    d2.layout = Layout::SoA;
    d2.vector_width = Some(4);

    let changed = d1.merge(&d2);

    assert!(changed);
    assert_eq!(d1.layout, Layout::SoA);
    assert_eq!(d1.vector_width, Some(4));

    // Merging again should return false
    assert!(!d1.merge(&d2));
}
