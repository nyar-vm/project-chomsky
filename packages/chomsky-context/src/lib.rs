#![warn(missing_docs)]

use chomsky_uir::IKun;
use chomsky_uir::egraph::{Analysis, EGraph, Id};

pub struct ContextInjector;

impl ContextInjector {
    /// 在意图图中注入 GPU 上下文
    pub fn inject_gpu<A: Analysis<IKun>>(egraph: &EGraph<IKun, A>, id: Id) -> Id {
        let gpu_ctx = egraph.add(IKun::GpuContext);
        egraph.add(IKun::WithContext(gpu_ctx, id))
    }

    /// 在意图图中注入 CPU 上下文
    pub fn inject_cpu<A: Analysis<IKun>>(egraph: &EGraph<IKun, A>, id: Id) -> Id {
        let cpu_ctx = egraph.add(IKun::CpuContext);
        egraph.add(IKun::WithContext(cpu_ctx, id))
    }

    /// 在意图图中注入 Async 上下文
    pub fn inject_async<A: Analysis<IKun>>(egraph: &EGraph<IKun, A>, id: Id) -> Id {
        let async_ctx = egraph.add(IKun::AsyncContext);
        egraph.add(IKun::WithContext(async_ctx, id))
    }

    /// 在意图图中注入 Spatial 上下文
    pub fn inject_spatial<A: Analysis<IKun>>(egraph: &EGraph<IKun, A>, id: Id) -> Id {
        let spatial_ctx = egraph.add(IKun::SpatialContext);
        egraph.add(IKun::WithContext(spatial_ctx, id))
    }

    /// 注入通用的上下文
    pub fn inject_context<A: Analysis<IKun>>(
        egraph: &EGraph<IKun, A>,
        id: Id,
        context: IKun,
    ) -> Id {
        let ctx_id = egraph.add(context);
        egraph.add(IKun::WithContext(ctx_id, id))
    }
}
