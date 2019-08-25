use chomsky_uir::analysis::ConstraintAnalysis;
use chomsky_uir::constraint::{ConstraintSet, Effect, Ownership};
use chomsky_uir::egraph::EGraph;
use chomsky_uir::intent::{CrossLanguageCall, IKun};

#[test]
fn test_effect_propagation() {
    let egraph = EGraph::<IKun, ConstraintAnalysis>::new();

    // 1. Create a pure constant
    let c1 = egraph.add(IKun::Constant(42));
    let data1 = egraph.get_class(c1).data.clone();
    assert_eq!(data1.effect, Effect::Pure);

    // 2. Create a state update (WriteOnly)
    let target = egraph.add(IKun::Symbol("x".to_string()));
    let val = egraph.add(IKun::Constant(1));
    let update = egraph.add(IKun::StateUpdate(target, val));
    let data_update = egraph.get_class(update).data.clone();
    assert_eq!(data_update.effect, Effect::WriteOnly);

    // 3. Seq of pure and write -> WriteOnly
    let seq = egraph.add(IKun::Seq(vec![c1, update]));
    let data_seq = egraph.get_class(seq).data.clone();
    assert_eq!(data_seq.effect, Effect::WriteOnly);

    // 4. Cross-lang call -> ReadWrite
    let call = egraph.add(IKun::CrossLangCall(CrossLanguageCall {
        language: "js".to_string(),
        module_path: "print".to_string(),
        function_name: vec![c1],
    }));
    let data_call = egraph.get_class(call).data.clone();
    assert_eq!(data_call.effect, Effect::ReadWrite);

    // 5. Seq of WriteOnly and ReadWrite -> ReadWrite
    let seq2 = egraph.add(IKun::Seq(vec![update, call]));
    let data_seq2 = egraph.get_class(seq2).data.clone();
    assert_eq!(data_seq2.effect, Effect::ReadWrite);
}

#[test]
fn test_ownership_and_type_conflict() {
    let egraph = EGraph::<IKun, ConstraintAnalysis>::new();

    // Create two nodes with different types
    let t1 = egraph.add(IKun::TypeConstraint("int".to_string()));
    let t2 = egraph.add(IKun::TypeConstraint("float".to_string()));

    // Try to union them -> should fail compatibility check
    let root = egraph.union(t1, t2);

    // In our EGraph implementation, union returns root1 if incompatible
    // Let's verify they are still in different classes or at least the merge didn't happen
    let root1 = egraph.union_find.find(t1);
    let root2 = egraph.union_find.find(t2);
    assert_ne!(root1, root2);

    // Same for ownership
    let o1 = egraph.add(IKun::OwnershipConstraint(Ownership::Linear));
    let o2 = egraph.add(IKun::OwnershipConstraint(Ownership::Shared));
    let root_o = egraph.union(o1, o2);
    let root_o1 = egraph.union_find.find(o1);
    let root_o2 = egraph.union_find.find(o2);
    assert_ne!(root_o1, root_o2);
}

#[test]
fn test_with_constraint() {
    let egraph = EGraph::<IKun, ConstraintAnalysis>::new();

    let val = egraph.add(IKun::Constant(100));
    let constraint = egraph.add(IKun::EffectConstraint(Effect::ReadOnly));

    let constrained_val = egraph.add(IKun::WithConstraint(val, constraint));
    let data = egraph.get_class(constrained_val).data.clone();

    assert_eq!(data.effect, Effect::ReadOnly);
}
