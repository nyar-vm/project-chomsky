use dashmap::DashMap;

pub type Id = usize;

#[derive(Debug, Clone, Default)]
pub struct UnionFind {
    pub parents: DashMap<Id, Id>,
}

impl UnionFind {
    pub fn new() -> Self {
        Self {
            parents: DashMap::new(),
        }
    }

    pub fn find(&self, i: Id) -> Id {
        if !self.parents.contains_key(&i) {
            self.parents.insert(i, i);
            return i;
        }

        let mut curr = i;
        let mut path = Vec::new();
        while let Some(p) = self.parents.get(&curr) {
            if *p == curr {
                break;
            }
            path.push(curr);
            curr = *p;
        }

        for id in path {
            self.parents.insert(id, curr);
        }
        curr
    }

    pub fn union(&self, i: Id, j: Id) -> Id {
        let root_i = self.find(i);
        let root_j = self.find(j);
        if root_i != root_j {
            self.parents.insert(root_i, root_j);
        }
        root_j
    }
}
