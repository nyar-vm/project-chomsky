#![warn(missing_docs)]

use chomsky_uir::IKun;
use chomsky_uir::constraint::ConstraintSet;
use chomsky_uir::egraph::EGraph;
use chomsky_uir::intent::CrossLanguageCall;
use std::collections::HashMap;

/// Metadata for a function in another language
pub struct ExternalFuncMetadata {
    pub lang: String,
    pub name: String,
    pub required_constraints: ConstraintSet,
    pub provided_constraints: ConstraintSet,
}

/// A registry of external functions and their constraints
pub struct CrossLangRegistry {
    funcs: HashMap<(String, String), ExternalFuncMetadata>,
}

impl CrossLangRegistry {
    pub fn new() -> Self {
        Self {
            funcs: HashMap::new(),
        }
    }

    pub fn register(&mut self, metadata: ExternalFuncMetadata) {
        self.funcs
            .insert((metadata.lang.clone(), metadata.name.clone()), metadata);
    }

    pub fn get(&self, lang: &str, name: &str) -> Option<&ExternalFuncMetadata> {
        self.funcs.get(&(lang.to_string(), name.to_string()))
    }
}

pub struct CrossLangLinter<'a> {
    registry: &'a CrossLangRegistry,
}

impl<'a> CrossLangLinter<'a> {
    pub fn new(registry: &'a CrossLangRegistry) -> Self {
        Self { registry }
    }

    /// Validates all cross-language calls in the E-Graph
    pub fn lint<A>(&self, egraph: &EGraph<IKun, A>) -> Vec<String>
    where
        A: chomsky_uir::egraph::Analysis<IKun, Data = chomsky_uir::constraint::ConstraintSet>,
    {
        let mut errors = Vec::new();

        for entry in egraph.classes.iter() {
            let eclass = entry.value();
            for enode in &eclass.nodes {
                if let IKun::CrossLangCall(CrossLanguageCall {
                    language: lang,
                    module_path: _group,
                    function_name: func,
                    arguments: _args,
                }) = enode
                {
                    if let Some(metadata) = self.registry.get(lang, func) {
                        // Check if the current context/constraints satisfy the external function's requirements
                        let current_data = &eclass.data;

                        // 1. Check effects
                        let req_effect = metadata.required_constraints.effect;
                        let cur_effect = current_data.effect;
                        // Simple check: for now, assume they must match or cur must be "purer" than req
                        // (This logic might need refinement based on effect lattice)
                        if cur_effect != req_effect
                            && req_effect != chomsky_uir::constraint::Effect::Pure
                        {
                            errors.push(format!(
                                "Cross-lang call effect error: {}:{} requires effect {:?}, but current is {:?}",
                                lang, func, req_effect, cur_effect
                            ));
                        }

                        // 2. Check ownership constraints
                        if let Some(req_own) = metadata.required_constraints.ownership {
                            if current_data.ownership != Some(req_own) {
                                errors.push(format!(
                                    "Cross-lang call ownership error: {}:{} requires ownership {:?}, but current is {:?}",
                                    lang, func, req_own, current_data.ownership
                                ));
                            }
                        }

                        // 3. Check type constraints
                        if let Some(req_type) = &metadata.required_constraints.r#type {
                            if current_data.r#type.as_ref() != Some(req_type) {
                                errors.push(format!(
                                    "Cross-lang call type error: {}:{} requires type {}, but current is {:?}",
                                    lang, func, req_type, current_data.r#type
                                ));
                            }
                        }
                    } else {
                        errors.push(format!("Unknown cross-lang function: {}:{}", lang, func));
                    }
                }
            }
        }

        errors
    }
}

/// Trait for language-specific glue generation
pub trait GlueProvider {
    fn lang(&self) -> &str;
    fn generate_adapter(&self, func: &str) -> String;
}

pub struct PythonGlueProvider;
impl GlueProvider for PythonGlueProvider {
    fn lang(&self) -> &str {
        "python"
    }
    fn generate_adapter(&self, func: &str) -> String {
        format!(
            "// Generated glue for Python function: {}\nextern \"C\" void chomsky_py_{}(void* args);",
            func, func
        )
    }
}

pub struct TypeScriptGlueProvider;
impl GlueProvider for TypeScriptGlueProvider {
    fn lang(&self) -> &str {
        "typescript"
    }
    fn generate_adapter(&self, func: &str) -> String {
        format!(
            "// Generated glue for TS function: {}\nextern \"C\" void chomsky_ts_{}(void* args);",
            func, func
        )
    }
}

/// Extensible Glue Generator
pub struct GlueGenerator {
    providers: HashMap<String, Box<dyn GlueProvider>>,
}

impl GlueGenerator {
    pub fn new() -> Self {
        let mut providers: HashMap<String, Box<dyn GlueProvider>> = HashMap::new();

        // Register default providers
        let py = PythonGlueProvider;
        providers.insert(py.lang().to_string(), Box::new(py));

        let ts = TypeScriptGlueProvider;
        providers.insert(ts.lang().to_string(), Box::new(ts));

        Self { providers }
    }

    pub fn register_provider(&mut self, provider: Box<dyn GlueProvider>) {
        self.providers.insert(provider.lang().to_string(), provider);
    }

    pub fn generate_adapter(&self, lang: &str, func: &str) -> Option<String> {
        self.providers.get(lang).map(|p| p.generate_adapter(func))
    }
}
