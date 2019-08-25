use crate::egraph::Language;
use crate::union_find::Id;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IKun {
    // --- Data Atoms ---
    Constant(i64),
    FloatConstant(u64), // Use bits for Eq/Hash/Ord
    BooleanConstant(bool),
    StringConstant(String),
    // --- Module and Symbols ---
    Symbol(String),
    /// Import a symbol from another module: (ModuleName, SymbolName)
    Import(String, String),
    /// Export a symbol: (SymbolName, Body)
    Export(String, Id),
    /// A module container: (ModuleName, Items)
    Module(String, Vec<Id>),

    // --- Basic Intents ---
    Map(Id, Id),
    Filter(Id, Id),
    Reduce(Id, Id, Id),
    StateUpdate(Id, Id),
    Choice(Id, Id, Id),
    Repeat(Id, Id),
    /// LifeCycle: (setup, cleanup)
    LifeCycle(Id, Id),
    /// Meta: (body) for compile-time execution
    Meta(Id),
    /// Trap: (body) for error handling/interruption
    Trap(Id),
    /// Return: (value) for returning from a function
    Return(Id),

    // --- Composition ---
    Seq(Vec<Id>),
    Compose(Id, Id),

    // --- Context and Constraints ---
    WithContext(Id, Id),
    WithConstraint(Id, Id),

    // --- Context Atoms ---
    CpuContext,
    GpuContext,
    AsyncContext,
    SpatialContext,
    ComptimeContext,
    ResourceContext,
    SafeContext,

    // --- Constraint Atoms ---
    EffectConstraint(crate::constraint::Effect),
    OwnershipConstraint(crate::constraint::Ownership),
    TypeConstraint(String),
    AtomicConstraint,

    // --- Extension Point ---
    Extension(String, Vec<Id>),

    // --- Cross Language ---
    /// Cross-language call
    CrossLangCall(CrossLanguageCall),

    // --- Concretization Layer (Phase 5) ---
    SoALayout(Id),
    AoSLayout(Id),
    Tiled(usize, Id),
    Unrolled(usize, Id),
    Vectorized(usize, Id),

    // --- Optimized Specialized Intents (Avoid E-Graph cycles) ---
    /// Tiled version of a Map: (size, f, x)
    TiledMap(usize, Id, Id),
    /// Vectorized version of a Map: (width, f, x)
    VectorizedMap(usize, Id, Id),
    /// Unrolled version of a Map: (factor, f, x)
    UnrolledMap(usize, Id, Id),
    /// SoA Layout applied to a Map: (f, x)
    SoAMap(Id, Id),

    // 特化的扩展，方便在规则中使用
    GpuMap(Id, Id),
    CpuMap(Id, Id),

    // --- Spatial Layer (Phase 7) ---
    /// Declare a pipeline stage: (body, metadata)
    Pipe(Id, Id),
    /// Explicit register: (value)
    Reg(Id),

    // --- Function and Closure (New) ---
    /// Lambda definition: (params, body)
    Lambda(Vec<String>, Id),
    /// Function application: (func, args)
    Apply(Id, Vec<Id>),
    /// Closure capture: (body, captured_vars)
    Closure(Id, Vec<Id>),

    // --- Resource Management ---
    /// Explicit Reference Count Clone: (target)
    ResourceClone(Id),
    /// Explicit Reference Count Drop: (target)
    ResourceDrop(Id),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CrossLanguageCall {
    /// The target language/platform
    pub language: String,
    /// Full name of module/libiary
    pub module_path: String,
    /// Unique function name
    pub function_name: String,
    /// The fixed arguments
    pub arguments: Vec<Id>,
}

impl Language for IKun {
    fn children(&self) -> Vec<Id> {
        match self {
            IKun::Constant(_)
            | IKun::FloatConstant(_)
            | IKun::BooleanConstant(_)
            | IKun::StringConstant(_)
            | IKun::Symbol(_)
            | IKun::CpuContext
            | IKun::GpuContext
            | IKun::AsyncContext
            | IKun::SpatialContext
            | IKun::ComptimeContext
            | IKun::ResourceContext
            | IKun::SafeContext
            | IKun::EffectConstraint(_)
            | IKun::OwnershipConstraint(_)
            | IKun::TypeConstraint(_)
            | IKun::AtomicConstraint
            | IKun::Import(_, _) => vec![],

            IKun::Map(a, b)
            | IKun::Filter(a, b)
            | IKun::StateUpdate(a, b)
            | IKun::Repeat(a, b)
            | IKun::LifeCycle(a, b)
            | IKun::Compose(a, b)
            | IKun::WithContext(a, b)
            | IKun::WithConstraint(a, b)
            | IKun::GpuMap(a, b)
            | IKun::CpuMap(a, b)
            | IKun::Pipe(a, b)
            | IKun::SoAMap(a, b)
            | IKun::TiledMap(_, a, b)
            | IKun::VectorizedMap(_, a, b)
            | IKun::UnrolledMap(_, a, b) => vec![*a, *b],

            IKun::Reduce(a, b, c) | IKun::Choice(a, b, c) => vec![*a, *b, *c],

            IKun::Meta(a)
            | IKun::Trap(a)
            | IKun::Return(a)
            | IKun::Reg(a)
            | IKun::SoALayout(a)
            | IKun::AoSLayout(a)
            | IKun::Tiled(_, a)
            | IKun::Unrolled(_, a)
            | IKun::Vectorized(_, a)
            | IKun::Lambda(_, a)
            | IKun::ResourceClone(a)
            | IKun::ResourceDrop(a)
            | IKun::Export(_, a) => vec![*a],

            IKun::Seq(ids) | IKun::Extension(_, ids) | IKun::Module(_, ids) => ids.clone(),

            IKun::CrossLangCall(call) => call.arguments.clone(),

            IKun::Apply(a, ids) | IKun::Closure(a, ids) => {
                let mut res = vec![*a];
                res.extend(ids.iter().cloned());
                res
            }
        }
    }

    fn map_children(&self, mut f: impl FnMut(Id) -> Id) -> Self {
        match self {
            IKun::Constant(v) => IKun::Constant(*v),
            IKun::FloatConstant(v) => IKun::FloatConstant(*v),
            IKun::BooleanConstant(v) => IKun::BooleanConstant(*v),
            IKun::StringConstant(s) => IKun::StringConstant(s.clone()),
            IKun::Symbol(s) => IKun::Symbol(s.clone()),
            IKun::Map(a, b) => IKun::Map(f(*a), f(*b)),
            IKun::Filter(a, b) => IKun::Filter(f(*a), f(*b)),
            IKun::Reduce(a, b, c) => IKun::Reduce(f(*a), f(*b), f(*c)),
            IKun::StateUpdate(a, b) => IKun::StateUpdate(f(*a), f(*b)),
            IKun::Choice(a, b, c) => IKun::Choice(f(*a), f(*b), f(*c)),
            IKun::Repeat(a, b) => IKun::Repeat(f(*a), f(*b)),
            IKun::LifeCycle(a, b) => IKun::LifeCycle(f(*a), f(*b)),
            IKun::Meta(a) => IKun::Meta(f(*a)),
            IKun::Trap(a) => IKun::Trap(f(*a)),
            IKun::Return(a) => IKun::Return(f(*a)),
            IKun::Seq(ids) => IKun::Seq(ids.iter().map(|id| f(*id)).collect()),
            IKun::Compose(a, b) => IKun::Compose(f(*a), f(*b)),
            IKun::WithContext(a, b) => IKun::WithContext(f(*a), f(*b)),
            IKun::WithConstraint(a, b) => IKun::WithConstraint(f(*a), f(*b)),
            IKun::CpuContext => IKun::CpuContext,
            IKun::GpuContext => IKun::GpuContext,
            IKun::AsyncContext => IKun::AsyncContext,
            IKun::SpatialContext => IKun::SpatialContext,
            IKun::ComptimeContext => IKun::ComptimeContext,
            IKun::ResourceContext => IKun::ResourceContext,
            IKun::SafeContext => IKun::SafeContext,
            IKun::EffectConstraint(e) => IKun::EffectConstraint(e.clone()),
            IKun::OwnershipConstraint(o) => IKun::OwnershipConstraint(o.clone()),
            IKun::TypeConstraint(t) => IKun::TypeConstraint(t.clone()),
            IKun::AtomicConstraint => IKun::AtomicConstraint,
            IKun::Extension(name, ids) => {
                IKun::Extension(name.clone(), ids.iter().map(|id| f(*id)).collect())
            }
            IKun::CrossLangCall(call) => IKun::CrossLangCall(CrossLanguageCall {
                language: call.language.clone(),
                module_path: call.module_path.clone(),
                function_name: call.function_name.clone(),
                arguments: call.arguments.iter().map(|id| f(*id)).collect(),
            }),
            IKun::SoALayout(a) => IKun::SoALayout(f(*a)),
            IKun::AoSLayout(a) => IKun::AoSLayout(f(*a)),
            IKun::Tiled(s, a) => IKun::Tiled(*s, f(*a)),
            IKun::Unrolled(s, a) => IKun::Unrolled(*s, f(*a)),
            IKun::Vectorized(s, a) => IKun::Vectorized(*s, f(*a)),
            IKun::TiledMap(s, a, b) => IKun::TiledMap(*s, f(*a), f(*b)),
            IKun::VectorizedMap(s, a, b) => IKun::VectorizedMap(*s, f(*a), f(*b)),
            IKun::UnrolledMap(s, a, b) => IKun::UnrolledMap(*s, f(*a), f(*b)),
            IKun::SoAMap(a, b) => IKun::SoAMap(f(*a), f(*b)),
            IKun::GpuMap(a, b) => IKun::GpuMap(f(*a), f(*b)),
            IKun::CpuMap(a, b) => IKun::CpuMap(f(*a), f(*b)),
            IKun::Pipe(a, b) => IKun::Pipe(f(*a), f(*b)),
            IKun::Reg(a) => IKun::Reg(f(*a)),
            IKun::Lambda(params, body) => IKun::Lambda(params.clone(), f(*body)),
            IKun::Apply(func, args) => {
                IKun::Apply(f(*func), args.iter().map(|id| f(*id)).collect())
            }
            IKun::Closure(body, captured) => {
                IKun::Closure(f(*body), captured.iter().map(|id| f(*id)).collect())
            }
            IKun::ResourceClone(a) => IKun::ResourceClone(f(*a)),
            IKun::ResourceDrop(a) => IKun::ResourceDrop(f(*a)),
            IKun::Import(m, s) => IKun::Import(m.clone(), s.clone()),
            IKun::Export(s, b) => IKun::Export(s.clone(), f(*b)),
            IKun::Module(m, ids) => IKun::Module(m.clone(), ids.iter().map(|id| f(*id)).collect()),
        }
    }
}

impl fmt::Display for IKun {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IKun::Constant(v) => write!(f, "{}", v),
            IKun::FloatConstant(v) => write!(f, "{}", f64::from_bits(*v)),
            IKun::BooleanConstant(v) => write!(f, "{}", v),
            IKun::StringConstant(s) => write!(f, "\"{}\"", s),
            IKun::Symbol(s) => write!(f, "{}", s),
            IKun::Map(_, _) => write!(f, "map"),
            IKun::Filter(_, _) => write!(f, "filter"),
            IKun::Reduce(_, _, _) => write!(f, "reduce"),
            IKun::StateUpdate(_, _) => write!(f, "state-update"),
            IKun::Choice(_, _, _) => write!(f, "choice"),
            IKun::Repeat(_, _) => write!(f, "repeat"),
            IKun::Seq(_) => write!(f, "seq"),
            IKun::Compose(_, _) => write!(f, "compose"),
            IKun::WithContext(_, _) => write!(f, "with-context"),
            IKun::WithConstraint(_, _) => write!(f, "with-constraint"),
            IKun::CpuContext => write!(f, "cpu-context"),
            IKun::GpuContext => write!(f, "gpu-context"),
            IKun::AsyncContext => write!(f, "async-context"),
            IKun::SpatialContext => write!(f, "spatial-context"),
            IKun::ComptimeContext => write!(f, "comptime-context"),
            IKun::ResourceContext => write!(f, "res-context"),
            IKun::SafeContext => write!(f, "safe-context"),
            IKun::LifeCycle(_, _) => write!(f, "lifecycle"),
            IKun::Meta(_) => write!(f, "meta"),
            IKun::Trap(_) => write!(f, "trap"),
            IKun::Return(_) => write!(f, "return"),
            IKun::EffectConstraint(e) => write!(f, "effect:{:?}", e),
            IKun::OwnershipConstraint(o) => write!(f, "ownership:{:?}", o),
            IKun::TypeConstraint(t) => write!(f, "type:{}", t),
            IKun::AtomicConstraint => write!(f, "atomic-constraint"),
            IKun::Extension(name, _) => write!(f, "ext:{}", name),
            IKun::CrossLangCall(call) => {
                write!(
                    f,
                    "cross-call:{}:{}:{}",
                    call.language, call.module_path, call.function_name
                )
            }
            IKun::GpuMap(_, _) => write!(f, "gpu-map"),
            IKun::CpuMap(_, _) => write!(f, "cpu-map"),
            IKun::SoALayout(_) => write!(f, "soa"),
            IKun::AoSLayout(_) => write!(f, "aos"),
            IKun::Tiled(s, _) => write!(f, "tiled:{}", s),
            IKun::Unrolled(s, _) => write!(f, "unrolled:{}", s),
            IKun::Vectorized(s, _) => write!(f, "vectorized:{}", s),
            IKun::SoAMap(_, _) => write!(f, "soa-map"),
            IKun::TiledMap(s, _, _) => write!(f, "tiled-map:{}", s),
            IKun::VectorizedMap(s, _, _) => write!(f, "vectorized-map:{}", s),
            IKun::UnrolledMap(s, _, _) => write!(f, "unrolled-map:{}", s),
            IKun::Pipe(_, _) => write!(f, "pipe"),
            IKun::Reg(_) => write!(f, "reg"),
            IKun::Lambda(params, _) => write!(f, "lambda({})", params.join(", ")),
            IKun::Apply(_, _) => write!(f, "apply"),
            IKun::Closure(_, _) => write!(f, "closure"),
            IKun::ResourceClone(_) => write!(f, "res-clone"),
            IKun::ResourceDrop(_) => write!(f, "res-drop"),
            IKun::Import(m, s) => write!(f, "import:{}:{}", m, s),
            IKun::Export(s, _) => write!(f, "export:{}", s),
            IKun::Module(m, _) => write!(f, "module:{}", m),
        }
    }
}

pub type Intent = IKun;
pub type IntentOp = IKun;

/// Recursive tree structure for Backends
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IKunTree {
    Constant(i64),
    FloatConstant(u64), // Use bits for Eq/Hash
    BooleanConstant(bool),
    StringConstant(String),
    Symbol(String),
    Map(Box<IKunTree>, Box<IKunTree>),
    Filter(Box<IKunTree>, Box<IKunTree>),
    Reduce(Box<IKunTree>, Box<IKunTree>, Box<IKunTree>),
    StateUpdate(Box<IKunTree>, Box<IKunTree>),
    Choice(Box<IKunTree>, Box<IKunTree>, Box<IKunTree>),
    Repeat(Box<IKunTree>, Box<IKunTree>),
    LifeCycle(Box<IKunTree>, Box<IKunTree>),
    Meta(Box<IKunTree>),
    Trap(Box<IKunTree>),
    Return(Box<IKunTree>),
    Seq(Vec<IKunTree>),
    Compose(Box<IKunTree>, Box<IKunTree>),
    WithContext(Box<IKunTree>, Box<IKunTree>),
    WithConstraint(Box<IKunTree>, Box<IKunTree>),
    CpuContext,
    GpuContext,
    AsyncContext,
    SpatialContext,
    ComptimeContext,
    ResourceContext,
    SafeContext,
    EffectConstraint(crate::constraint::Effect),
    OwnershipConstraint(crate::constraint::Ownership),
    TypeConstraint(String),
    AtomicConstraint,
    Extension(String, Vec<IKunTree>),
    CrossLangCall {
        language: String,
        module_path: String,
        function_name: String,
        arguments: Vec<IKunTree>,
    },
    GpuMap(Box<IKunTree>, Box<IKunTree>),
    CpuMap(Box<IKunTree>, Box<IKunTree>),
    TiledMap(usize, Box<IKunTree>, Box<IKunTree>),
    VectorizedMap(usize, Box<IKunTree>, Box<IKunTree>),
    UnrolledMap(usize, Box<IKunTree>, Box<IKunTree>),
    SoAMap(Box<IKunTree>, Box<IKunTree>),

    // Phase 5: Concretization
    SoALayout(Box<IKunTree>),
    AoSLayout(Box<IKunTree>),
    Tiled(usize, Box<IKunTree>),
    Unrolled(usize, Box<IKunTree>),
    Vectorized(usize, Box<IKunTree>),

    // Phase 7: Spatial
    Pipe(Box<IKunTree>, Box<IKunTree>),
    Reg(Box<IKunTree>),

    // --- Function and Closure (New) ---
    Lambda(Vec<String>, Box<IKunTree>),
    Apply(Box<IKunTree>, Vec<IKunTree>),
    Closure(Box<IKunTree>, Vec<IKunTree>),

    ResourceClone(Box<IKunTree>),
    ResourceDrop(Box<IKunTree>),

    Import(String, String),
    Export(String, Box<IKunTree>),
    Module(String, Vec<IKunTree>),

    /// Source location information for debugging
    Source(chomsky_types::Loc, Box<IKunTree>),
}

impl Default for IKunTree {
    fn default() -> Self {
        IKunTree::Module("default".to_string(), Vec::new())
    }
}

impl IKunTree {
    pub fn to_egraph<A: crate::egraph::Analysis<IKun>>(
        &self,
        egraph: &mut crate::egraph::EGraph<IKun, A>,
    ) -> Id {
        match self {
            IKunTree::Constant(v) => egraph.add(IKun::Constant(*v)),
            IKunTree::FloatConstant(v) => egraph.add(IKun::FloatConstant(*v)),
            IKunTree::BooleanConstant(v) => egraph.add(IKun::BooleanConstant(*v)),
            IKunTree::StringConstant(s) => egraph.add(IKun::StringConstant(s.clone())),
            IKunTree::Symbol(s) => egraph.add(IKun::Symbol(s.clone())),
            IKunTree::Map(f, x) => {
                let f_id = f.to_egraph(egraph);
                let x_id = x.to_egraph(egraph);
                egraph.add(IKun::Map(f_id, x_id))
            }
            IKunTree::Filter(f, x) => {
                let f_id = f.to_egraph(egraph);
                let x_id = x.to_egraph(egraph);
                egraph.add(IKun::Filter(f_id, x_id))
            }
            IKunTree::Reduce(f, init, list) => {
                let f_id = f.to_egraph(egraph);
                let init_id = init.to_egraph(egraph);
                let list_id = list.to_egraph(egraph);
                egraph.add(IKun::Reduce(f_id, init_id, list_id))
            }
            IKunTree::StateUpdate(var, val) => {
                let var_id = var.to_egraph(egraph);
                let val_id = val.to_egraph(egraph);
                egraph.add(IKun::StateUpdate(var_id, val_id))
            }
            IKunTree::Choice(cond, t, f) => {
                let cond_id = cond.to_egraph(egraph);
                let t_id = t.to_egraph(egraph);
                let f_id = f.to_egraph(egraph);
                egraph.add(IKun::Choice(cond_id, t_id, f_id))
            }
            IKunTree::Repeat(count, body) => {
                let count_id = count.to_egraph(egraph);
                let body_id = body.to_egraph(egraph);
                egraph.add(IKun::Repeat(count_id, body_id))
            }
            IKunTree::LifeCycle(setup, cleanup) => {
                let setup_id = setup.to_egraph(egraph);
                let cleanup_id = cleanup.to_egraph(egraph);
                egraph.add(IKun::LifeCycle(setup_id, cleanup_id))
            }
            IKunTree::Meta(body) => {
                let body_id = body.to_egraph(egraph);
                egraph.add(IKun::Meta(body_id))
            }
            IKunTree::Trap(body) => {
                let body_id = body.to_egraph(egraph);
                egraph.add(IKun::Trap(body_id))
            }
            IKunTree::Return(val) => {
                let val_id = val.to_egraph(egraph);
                egraph.add(IKun::Return(val_id))
            }
            IKunTree::Seq(trees) => {
                let ids = trees.iter().map(|t| t.to_egraph(egraph)).collect();
                egraph.add(IKun::Seq(ids))
            }
            IKunTree::Compose(f, g) => {
                let f_id = f.to_egraph(egraph);
                let g_id = g.to_egraph(egraph);
                egraph.add(IKun::Compose(f_id, g_id))
            }
            IKunTree::WithContext(ctx, body) => {
                let ctx_id = ctx.to_egraph(egraph);
                let body_id = body.to_egraph(egraph);
                egraph.add(IKun::WithContext(ctx_id, body_id))
            }
            IKunTree::WithConstraint(c, body) => {
                let c_id = c.to_egraph(egraph);
                let body_id = body.to_egraph(egraph);
                egraph.add(IKun::WithConstraint(c_id, body_id))
            }
            IKunTree::CpuContext => egraph.add(IKun::CpuContext),
            IKunTree::GpuContext => egraph.add(IKun::GpuContext),
            IKunTree::AsyncContext => egraph.add(IKun::AsyncContext),
            IKunTree::SpatialContext => egraph.add(IKun::SpatialContext),
            IKunTree::ComptimeContext => egraph.add(IKun::ComptimeContext),
            IKunTree::ResourceContext => egraph.add(IKun::ResourceContext),
            IKunTree::SafeContext => egraph.add(IKun::SafeContext),
            IKunTree::EffectConstraint(e) => egraph.add(IKun::EffectConstraint(e.clone())),
            IKunTree::OwnershipConstraint(o) => egraph.add(IKun::OwnershipConstraint(o.clone())),
            IKunTree::TypeConstraint(t) => egraph.add(IKun::TypeConstraint(t.clone())),
            IKunTree::AtomicConstraint => egraph.add(IKun::AtomicConstraint),
            IKunTree::Extension(name, trees) => {
                let ids = trees.iter().map(|t| t.to_egraph(egraph)).collect();
                egraph.add(IKun::Extension(name.clone(), ids))
            }
            IKunTree::Source(_, body) => body.to_egraph(egraph),
            IKunTree::GpuMap(f, x) => {
                let f_id = f.to_egraph(egraph);
                let x_id = x.to_egraph(egraph);
                egraph.add(IKun::GpuMap(f_id, x_id))
            }
            IKunTree::CpuMap(f, x) => {
                let f_id = f.to_egraph(egraph);
                let x_id = x.to_egraph(egraph);
                egraph.add(IKun::CpuMap(f_id, x_id))
            }
            IKunTree::TiledMap(s, f, x) => {
                let f_id = f.to_egraph(egraph);
                let x_id = x.to_egraph(egraph);
                egraph.add(IKun::TiledMap(*s, f_id, x_id))
            }
            IKunTree::VectorizedMap(s, f, x) => {
                let f_id = f.to_egraph(egraph);
                let x_id = x.to_egraph(egraph);
                egraph.add(IKun::VectorizedMap(*s, f_id, x_id))
            }
            IKunTree::UnrolledMap(s, f, x) => {
                let f_id = f.to_egraph(egraph);
                let x_id = x.to_egraph(egraph);
                egraph.add(IKun::UnrolledMap(*s, f_id, x_id))
            }
            IKunTree::SoAMap(f, x) => {
                let f_id = f.to_egraph(egraph);
                let x_id = x.to_egraph(egraph);
                egraph.add(IKun::SoAMap(f_id, x_id))
            }
            IKunTree::SoALayout(a) => {
                let a_id = a.to_egraph(egraph);
                egraph.add(IKun::SoALayout(a_id))
            }
            IKunTree::AoSLayout(a) => {
                let a_id = a.to_egraph(egraph);
                egraph.add(IKun::AoSLayout(a_id))
            }
            IKunTree::Tiled(s, a) => {
                let a_id = a.to_egraph(egraph);
                egraph.add(IKun::Tiled(*s, a_id))
            }
            IKunTree::Unrolled(s, a) => {
                let a_id = a.to_egraph(egraph);
                egraph.add(IKun::Unrolled(*s, a_id))
            }
            IKunTree::Vectorized(s, a) => {
                let a_id = a.to_egraph(egraph);
                egraph.add(IKun::Vectorized(*s, a_id))
            }
            IKunTree::Pipe(a, b) => {
                let a_id = a.to_egraph(egraph);
                let b_id = b.to_egraph(egraph);
                egraph.add(IKun::Pipe(a_id, b_id))
            }
            IKunTree::Reg(a) => {
                let a_id = a.to_egraph(egraph);
                egraph.add(IKun::Reg(a_id))
            }
            IKunTree::Lambda(params, body) => {
                let body_id = body.to_egraph(egraph);
                egraph.add(IKun::Lambda(params.clone(), body_id))
            }
            IKunTree::Apply(func, args) => {
                let func_id = func.to_egraph(egraph);
                let arg_ids = args.iter().map(|a| a.to_egraph(egraph)).collect();
                egraph.add(IKun::Apply(func_id, arg_ids))
            }
            IKunTree::Closure(body, captured) => {
                let body_id = body.to_egraph(egraph);
                let captured_ids = captured.iter().map(|c| c.to_egraph(egraph)).collect();
                egraph.add(IKun::Closure(body_id, captured_ids))
            }
            IKunTree::ResourceClone(target) => {
                let target_id = target.to_egraph(egraph);
                egraph.add(IKun::ResourceClone(target_id))
            }
            IKunTree::ResourceDrop(target) => {
                let target_id = target.to_egraph(egraph);
                egraph.add(IKun::ResourceDrop(target_id))
            }
            IKunTree::CrossLangCall {
                language,
                module_path,
                function_name,
                arguments,
            } => {
                let args_ids = arguments.iter().map(|arg| arg.to_egraph(egraph)).collect();
                egraph.add(IKun::CrossLangCall(CrossLanguageCall {
                    language: language.clone(),
                    module_path: module_path.clone(),
                    function_name: function_name.clone(),
                    arguments: args_ids,
                }))
            }
            IKunTree::Import(m, s) => egraph.add(IKun::Import(m.clone(), s.clone())),
            IKunTree::Export(s, b) => {
                let b_id = b.to_egraph(egraph);
                egraph.add(IKun::Export(s.clone(), b_id))
            }
            IKunTree::Module(m, trees) => {
                let ids = trees.iter().map(|t| t.to_egraph(egraph)).collect();
                egraph.add(IKun::Module(m.clone(), ids))
            }
        }
    }
}
