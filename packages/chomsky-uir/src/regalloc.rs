use crate::IKunTree;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Register {
    Physical(String),
    Stack(usize),
}

pub struct RegisterAllocation {
    pub mapping: HashMap<String, Register>,
}

pub struct LinearScanAllocator {
    registers: Vec<String>,
}

impl LinearScanAllocator {
    pub fn new(registers: Vec<String>) -> Self {
        Self { registers }
    }

    /// 执行简单的寄存器分配
    pub fn allocate(&self, tree: &IKunTree) -> RegisterAllocation {
        let mut mapping = HashMap::new();
        let mut available = self.registers.clone();
        let mut stack_offset = 0;

        // 简单的线性扫描演示实现
        self.collect_and_alloc(tree, &mut mapping, &mut available, &mut stack_offset);

        RegisterAllocation { mapping }
    }

    fn collect_and_alloc(
        &self,
        tree: &IKunTree,
        mapping: &mut HashMap<String, Register>,
        available: &mut Vec<String>,
        stack_offset: &mut usize,
    ) {
        match tree {
            IKunTree::Symbol(s) => {
                if !mapping.contains_key(s) {
                    if let Some(reg) = available.pop() {
                        mapping.insert(s.clone(), Register::Physical(reg));
                    } else {
                        mapping.insert(s.clone(), Register::Stack(*stack_offset));
                        *stack_offset += 8;
                    }
                }
            }
            IKunTree::Apply(f, args) => {
                self.collect_and_alloc(f, mapping, available, stack_offset);
                for arg in args {
                    self.collect_and_alloc(arg, mapping, available, stack_offset);
                }
            }
            IKunTree::Map(f, x) | IKunTree::Filter(f, x) => {
                self.collect_and_alloc(f, mapping, available, stack_offset);
                self.collect_and_alloc(x, mapping, available, stack_offset);
            }
            IKunTree::Reduce(f, init, x) => {
                self.collect_and_alloc(f, mapping, available, stack_offset);
                self.collect_and_alloc(init, mapping, available, stack_offset);
                self.collect_and_alloc(x, mapping, available, stack_offset);
            }
            IKunTree::Seq(ids) => {
                for id in ids {
                    self.collect_and_alloc(id, mapping, available, stack_offset);
                }
            }
            IKunTree::StateUpdate(target, val) => {
                self.collect_and_alloc(target, mapping, available, stack_offset);
                self.collect_and_alloc(val, mapping, available, stack_offset);
            }
            IKunTree::Lambda(_, body) => {
                self.collect_and_alloc(body, mapping, available, stack_offset);
            }
            IKunTree::Extension(_, args) => {
                for arg in args {
                    self.collect_and_alloc(arg, mapping, available, stack_offset);
                }
            }
            IKunTree::Choice(cond, then_b, else_b) => {
                self.collect_and_alloc(cond, mapping, available, stack_offset);
                self.collect_and_alloc(then_b, mapping, available, stack_offset);
                self.collect_and_alloc(else_b, mapping, available, stack_offset);
            }
            _ => {}
        }
    }
}
