use chomsky_cost::{CostModel, CpuCostModel, JsCostModel};
use chomsky_emit::{GaiaEmitter, JvmBackend, WasiBackend, X86Backend};
use chomsky_extract::{Backend, BackendArtifact, IKun, IKunTree};
use chomsky_types::ChomskyResult;
use std::sync::Arc;

#[cfg(feature = "jvm")]
pub struct GaiaJvmAdapter;

#[cfg(feature = "jvm")]
impl Backend for GaiaJvmAdapter {
    fn name(&self) -> &str {
        "gaia-jvm"
    }

    fn generate(&self, tree: &IKunTree) -> ChomskyResult<BackendArtifact> {
        let emitter = GaiaEmitter::new("jvm").with_backend("jvm", Arc::new(JvmBackend::default()));
        emitter.emit(tree)
    }

    fn get_model(&self) -> &dyn CostModel {
        &CpuCostModel
    }
}

pub struct GaiaX86Adapter;

impl Backend for GaiaX86Adapter {
    fn name(&self) -> &str {
        "gaia-x86_64"
    }

    fn generate(&self, tree: &IKunTree) -> ChomskyResult<BackendArtifact> {
        let emitter =
            GaiaEmitter::new("x86_64").with_backend("x86_64", Arc::new(X86Backend::default()));
        emitter.emit(tree)
    }

    fn get_model(&self) -> &dyn CostModel {
        &CpuCostModel
    }
}

pub struct GaiaJsAdapter;

impl Backend for GaiaJsAdapter {
    fn name(&self) -> &str {
        "gaia-js"
    }

    fn generate(&self, tree: &IKunTree) -> ChomskyResult<BackendArtifact> {
        let emitter = GaiaEmitter::new("js");
        emitter.emit(tree)
    }

    fn get_model(&self) -> &dyn CostModel {
        &JsCostModel { prefer_loop: true }
    }
}

pub struct GaiaWasiAdapter;

impl Backend for GaiaWasiAdapter {
    fn name(&self) -> &str {
        "gaia-wasi"
    }

    fn generate(&self, tree: &IKunTree) -> ChomskyResult<BackendArtifact> {
        let emitter = GaiaEmitter::new("wasm32-wasi")
            .with_backend("wasm32-wasi", Arc::new(WasiBackend::default()))
            .standalone();
        emitter.emit(tree)
    }

    fn get_model(&self) -> &dyn CostModel {
        &CpuCostModel
    }
}
