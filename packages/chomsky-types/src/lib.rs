#![warn(missing_docs)]

//! # Chomsky Types
//! 核心类型定义，包含意图 (Intent) 和上下文 (Context) 的形式化描述。

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// 紧凑的位置偏移信息
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn unknown() -> Self {
        Self { start: 0, end: 0 }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::unknown()
    }
}

/// 携带源码 ID 的定位信息
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Loc {
    pub source_id: u32,
    pub span: Span,
}

impl Loc {
    pub fn new(source_id: u32, start: u32, end: u32) -> Self {
        Self {
            source_id,
            span: Span::new(start, end),
        }
    }

    /// 约定 source_id 0 为匿名/位置未知
    pub fn unknown() -> Self {
        Self {
            source_id: 0,
            span: Span::unknown(),
        }
    }

    pub fn is_unknown(&self) -> bool {
        self.source_id == 0
    }
}

impl Default for Loc {
    fn default() -> Self {
        Self::unknown()
    }
}

/// 源码管理器的单个条目
pub struct SourceEntry {
    pub path: String,
    pub content: Arc<String>,
    pub line_map: LineMap,
}

/// 偏移量到行列号的映射工具
pub struct LineMap {
    line_starts: Vec<u32>,
}

impl LineMap {
    pub fn new(content: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, c) in content.char_indices() {
            if c == '\n' {
                line_starts.push((i + 1) as u32);
            }
        }
        Self { line_starts }
    }

    pub fn lookup(&self, offset: u32) -> (u32, u32) {
        let line = match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(line) => line - 1,
        };
        let col = offset - self.line_starts[line];
        (line as u32 + 1, col + 1)
    }
}

/// 全局源码管理器
pub struct SourceManager {
    sources: DashMap<u32, SourceEntry>,
    next_id: std::sync::atomic::AtomicUsize,
}

impl SourceManager {
    pub fn new() -> Self {
        Self {
            sources: DashMap::new(),
            next_id: std::sync::atomic::AtomicUsize::new(1),
        }
    }

    pub fn register(&self, path: &str, content: String) -> u32 {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst) as u32;
        let line_map = LineMap::new(&content);
        let entry = SourceEntry {
            path: path.to_string(),
            content: Arc::new(content),
            line_map,
        };
        self.sources.insert(id, entry);
        id
    }

    pub fn get(&self, id: u32) -> Option<dashmap::mapref::one::Ref<'_, u32, SourceEntry>> {
        self.sources.get(&id)
    }
}

impl Default for SourceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 基础计算意图 (Intent Atoms)
/// 对应乔姆斯基框架中的“普遍语法”核心。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    /// 映射与变换：将数据从一种形式转换为另一种形式 (map, filter, transduce)
    Map,
    /// 归纳与汇总：将集合聚合成单一值 (reduce, fold, summation)
    Reduce,
    /// 状态与交互：随时间改变并与外界交互 (state mutation, I/O)
    StateUpdate,
    /// 选择与分支：基于条件选择路径 (if/else, pattern matching)
    Branch,
    /// 重复与递归：以循环或递归方式重复操作
    Loop,
    /// 资源生存期：管理资源的分配、释放和边界 (defer, errdefer, cleanup)
    LifeCycle,
    /// 元计算：在编译时执行的逻辑 (Zig comptime, Rust macros)
    Meta,
    /// 异常与中断：非局部控制流跳转 (try/catch, longjmp, panic)
    Trap,
}

/// 意图上下文 (Intent Context)
/// 约束的容器和作用范围。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Context {
    /// 线性生命周期上下文 (Ownership, Move semantics)
    Linear,
    /// 引用计数上下文 (Swift ARC, Python RC)
    RefCounting,
    /// 异步调度上下文 (Kotlin Coroutines, Go Goroutines)
    Async,
    /// 编译时上下文 (Zig comptime, Mojo static evaluation)
    Comptime,
    /// 硬件相关：GPU 硬件加速上下文
    GPU,
    /// 硬件相关：SIMD 向量化上下文
    SIMD,
    /// 内存安全：非空约束 (Kotlin Non-null, Swift Optional)
    Safe,
    /// 默认通用上下文
    General,
}

/// 携带约束的意图原子
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentNode {
    pub intent: Intent,
    pub contexts: Vec<Context>,
    /// 属性集 (Attributes)，用于存储变量名、常量值等元数据
    pub attributes: HashMap<String, String>,
}

impl IntentNode {
    pub fn new(intent: Intent) -> Self {
        Self {
            intent,
            contexts: vec![Context::General],
            attributes: HashMap::new(),
        }
    }

    pub fn with_context(mut self, context: Context) -> Self {
        self.contexts.push(context);
        self
    }

    pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }
}

/// 乔姆斯基框架中心化错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChomskyError {
    pub kind: Box<ChomskyErrorKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChomskyErrorKind {
    UirError {
        kind: String, // e.g., "CycleDetected", "TypeMismatch"
        details: String,
    },
    FrontendError {
        stage: String, // e.g., "Parsing", "TypeChecking"
        file: Option<String>,
        line: Option<u32>,
        message: String,
    },
    BackendError {
        target: String, // e.g., "x86_64", "CUDA"
        stage: String,  // e.g., "Selection", "Scheduling"
        message: String,
    },
    IoError {
        path: Option<String>,
        operation: String,
        message: String,
    },
    Unknown {
        code: i32,
        message: String,
    },
}

impl ChomskyErrorKind {
    pub fn key(&self) -> &str {
        match self {
            Self::UirError { .. } => "uir_error",
            Self::FrontendError { .. } => "frontend_error",
            Self::BackendError { .. } => "backend_error",
            Self::IoError { .. } => "io_error",
            Self::Unknown { .. } => "unknown",
        }
    }
}

impl std::fmt::Display for ChomskyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self.kind {
            ChomskyErrorKind::UirError { kind, details } => {
                write!(f, "UIR Error [{}]: {}", kind, details)
            }
            ChomskyErrorKind::FrontendError {
                stage,
                file,
                line,
                message,
            } => {
                let loc = match (file, line) {
                    (Some(file), Some(line)) => format!("{}:{}", file, line),
                    (Some(file), None) => file.clone(),
                    _ => "unknown location".to_string(),
                };
                write!(f, "Frontend Error [{} at {}]: {}", stage, loc, message)
            }
            ChomskyErrorKind::BackendError {
                target,
                stage,
                message,
            } => write!(f, "Backend Error [{} - {}]: {}", target, stage, message),
            ChomskyErrorKind::IoError {
                path,
                operation,
                message,
            } => {
                let p = path.as_deref().unwrap_or("unknown path");
                write!(f, "IO Error [{} on {}]: {}", operation, p, message)
            }
            ChomskyErrorKind::Unknown { code, message } => {
                write!(f, "Unknown Error ({}): {}", code, message)
            }
        }
    }
}

impl std::error::Error for ChomskyError {}

impl ChomskyError {
    pub fn new(kind: ChomskyErrorKind) -> Self {
        Self {
            kind: Box::new(kind),
        }
    }

    pub fn uir_error(msg: impl Into<String>) -> Self {
        Self::new(ChomskyErrorKind::UirError {
            kind: "General".to_string(),
            details: msg.into(),
        })
    }

    pub fn frontend_error(msg: impl Into<String>) -> Self {
        Self::new(ChomskyErrorKind::FrontendError {
            stage: "General".to_string(),
            file: None,
            line: None,
            message: msg.into(),
        })
    }

    pub fn backend_error(msg: impl Into<String>) -> Self {
        Self::new(ChomskyErrorKind::BackendError {
            target: "Unknown".to_string(),
            stage: "General".to_string(),
            message: msg.into(),
        })
    }

    pub fn io_error(msg: impl Into<String>) -> Self {
        Self::new(ChomskyErrorKind::IoError {
            path: None,
            operation: "Unknown".to_string(),
            message: msg.into(),
        })
    }

    pub fn unknown(msg: impl Into<String>) -> Self {
        Self::new(ChomskyErrorKind::Unknown {
            code: -1,
            message: msg.into(),
        })
    }
}

impl From<std::io::Error> for ChomskyError {
    fn from(err: std::io::Error) -> Self {
        Self::io_error(err.to_string())
    }
}

pub type ChomskyResult<T> = Result<T, ChomskyError>;
