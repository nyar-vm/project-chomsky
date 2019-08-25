use crate::egraph::{Analysis, EGraph, Id};
use crate::intent::{CrossLanguageCall, IKun};
use chomsky_types::Loc;

pub struct IntentBuilder<'a, A: Analysis<IKun>> {
    pub egraph: &'a mut EGraph<IKun, A>,
}

impl<'a, A: Analysis<IKun>> IntentBuilder<'a, A> {
    pub fn new(egraph: &'a mut EGraph<IKun, A>) -> Self {
        Self { egraph }
    }

    /// Add a raw IKun node to the graph with location
    pub fn add(&mut self, node: IKun, loc: Loc) -> Id {
        self.egraph.add_with_loc(node, loc)
    }

    // --- Basic Atoms ---

    pub fn constant(&mut self, v: i64, loc: Loc) -> Id {
        self.add(IKun::Constant(v), loc)
    }

    pub fn int(&mut self, v: i64, loc: Loc) -> Id {
        self.constant(v, loc)
    }

    pub fn float(&mut self, v: f64, loc: Loc) -> Id {
        self.add(IKun::FloatConstant(v.to_bits()), loc)
    }

    pub fn bool(&mut self, v: bool, loc: Loc) -> Id {
        self.add(IKun::BooleanConstant(v), loc)
    }

    pub fn string(&mut self, s: &str, loc: Loc) -> Id {
        self.add(IKun::StringConstant(s.to_string()), loc)
    }

    pub fn symbol(&mut self, s: &str, loc: Loc) -> Id {
        self.add(IKun::Symbol(s.to_string()), loc)
    }

    pub fn import(&mut self, module: &str, name: &str, loc: Loc) -> Id {
        self.add(IKun::Import(module.to_string(), name.to_string()), loc)
    }

    pub fn export(&mut self, name: &str, body: Id, loc: Loc) -> Id {
        self.add(IKun::Export(name.to_string(), body), loc)
    }

    // --- Structure ---

    pub fn seq(&mut self, items: Vec<Id>, loc: Loc) -> Id {
        self.add(IKun::Seq(items), loc)
    }

    pub fn map(&mut self, f: Id, input: Id, loc: Loc) -> Id {
        self.add(IKun::Map(f, input), loc)
    }

    pub fn reduce(&mut self, f: Id, init: Id, list: Id, loc: Loc) -> Id {
        self.add(IKun::Reduce(f, init, list), loc)
    }

    pub fn filter(&mut self, f: Id, input: Id, loc: Loc) -> Id {
        self.add(IKun::Filter(f, input), loc)
    }

    pub fn call(&mut self, func: Id, args: Vec<Id>, loc: Loc) -> Id {
        self.add(IKun::Apply(func, args), loc)
    }

    pub fn lambda(&mut self, params: Vec<String>, body: Id, loc: Loc) -> Id {
        self.add(IKun::Lambda(params, body), loc)
    }

    pub fn assign(&mut self, name: &str, value: Id, loc: Loc) -> Id {
        let sym = self.symbol(name, loc);
        self.add(IKun::StateUpdate(sym, value), loc)
    }

    pub fn assign_to_id(&mut self, target: Id, value: Id, loc: Loc) -> Id {
        self.add(IKun::StateUpdate(target, value), loc)
    }

    pub fn block(&mut self, stmts: Vec<Id>, loc: Loc) -> Id {
        self.seq(stmts, loc)
    }

    pub fn module(&mut self, name: &str, items: Vec<Id>, loc: Loc) -> Id {
        self.add(IKun::Module(name.to_string(), items), loc)
    }

    // --- Control Flow ---

    pub fn branch(&mut self, cond: Id, then_branch: Id, else_branch: Id, loc: Loc) -> Id {
        self.add(IKun::Choice(cond, then_branch, else_branch), loc)
    }

    pub fn if_(&mut self, cond: Id, then_branch: Id, else_branch: Option<Id>, loc: Loc) -> Id {
        let else_id = else_branch.unwrap_or_else(|| self.block(vec![], loc));
        self.branch(cond, then_branch, else_id, loc)
    }

    pub fn loop_(&mut self, count: Id, body: Id, loc: Loc) -> Id {
        self.add(IKun::Repeat(count, body), loc)
    }

    pub fn while_loop(&mut self, cond: Id, body: Id, loc: Loc) -> Id {
        // While loop is a bit complex in pure functional IKun, usually represented as
        // Repeat(Inf, Choice(cond, body, Break)) or similar.
        // For now, let's map it to an Extension "while" or just use Repeat if semantic allows.
        // But IKun::Repeat takes a count.
        // Let's use Extension for "while" for now until we have a proper construct.
        self.extension("while", vec![cond, body], loc)
    }

    pub fn while_( &mut self, cond: Id, body: Id, loc: Loc) -> Id {
        self.while_loop(cond, body, loc)
    }

    pub fn break_(&mut self, loc: Loc) -> Id {
        self.extension("break", vec![], loc)
    }

    pub fn continue_(&mut self, loc: Loc) -> Id {
        self.extension("continue", vec![], loc)
    }

    pub fn return_(&mut self, value: Id, loc: Loc) -> Id {
        self.add(IKun::Return(value), loc)
    }

    pub fn cross_lang_call(
        &mut self,
        lang: &str,
        group: &str,
        func: &str,
        args: Vec<Id>,
        loc: Loc,
    ) -> Id {
        self.add(
            IKun::CrossLangCall(CrossLanguageCall {
                language: lang.to_string(),
                module_path: group.to_string(),
                function_name: func.to_string(),
                arguments: args,
            }),
            loc,
        )
    }

    // --- Operations ---

    pub fn extension(&mut self, name: &str, args: Vec<Id>, loc: Loc) -> Id {
        self.add(IKun::Extension(name.to_string(), args), loc)
    }

    pub fn binary_op(&mut self, op: &str, left: Id, right: Id, loc: Loc) -> Id {
        self.extension(op, vec![left, right], loc)
    }

    pub fn add_op(&mut self, left: Id, right: Id, loc: Loc) -> Id {
        self.binary_op("add", left, right, loc)
    }

    pub fn sub_op(&mut self, left: Id, right: Id, loc: Loc) -> Id {
        self.binary_op("sub", left, right, loc)
    }

    pub fn mul_op(&mut self, left: Id, right: Id, loc: Loc) -> Id {
        self.binary_op("mul", left, right, loc)
    }

    pub fn div_op(&mut self, left: Id, right: Id, loc: Loc) -> Id {
        self.binary_op("div", left, right, loc)
    }

    pub fn eq_op(&mut self, left: Id, right: Id, loc: Loc) -> Id {
        self.binary_op("eq", left, right, loc)
    }

    pub fn ne_op(&mut self, left: Id, right: Id, loc: Loc) -> Id {
        self.binary_op("ne", left, right, loc)
    }

    pub fn lt_op(&mut self, left: Id, right: Id, loc: Loc) -> Id {
        self.binary_op("lt", left, right, loc)
    }

    pub fn le_op(&mut self, left: Id, right: Id, loc: Loc) -> Id {
        self.binary_op("le", left, right, loc)
    }

    pub fn gt_op(&mut self, left: Id, right: Id, loc: Loc) -> Id {
        self.binary_op("gt", left, right, loc)
    }

    pub fn ge_op(&mut self, left: Id, right: Id, loc: Loc) -> Id {
        self.binary_op("ge", left, right, loc)
    }

    pub fn neg_op(&mut self, val: Id, loc: Loc) -> Id {
        self.extension("neg", vec![val], loc)
    }

    pub fn not_op(&mut self, val: Id, loc: Loc) -> Id {
        self.extension("not", vec![val], loc)
    }

    pub fn len_op(&mut self, val: Id, loc: Loc) -> Id {
        self.extension("len", vec![val], loc)
    }

    pub fn bit_not_op(&mut self, val: Id, loc: Loc) -> Id {
        self.extension("bit_not", vec![val], loc)
    }

    pub fn get_index(&mut self, obj: Id, key: Id, loc: Loc) -> Id {
        self.extension("get_index", vec![obj, key], loc)
    }

    pub fn set_index(&mut self, obj: Id, key: Id, val: Id, loc: Loc) -> Id {
        self.extension("set_index", vec![obj, key, val], loc)
    }

    pub fn parameter(&mut self, name: &str, _loc: Loc) -> String {
        name.to_string()
    }

    pub fn function(&mut self, name: &str, params: Vec<String>, body: Vec<Id>, loc: Loc) -> Id {
        let body_seq = self.block(body, loc.clone());
        let lam = self.lambda(params, body_seq, loc.clone());
        if name == "anonymous" {
            lam
        } else {
            self.assign(name, lam, loc)
        }
    }

    // --- Resource Management ---

    pub fn resource_clone(&mut self, target: Id, loc: Loc) -> Id {
        self.add(IKun::ResourceClone(target), loc)
    }

    pub fn resource_drop(&mut self, target: Id, loc: Loc) -> Id {
        self.add(IKun::ResourceDrop(target), loc)
    }

    pub fn resource_context(&mut self, loc: Loc) -> Id {
        self.add(IKun::ResourceContext, loc)
    }
}
