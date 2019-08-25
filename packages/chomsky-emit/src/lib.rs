#![warn(missing_docs)]

use chomsky_extract::{Backend, BackendArtifact, IKunTree};
use chomsky_types::{ChomskyError, ChomskyResult};
use gaia_assembler::backends::Backend as GaiaBackend;
pub use gaia_assembler::backends::jvm::JvmBackend;
pub use gaia_assembler::backends::wasi::WasiBackend;
pub use gaia_assembler::backends::x86::X86Backend;
use gaia_assembler::config::GaiaConfig;
use gaia_assembler::instruction::{CoreInstruction, GaiaInstruction};
use gaia_assembler::program::{
    GaiaBlock, GaiaClass, GaiaConstant, GaiaFunction, GaiaModule, GaiaTerminator,
};
use gaia_assembler::types::{GaiaSignature, GaiaType};
use std::collections::HashMap;
use std::sync::Arc;

/// Unified emitter for Gaia.
/// Directly generates Gaia IR (GaiaModule) to avoid assembly parsing overhead.
pub struct GaiaEmitter {
    pub target_arch: String,
    pub standalone: bool,
    backends: HashMap<String, Arc<dyn GaiaBackend>>,
}

impl GaiaEmitter {
    pub fn new(target_arch: &str) -> Self {
        Self {
            target_arch: target_arch.to_string(),
            standalone: false,
            backends: HashMap::new(),
        }
    }

    pub fn with_backend(mut self, name: &str, backend: Arc<dyn GaiaBackend>) -> Self {
        self.backends.insert(name.to_string(), backend);
        self
    }

    pub fn emit(&self, tree: &IKunTree) -> ChomskyResult<BackendArtifact> {
        let module = self.tree_to_ir(tree);

        if let Some(backend) = self.backends.get(&self.target_arch) {
            let config = GaiaConfig {
                target: backend.primary_target(),
                ..Default::default()
            };

            let result = backend.generate(&module, &config).map_err(|e| {
                ChomskyError::new(chomsky_types::ChomskyErrorKind::BackendError {
                    target: self.target_arch.clone(),
                    stage: "Generation".to_string(),
                    message: format!("{:?}", e),
                })
            })?;

            // Take the first file as the main artifact for now
            if let Some((_, bytes)) = result.files.into_iter().next() {
                Ok(BackendArtifact::Binary(bytes))
            } else {
                Ok(BackendArtifact::Binary(vec![]))
            }
        } else {
            // Fallback: just return the IR if no backend matches
            Ok(BackendArtifact::Assembly(format!("{:#?}", module)))
        }
    }

    pub fn standalone(mut self) -> Self {
        self.standalone = true;
        self
    }

    fn tree_to_ir(&self, tree: &IKunTree) -> GaiaModule {
        let mut module = GaiaModule {
            name: "jit_module".to_string(),
            functions: Vec::new(),
            structs: Vec::new(),
            classes: Vec::new(),
            constants: Vec::new(),
            globals: Vec::new(),
            imports: Vec::new(),
        };

        self.extract_module_elements(tree, &mut module);

        // If no elements were extracted, wrap the whole tree in a default function
        if module.functions.is_empty() && module.classes.is_empty() {
            let mut body = Vec::new();
            if !self.standalone {
                self.push_jit_prologue(&mut body);
            }
            self.emit_tree_to_instructions(tree, &mut body);
            if !self.standalone {
                body.push(GaiaInstruction::Core(CoreInstruction::Ret));
            }

            module.functions.push(GaiaFunction {
                name: "main".to_string(),
                signature: GaiaSignature {
                    params: Vec::new(),
                    return_type: GaiaType::Void,
                },
                blocks: vec![GaiaBlock {
                    label: "entry".to_string(),
                    instructions: body,
                    terminator: GaiaTerminator::Return,
                }],
                is_external: false,
            });
        }

        module
    }

    fn extract_module_elements(&self, tree: &IKunTree, module: &mut GaiaModule) {
        match tree {
            IKunTree::Seq(items) => {
                for item in items {
                    self.extract_module_elements(item, module);
                }
            }
            IKunTree::Extension(name, args) if name == "class" => {
                if let (Some(IKunTree::StringConstant(name)), Some(IKunTree::Seq(members))) =
                    (args.get(0), args.get(1))
                {
                    let mut class = GaiaClass {
                        name: name.clone(),
                        parent: Some("java/lang/Object".to_string()),
                        interfaces: Vec::new(),
                        fields: Vec::new(),
                        methods: Vec::new(),
                        attributes: Vec::new(),
                    };

                    for member in members {
                        if let IKunTree::Extension(ext_name, ext_args) = member {
                            if ext_name == "method" {
                                if let (
                                    Some(IKunTree::StringConstant(m_name)),
                                    Some(IKunTree::StringConstant(_m_ret)),
                                    Some(m_body),
                                ) = (ext_args.get(0), ext_args.get(1), ext_args.get(2))
                                {
                                    let mut body = Vec::new();
                                    self.emit_tree_to_instructions(m_body, &mut body);

                                    let method = GaiaFunction {
                                        name: m_name.clone(),
                                        signature: GaiaSignature {
                                            params: Vec::new(), // TODO: parse params
                                            return_type: GaiaType::Void,
                                        },
                                        blocks: vec![GaiaBlock {
                                            label: "entry".to_string(),
                                            instructions: body,
                                            terminator: GaiaTerminator::Return,
                                        }],
                                        is_external: false,
                                    };
                                    class.methods.push(method);
                                }
                            }
                        }
                    }
                    module.classes.push(class);
                }
            }
            _ => {}
        }
    }

    fn push_jit_prologue(&self, body: &mut Vec<GaiaInstruction>) {
        // Prologue for NyarVM JIT calling convention (Win64)
        body.push(GaiaInstruction::Core(CoreInstruction::LoadArg(
            0,
            GaiaType::I64,
        )));
        body.push(GaiaInstruction::Core(CoreInstruction::StoreLocal(
            0,
            GaiaType::I64,
        )));
        body.push(GaiaInstruction::Core(CoreInstruction::LoadArg(
            1,
            GaiaType::I64,
        )));
        body.push(GaiaInstruction::Core(CoreInstruction::StoreLocal(
            1,
            GaiaType::I64,
        )));
        body.push(GaiaInstruction::Core(CoreInstruction::LoadArg(
            2,
            GaiaType::I64,
        )));
        body.push(GaiaInstruction::Core(CoreInstruction::StoreLocal(
            2,
            GaiaType::I64,
        )));
        body.push(GaiaInstruction::Core(CoreInstruction::LoadArg(
            3,
            GaiaType::I64,
        )));
        body.push(GaiaInstruction::Core(CoreInstruction::StoreLocal(
            3,
            GaiaType::I64,
        )));
    }

    fn emit_tree_to_instructions(&self, tree: &IKunTree, body: &mut Vec<GaiaInstruction>) {
        match tree {
            IKunTree::Seq(items) => {
                for item in items {
                    self.emit_tree_to_instructions(item, body);
                }
            }
            IKunTree::Extension(name, args) => match name.as_str() {
                "ldc" => {
                    if let Some(IKunTree::StringConstant(s)) = args.get(0) {
                        body.push(GaiaInstruction::Core(CoreInstruction::PushConstant(
                            GaiaConstant::String(s.clone()),
                        )));
                    }
                }
                "getstatic" => {
                    if let (
                        Some(IKunTree::StringConstant(cls)),
                        Some(IKunTree::StringConstant(fld)),
                    ) = (args.get(0), args.get(1))
                    {
                        body.push(GaiaInstruction::Core(CoreInstruction::LoadField(
                            cls.clone(),
                            fld.clone(),
                        )));
                    }
                }
                "invokevirtual" => {
                    if let (
                        Some(IKunTree::StringConstant(_cls)),
                        Some(IKunTree::StringConstant(meth)),
                    ) = (args.get(0), args.get(1))
                    {
                        body.push(GaiaInstruction::Core(CoreInstruction::Call(
                            meth.clone(),
                            1,
                        )));
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl Backend for GaiaEmitter {
    fn name(&self) -> &str {
        &self.target_arch
    }

    fn generate(&self, tree: &IKunTree) -> ChomskyResult<BackendArtifact> {
        self.emit(tree)
    }
}
