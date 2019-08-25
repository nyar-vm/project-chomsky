#![warn(missing_docs)]

pub mod optimizer;
pub mod verify;

pub mod adapters;

#[cfg(feature = "clr")]
pub use clr_assembler as backend_clr;
#[cfg(feature = "elf")]
pub use elf_assembler as backend_elf;
#[cfg(feature = "gcn")]
pub use gcn_assembler as backend_gcn;
#[cfg(feature = "jvm")]
pub use jvm_assembler as backend_jvm;
#[cfg(feature = "sass")]
pub use sass_assembler as backend_cuda;
#[cfg(feature = "spirv")]
pub use spirv_assembler as backend_spirv;
#[cfg(feature = "wasi")]
pub use wasi_assembler as backend_wasm;
#[cfg(feature = "x86")]
pub use x86_64_assembler as backend_x86_64;

pub use chomsky_concretize as concretize;
pub use chomsky_context as context;
pub use chomsky_cost as cost;
pub use chomsky_emit as emit;
pub use chomsky_extract as extract;
pub use chomsky_glue as glue;
pub use chomsky_lifetime as lifetime;
pub use chomsky_rule_engine as rule_engine;
pub use chomsky_rules as rules;
pub use chomsky_uir as uir;
