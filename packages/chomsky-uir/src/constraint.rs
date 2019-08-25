use crate::egraph::HasDebugInfo;
use chomsky_types::Loc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Effect {
    Pure,
    ReadOnly,
    WriteOnly,
    ReadWrite,
    Panic,
    Diverge,
}

impl Effect {
    pub fn join(&self, other: &Self) -> Self {
        match (self, other) {
            (Effect::Diverge, _) | (_, Effect::Diverge) => Effect::Diverge,
            (Effect::Panic, _) | (_, Effect::Panic) => Effect::Panic,
            (Effect::ReadWrite, _) | (_, Effect::ReadWrite) => Effect::ReadWrite,
            (Effect::WriteOnly, Effect::ReadOnly) | (Effect::ReadOnly, Effect::WriteOnly) => {
                Effect::ReadWrite
            }
            (Effect::WriteOnly, _) | (_, Effect::WriteOnly) => Effect::WriteOnly,
            (Effect::ReadOnly, _) | (_, Effect::ReadOnly) => Effect::ReadOnly,
            (Effect::Pure, other) => *other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Ownership {
    Borrowed,
    Owned,
    Shared,
    Linear,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Constraint {
    Effect(Effect),
    Ownership(Ownership),
    Atomic,
    Type(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConstraintSet {
    pub effect: Effect,
    pub ownership: Option<Ownership>,
    pub is_atomic: bool,
    pub r#type: Option<String>,
}

impl Default for ConstraintSet {
    fn default() -> Self {
        Self {
            effect: Effect::Pure,
            ownership: None,
            is_atomic: false,
            r#type: None,
        }
    }
}

impl ConstraintSet {
    pub fn merge(&mut self, other: &Self) -> bool {
        let mut changed = false;

        let new_effect = self.effect.join(&other.effect);
        if new_effect != self.effect {
            self.effect = new_effect;
            changed = true;
        }

        if let Some(o) = &other.ownership {
            if self.ownership.as_ref() != Some(o) {
                if self.ownership.is_none() {
                    self.ownership = Some(*o);
                    changed = true;
                }
            }
        }

        if other.is_atomic && !self.is_atomic {
            self.is_atomic = true;
            changed = true;
        }

        if let Some(t) = &other.r#type {
            if self.r#type.as_ref() != Some(t) {
                if self.r#type.is_none() {
                    self.r#type = Some(t.clone());
                    changed = true;
                }
            }
        }

        changed
    }
}

impl HasDebugInfo for ConstraintSet {
    fn get_locs(&self) -> &[Loc] {
        &[]
    }
}

impl ConstraintSet {
    pub fn check_conflict(&self, other: &Self) -> Option<String> {
        // Ownership conflict: cannot merge different ownerships (except maybe some subtyping)
        if let (Some(o1), Some(o2)) = (&self.ownership, &other.ownership) {
            if o1 != o2 {
                return Some(format!("Ownership conflict: {:?} vs {:?}", o1, o2));
            }
        }

        // Type conflict: cannot merge different types
        if let (Some(t1), Some(t2)) = (&self.r#type, &other.r#type) {
            if t1 != t2 {
                return Some(format!("Type conflict: {} vs {}", t1, t2));
            }
        }

        None
    }

    pub fn can_merge(&self, other: &Self) -> bool {
        self.check_conflict(other).is_none()
    }
}
