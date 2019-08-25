use crate::constraint::{ConstraintSet, Effect};
use crate::egraph::{Analysis, EGraph, Language};
use crate::intent::IKun;

#[derive(Default, Clone, Debug)]
pub struct ConstraintAnalysis;

impl Analysis<IKun> for ConstraintAnalysis {
    type Data = ConstraintSet;

    fn make(egraph: &EGraph<IKun, Self>, enode: &IKun) -> Self::Data {
        let mut set = ConstraintSet::default();
        match enode {
            IKun::EffectConstraint(e) => set.effect = *e,
            IKun::OwnershipConstraint(o) => set.ownership = Some(*o),
            IKun::TypeConstraint(t) => set.r#type = Some(t.clone()),
            IKun::AtomicConstraint => set.is_atomic = true,

            // Propagation logic:
            // WithConstraint(expr, constraint)
            IKun::WithConstraint(expr, constraint) => {
                let expr_data = egraph.get_class(*expr).data.clone();
                let constraint_data = egraph.get_class(*constraint).data.clone();
                set.merge(&expr_data);
                set.merge(&constraint_data);
            }

            // Map(f, input) -> inherits constraints from f and input
            IKun::Map(f, input) => {
                let f_data = egraph.get_class(*f).data.clone();
                let input_data = egraph.get_class(*input).data.clone();
                set.merge(&f_data);
                set.merge(&input_data);
            }

            // Seq(actions) -> union of all constraints
            IKun::Seq(actions) => {
                for &action in actions {
                    let action_data = egraph.get_class(action).data.clone();
                    set.merge(&action_data);
                }
            }

            IKun::StateUpdate(target, val) => {
                let target_data = egraph.get_class(*target).data.clone();
                let val_data = egraph.get_class(*val).data.clone();
                set.merge(&target_data);
                set.merge(&val_data);
                set.effect = set.effect.join(&Effect::WriteOnly);
            }

            IKun::ResourceClone(target) | IKun::ResourceDrop(target) => {
                let target_data = egraph.get_class(*target).data.clone();
                set.merge(&target_data);
                // RC operations are side-effecting (modifying reference counts)
                set.effect = set.effect.join(&Effect::ReadWrite);
            }

            IKun::CrossLangCall(call) => {
                for &arg in &call.arguments {
                    let arg_data = egraph.get_class(arg).data.clone();
                    set.merge(&arg_data);
                }
                // Cross-language calls and intrinsics are assumed to be non-pure by default
                set.effect = set.effect.join(&Effect::ReadWrite);
            }

            // Function application: inherits effect from the function and arguments
            IKun::Apply(func, args) => {
                let func_data = egraph.get_class(*func).data.clone();
                set.merge(&func_data);
                for &arg in args {
                    let arg_data = egraph.get_class(arg).data.clone();
                    set.merge(&arg_data);
                }
            }

            // Lambda definition itself is pure (it's just a value)
            IKun::Lambda(_, _) | IKun::Closure(_, _) => {
                set.effect = Effect::Pure;
            }

            // Default: most atoms are Pure by default?
            IKun::Constant(_)
            | IKun::FloatConstant(_)
            | IKun::BooleanConstant(_)
            | IKun::StringConstant(_)
            | IKun::Symbol(_) => {
                set.effect = Effect::Pure;
            }

            _ => {
                // For other nodes, we collect constraints from children
                for &child in enode.children().iter() {
                    let child_data = egraph.get_class(child).data.clone();
                    set.merge(&child_data);
                }
            }
        }
        set
    }

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> bool {
        to.merge(&from)
    }

    fn is_compatible(&self, data1: &Self::Data, data2: &Self::Data) -> bool {
        data1.can_merge(data2)
    }
}
