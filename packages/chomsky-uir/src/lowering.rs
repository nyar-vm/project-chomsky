use crate::intent::IKun;
use crate::egraph::{EGraph, Id, Analysis};
use std::collections::HashSet;

pub struct LoweringContext<'a, A: Analysis<IKun>> {
    pub egraph: &'a EGraph<IKun, A>,
    /// Current scope's defined variables (including params)
    pub scope_stack: Vec<HashSet<String>>,
    /// Variables that are captured by nested functions
    pub captured: Vec<HashSet<String>>,
}

impl<'a, A: Analysis<IKun>> LoweringContext<'a, A> {
    pub fn new(egraph: &'a EGraph<IKun, A>) -> Self {
        Self {
            egraph,
            scope_stack: vec![HashSet::new()],
            captured: vec![HashSet::new()],
        }
    }

    pub fn enter_scope(&mut self, params: &[String]) {
        let mut scope = HashSet::new();
        for p in params {
            scope.insert(p.clone());
        }
        self.scope_stack.push(scope);
        self.captured.push(HashSet::new());
    }

    pub fn exit_scope(&mut self) -> HashSet<String> {
        self.scope_stack.pop();
        self.captured.pop().expect("Captured stack underflow")
    }

    pub fn define(&mut self, name: String) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(name);
        }
    }

    pub fn reference(&mut self, name: &str) {
        // If the variable is not in the current scope but exists in a parent scope,
        // it is being captured.
        let mut found_in_current = false;
        if let Some(current) = self.scope_stack.last() {
            if current.contains(name) {
                found_in_current = true;
            }
        }

        if !found_in_current {
            // Check parent scopes
            for i in (0..self.scope_stack.len() - 1).rev() {
                if self.scope_stack[i].contains(name) {
                    // Mark as captured in all scopes from the defining one to the current one
                    for j in (i + 1)..self.scope_stack.len() {
                        self.captured[j].insert(name.to_string());
                    }
                    break;
                }
            }
        }
    }
}

