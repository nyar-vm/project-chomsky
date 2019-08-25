use chomsky_glue::{CrossLangLinter, CrossLangRegistry, ExternalFuncMetadata};
use chomsky_uir::constraint::ConstraintSet;
use chomsky_uir::intent::CrossLanguageCall;
use chomsky_uir::{ConstraintAnalysis, EGraph, IKun};

#[test]
fn test_cross_lang_registry() {
    let mut registry = CrossLangRegistry::new();
    registry.register(ExternalFuncMetadata {
        lang: "python".to_string(),
        name: "numpy_sum".to_string(),
        required_constraints: ConstraintSet::default(),
        provided_constraints: ConstraintSet::default(),
    });

    assert!(registry.get("python", "numpy_sum").is_some());
    assert!(registry.get("python", "missing").is_none());
}

#[test]
fn test_cross_lang_linter_unknown() {
    let registry = CrossLangRegistry::new();
    let linter = CrossLangLinter::new(&registry);
    let egraph = EGraph::<IKun, ConstraintAnalysis>::new();

    let call = egraph.add(IKun::CrossLangCall(CrossLanguageCall {
        language: "python".to_string(),
        module_path: "unknown".to_string(),
        function_name: vec![],
    }));

    let errors = linter.lint(&egraph);
    assert!(!errors.is_empty());
    assert!(errors[0].contains("Unknown cross-lang function"));
}
