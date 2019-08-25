pub use crate::union_find::Id;
use crate::union_find::UnionFind;
use chomsky_types::Loc;
use dashmap::DashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait Language: Hash + Eq + Clone + Ord {
    fn children(&self) -> Vec<Id>;
    fn map_children(&self, f: impl FnMut(Id) -> Id) -> Self;
}

#[derive(Debug, Clone)]
pub struct EClass<L: Language, D> {
    pub id: Id,
    pub nodes: Vec<L>,
    pub data: D,
}

pub trait Analysis<L: Language>: Default {
    type Data;
    fn make(egraph: &EGraph<L, Self>, enode: &L) -> Self::Data;
    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> bool;
    fn on_add(&self, _data: &mut Self::Data, _loc: Loc) {}
    fn is_compatible(&self, _data1: &Self::Data, _data2: &Self::Data) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct EGraph<L: Language, A: Analysis<L> = ()> {
    pub union_find: UnionFind,
    pub classes: DashMap<Id, EClass<L, A::Data>>,
    pub memo: DashMap<L, Id>,
    pub analysis: std::sync::RwLock<A>,
    pub next_id: AtomicUsize,
    pub dirty: DashMap<Id, ()>,
}

impl<L: Language> Analysis<L> for () {
    type Data = ();
    fn make(_egraph: &EGraph<L, Self>, _enode: &L) -> Self::Data {
        ()
    }
    fn merge(&mut self, _to: &mut Self::Data, _from: Self::Data) -> bool {
        false
    }
}

/// A specialized analysis for tracking debug information (Loc)
#[derive(Default)]
pub struct DebugAnalysis;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugData {
    pub locs: Vec<Loc>,
}

pub trait HasDebugInfo {
    fn get_locs(&self) -> &[Loc];
}

impl HasDebugInfo for DebugData {
    fn get_locs(&self) -> &[Loc] {
        &self.locs
    }
}

impl HasDebugInfo for () {
    fn get_locs(&self) -> &[Loc] {
        &[]
    }
}

impl<L: Language> Analysis<L> for DebugAnalysis {
    type Data = DebugData;

    fn make(_egraph: &EGraph<L, Self>, _enode: &L) -> Self::Data {
        DebugData { locs: Vec::new() }
    }

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> bool {
        let old_len = to.locs.len();
        for loc in from.locs {
            if !to.locs.contains(&loc) {
                to.locs.push(loc);
            }
        }
        to.locs.len() > old_len
    }

    fn on_add(&self, data: &mut Self::Data, loc: Loc) {
        if !data.locs.contains(&loc) {
            data.locs.push(loc);
        }
    }
}

impl<L: Language, A: Analysis<L>> EGraph<L, A> {
    pub fn get_class(&self, id: Id) -> dashmap::mapref::one::Ref<'_, Id, EClass<L, A::Data>> {
        let root = self.union_find.find(id);
        self.classes.get(&root).expect("Class not found")
    }

    pub fn new() -> Self {
        Self {
            union_find: UnionFind::new(),
            classes: DashMap::new(),
            memo: DashMap::new(),
            analysis: std::sync::RwLock::new(A::default()),
            next_id: AtomicUsize::new(0),
            dirty: DashMap::new(),
        }
    }

    pub fn add(&self, enode: L) -> Id {
        let canonical = enode.map_children(|id| self.union_find.find(id));
        if let Some(id) = self.memo.get(&canonical) {
            return self.union_find.find(*id);
        }

        let id = Id::from(self.next_id.fetch_add(1, Ordering::SeqCst));

        let data = A::make(self, &canonical);

        self.memo.insert(canonical.clone(), id);
        let eclass = EClass {
            id,
            nodes: vec![canonical],
            data,
        };
        self.classes.insert(id, eclass);

        id
    }

    pub fn add_with_loc(&self, enode: L, loc: Loc) -> Id {
        let id = self.add(enode);
        let root = self.union_find.find(id);
        if let Some(mut eclass) = self.classes.get_mut(&root) {
            let analysis = self.analysis.read().unwrap();
            analysis.on_add(&mut eclass.data, loc);
        }
        root
    }

    pub fn union(&self, id1: Id, id2: Id) -> Id {
        let root1 = self.union_find.find(id1);
        let root2 = self.union_find.find(id2);
        if root1 == root2 {
            return root1;
        }

        // --- Conflict Detection ---
        {
            let analysis = self.analysis.read().unwrap();
            let data1 = &self.classes.get(&root1).unwrap().data;
            let data2 = &self.classes.get(&root2).unwrap().data;

            if !analysis.is_compatible(data1, data2) {
                // Return one of the roots without merging
                return root1;
            }
        }

        let new_root = self.union_find.union(root1, root2);
        let old_root = if new_root == root1 { root2 } else { root1 };

        self.dirty.insert(new_root, ());

        if let Some((_, old_class)) = self.classes.remove(&old_root) {
            let mut new_class = self.classes.get_mut(&new_root).unwrap();
            for node in old_class.nodes {
                if !new_class.nodes.contains(&node) {
                    new_class.nodes.push(node);
                }
            }

            // Actual analysis merge
            let mut analysis = self.analysis.write().unwrap();
            analysis.merge(&mut new_class.data, old_class.data);
        }

        new_root
    }

    pub fn rebuild(&self) {
        while !self.dirty.is_empty() {
            let mut todo = Vec::new();
            let dirty_list: Vec<Id> = self.dirty.iter().map(|e| *e.key()).collect();
            self.dirty.clear();

            for id in dirty_list {
                let root = self.union_find.find(id);
                if let Some(mut eclass) = self.classes.get_mut(&root) {
                    let mut new_nodes = Vec::new();
                    for node in eclass.nodes.drain(..) {
                        let canonical = node.map_children(|child| self.union_find.find(child));
                        if let Some(old_id) = self.memo.get(&canonical) {
                            let old_root = self.union_find.find(*old_id);
                            if old_root != root {
                                todo.push((old_root, root));
                            }
                        }
                        self.memo.insert(canonical.clone(), root);
                        new_nodes.push(canonical);
                    }
                    eclass.nodes = new_nodes;
                }
            }

            for (id1, id2) in todo {
                self.union(id1, id2);
            }
        }
    }
}
