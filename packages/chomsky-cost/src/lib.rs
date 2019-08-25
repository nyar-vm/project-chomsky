#![warn(missing_docs)]

use chomsky_uir::IKun;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Cost {
    pub latency: f64,
    pub throughput: f64,
    pub size: f64,
    pub energy: f64,
}

impl Default for Cost {
    fn default() -> Self {
        Cost {
            latency: 0.0,
            throughput: 1.0,
            size: 0.0,
            energy: 0.0,
        }
    }
}

impl Cost {
    pub fn infinite() -> Self {
        Cost {
            latency: f64::INFINITY,
            throughput: 0.0,
            size: f64::INFINITY,
            energy: f64::INFINITY,
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        Cost {
            latency: self.latency + other.latency,
            throughput: self.throughput.min(other.throughput),
            size: self.size + other.size,
            energy: self.energy + other.energy,
        }
    }

    pub fn score(&self) -> f64 {
        if self.latency == f64::INFINITY
            || self.size == f64::INFINITY
            || self.energy == f64::INFINITY
            || self.throughput <= 0.0
        {
            return f64::INFINITY;
        }
        // 改进的标量化成本：(latency / throughput) + size + energy
        (self.latency / self.throughput) + self.size * 0.5 + self.energy * 0.2
    }

    pub fn weighted_score(&self, latency_weight: f64, size_weight: f64, energy_weight: f64) -> f64 {
        if self.latency == f64::INFINITY
            || self.size == f64::INFINITY
            || self.energy == f64::INFINITY
        {
            return f64::INFINITY;
        }
        self.latency * latency_weight + self.size * size_weight + self.energy * energy_weight
    }
}

pub trait CostModel {
    fn cost(&self, enode: &IKun) -> Cost;

    /// 获取该模型的默认权重
    fn weights(&self) -> (f64, f64, f64) {
        (1.0, 0.5, 0.2)
    }
}

impl CostModel for &dyn CostModel {
    fn cost(&self, enode: &IKun) -> Cost {
        (**self).cost(enode)
    }
    fn weights(&self) -> (f64, f64, f64) {
        (**self).weights()
    }
}

impl CostModel for Box<dyn CostModel> {
    fn cost(&self, enode: &IKun) -> Cost {
        (**self).cost(enode)
    }
    fn weights(&self) -> (f64, f64, f64) {
        (**self).weights()
    }
}

#[derive(Debug, Clone, Default)]
pub struct DefaultCostModel;

impl CostModel for DefaultCostModel {
    fn cost(&self, _enode: &IKun) -> Cost {
        Cost::default()
    }
}

pub static DEFAULT_COST_MODEL: DefaultCostModel = DefaultCostModel;

#[derive(Debug, Clone)]
pub struct JsCostModel {
    pub prefer_loop: bool,
}

impl JsCostModel {
    pub fn new(prefer_loop: bool) -> Self {
        Self { prefer_loop }
    }
}

impl CostModel for JsCostModel {
    fn cost(&self, enode: &IKun) -> Cost {
        match enode {
            IKun::Map(_, _) | IKun::Filter(_, _) => Cost {
                latency: 5.0,
                throughput: 1.0,
                size: 1.0,
                energy: 1.0,
            },

            IKun::SoAMap(_, _) => Cost {
                latency: 1.0,
                throughput: 10.0,
                size: 1.0,
                energy: 0.5,
            },

            IKun::TiledMap(_, _, _) => Cost {
                latency: 1.5,
                throughput: 20.0,
                size: 2.0,
                energy: 1.0,
            },

            IKun::VectorizedMap(_, _, _) => Cost {
                latency: 0.8,
                throughput: 40.0,
                size: 1.5,
                energy: 0.8,
            },

            IKun::Return(_) => Cost {
                latency: 1.0,
                throughput: 1.0,
                size: 1.0,
                energy: 0.1,
            },

            IKun::Reduce(_, _, _) => Cost {
                latency: 6.0,
                throughput: 1.0,
                size: 1.0,
                energy: 1.2,
            },

            IKun::Extension(name, _) => match name.as_str() {
                "loop_map" | "loop_filter" | "loop_reduce" | "loop_map_reduce" => {
                    if self.prefer_loop {
                        Cost {
                            latency: 2.0,
                            throughput: 10.0,
                            size: 2.0,
                            energy: 1.0,
                        }
                    } else {
                        Cost {
                            latency: 5.0,
                            throughput: 2.0,
                            size: 10.0,
                            energy: 2.0,
                        }
                    }
                }
                "and_predicate" => Cost {
                    latency: 0.5,
                    throughput: 1.0,
                    size: 1.0,
                    energy: 0.5,
                },
                "filter_map" => Cost {
                    latency: 1.0,
                    throughput: 5.0,
                    size: 2.0,
                    energy: 1.0,
                },
                "add" | "sub" => Cost {
                    latency: 0.6,
                    throughput: 2.0,
                    size: 1.0,
                    energy: 0.2,
                },
                "shl" | "shr" => Cost {
                    latency: 0.4,
                    throughput: 2.0,
                    size: 1.0,
                    energy: 0.1,
                },
                "mul" => Cost {
                    latency: 0.8,
                    throughput: 1.5,
                    size: 1.2,
                    energy: 0.3,
                },
                _ => Cost {
                    latency: 1.0,
                    throughput: 1.0,
                    size: 1.0,
                    energy: 1.0,
                },
            },

            IKun::Seq(ids) => Cost {
                latency: ids.len() as f64 * 0.1,
                throughput: 1.0,
                size: ids.len() as f64 * 0.1,
                energy: ids.len() as f64 * 0.1,
            },

            _ => Cost::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CpuCostModel;

impl CostModel for CpuCostModel {
    fn cost(&self, enode: &IKun) -> Cost {
        match enode {
            IKun::Constant(_) | IKun::FloatConstant(_) | IKun::BooleanConstant(_) => Cost {
                latency: 1.0,
                throughput: 4.0,
                size: 1.0,
                energy: 0.1,
            },
            IKun::Symbol(_) => Cost {
                latency: 1.0,
                throughput: 4.0,
                size: 0.0,
                energy: 0.0,
            },
            IKun::Return(_) => Cost {
                latency: 1.0,
                throughput: 4.0,
                size: 1.0,
                energy: 0.1,
            },
            IKun::Map(_, _) => Cost {
                latency: 10.0,
                throughput: 1.0,
                size: 5.0,
                energy: 2.0,
            },
            IKun::VectorizedMap(_, _, _) => Cost {
                latency: 2.0,
                throughput: 8.0,
                size: 10.0,
                energy: 1.0,
            },
            IKun::TiledMap(_, _, _) => Cost {
                latency: 5.0,
                throughput: 2.0,
                size: 8.0,
                energy: 1.5,
            },
            IKun::CpuMap(_, _) => Cost {
                latency: 1.0,
                throughput: 4.0,
                size: 1.0,
                energy: 0.5,
            },
            IKun::GpuMap(_, _) => Cost::infinite(), // CPU model cannot run GPU map
            _ => Cost {
                latency: 2.0,
                throughput: 1.0,
                size: 2.0,
                energy: 1.0,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct GpuCostModel;

impl CostModel for GpuCostModel {
    fn cost(&self, enode: &IKun) -> Cost {
        match enode {
            IKun::Constant(_) | IKun::FloatConstant(_) | IKun::BooleanConstant(_) => Cost {
                latency: 1.0,
                throughput: 32.0,
                size: 1.0,
                energy: 0.05,
            },
            IKun::GpuMap(_, _) => Cost {
                latency: 5.0,
                throughput: 100.0,
                size: 10.0,
                energy: 5.0,
            },
            IKun::CpuMap(_, _) => Cost::infinite(), // GPU model cannot run CPU map
            IKun::Map(_, _) => Cost {
                latency: 20.0,
                throughput: 0.5,
                size: 10.0,
                energy: 10.0,
            }, // Unoptimized map is expensive on GPU
            _ => Cost {
                latency: 10.0,
                throughput: 10.0,
                size: 5.0,
                energy: 2.0,
            },
        }
    }

    fn weights(&self) -> (f64, f64, f64) {
        (0.1, 1.0, 2.0) // On GPU, throughput is key, latency is less important
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Js,
    Cpu,
    Gpu,
}

pub struct CostEvaluator {
    pub js: JsCostModel,
    pub cpu: CpuCostModel,
    pub gpu: GpuCostModel,
}

impl Default for CostEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl CostEvaluator {
    pub fn new() -> Self {
        Self {
            js: JsCostModel::new(true),
            cpu: CpuCostModel,
            gpu: GpuCostModel,
        }
    }

    pub fn evaluate_all(&self, enode: &IKun) -> Vec<(Backend, Cost)> {
        vec![
            (Backend::Js, self.js.cost(enode)),
            (Backend::Cpu, self.cpu.cost(enode)),
            (Backend::Gpu, self.gpu.cost(enode)),
        ]
    }

    pub fn best_backend(&self, enode: &IKun) -> (Backend, Cost) {
        let costs = self.evaluate_all(enode);
        costs
            .into_iter()
            .min_by(|(_, a), (_, b)| {
                a.score()
                    .partial_cmp(&b.score())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap()
    }

    pub fn get_model(&self, backend: Backend) -> Box<dyn CostModel> {
        match backend {
            Backend::Js => Box::new(self.js.clone()),
            Backend::Cpu => Box::new(self.cpu.clone()),
            Backend::Gpu => Box::new(self.gpu.clone()),
        }
    }
}
