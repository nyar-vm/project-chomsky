use chomsky_cost::{Cost, CostModel, JsCostModel};
use chomsky_uir::{IKun, Id};

#[test]
fn test_cost_default() {
    let c = Cost::default();
    assert_eq!(c.latency, 0.0);
    assert_eq!(c.throughput, 1.0);
}

#[test]
fn test_cost_infinite() {
    let c = Cost::infinite();
    assert!(c.latency.is_infinite());
    assert_eq!(c.score(), f64::INFINITY);
}

#[test]
fn test_cost_add() {
    let c1 = Cost {
        latency: 10.0,
        throughput: 2.0,
        size: 5.0,
        energy: 1.0,
    };
    let c2 = Cost {
        latency: 5.0,
        throughput: 4.0,
        size: 2.0,
        energy: 2.0,
    };
    let sum = c1.add(&c2);

    assert_eq!(sum.latency, 15.0);
    assert_eq!(sum.throughput, 2.0); // min throughput
    assert_eq!(sum.size, 7.0);
    assert_eq!(sum.energy, 3.0);
}

#[test]
fn test_cost_score() {
    let c = Cost {
        latency: 10.0,
        throughput: 2.0,
        size: 5.0,
        energy: 1.0,
    };
    // score = latency/throughput + size*0.5 + energy*0.2
    // 10/2 + 5*0.5 + 1*0.2 = 5 + 2.5 + 0.2 = 7.7
    assert!((c.score() - 7.7).abs() < 1e-6);
}

#[test]
fn test_js_cost_model() {
    let model = JsCostModel::new(true);
    let map_node = IKun::Map(Id::from(0usize), Id::from(1usize));
    let cost = model.cost(&map_node);

    assert_eq!(cost.latency, 5.0);
}
