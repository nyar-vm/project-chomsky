#![warn(missing_docs)]

use chomsky_uir::{Analysis, EGraph, IKun, Id, Language};
use std::collections::HashMap;

/// Linker for merging multiple IKun E-Graphs and resolving cross-module references.
pub struct IKunLinker<A: Analysis<IKun>> {
    /// The merged global E-Graph.
    pub global_graph: EGraph<IKun, A>,
    /// Mapping from (ModuleName, ExportName) to Id in the global graph.
    exports: HashMap<(String, String), Id>,
}

impl<A: Analysis<IKun> + Default> IKunLinker<A> {
    pub fn new() -> Self {
        Self {
            global_graph: EGraph::new(),
            exports: HashMap::new(),
        }
    }

    /// Add a module's E-Graph to the global graph.
    /// This will copy all nodes and record exports.
    pub fn add_module(&mut self, module_name: &str, graph: &EGraph<IKun, A>) {
        // Step 1: Record exports in the source graph
        for entry in graph.classes.iter() {
            let class = entry.value();
            for node in &class.nodes {
                if let IKun::Export(_name, _body_id) = node {
                    // Record that this module has this export.
                }
            }
        }

        self.merge_graph(module_name, graph);
    }

    fn merge_graph(&mut self, module_name: &str, source: &EGraph<IKun, A>) {
        let mut id_map = HashMap::new();

        // Step 1: Add all nodes and build id_map
        for entry in source.classes.iter() {
            let class = entry.value();
            for node in &class.nodes {
                let new_id = self.add_node_recursive(node, source, &mut id_map);
                id_map.insert(class.id, new_id);

                // Record global export ID
                if let IKun::Export(name, _) = node {
                    self.exports
                        .insert((module_name.to_string(), name.clone()), new_id);
                }
            }
        }
    }

    fn add_node_recursive(
        &mut self,
        node: &IKun,
        source: &EGraph<IKun, A>,
        id_map: &mut HashMap<Id, Id>,
    ) -> Id {
        let remapped = node.map_children(|child_id| {
            if let Some(&new_id) = id_map.get(&child_id) {
                new_id
            } else {
                // If child not in map yet, we must find it in source and add it
                let child_class = source.get_class(child_id);
                // For simplicity, pick the first node in the class
                let child_node = &child_class.nodes[0];
                let new_id = self.add_node_recursive(child_node, source, id_map);
                id_map.insert(child_id, new_id);
                new_id
            }
        });
        self.global_graph.add(remapped)
    }

    /// Link all imports to their corresponding exports.
    pub fn link(&mut self) {
        let mut links_to_make = Vec::new();

        for entry in self.global_graph.classes.iter() {
            let id = *entry.key();
            let class = entry.value();
            for node in &class.nodes {
                if let IKun::Import(mod_name, sym_name) = node {
                    if let Some(&export_id) =
                        self.exports.get(&(mod_name.clone(), sym_name.clone()))
                    {
                        links_to_make.push((id, export_id));
                    }
                }
            }
        }

        // Merge e-classes in the global graph
        for (import_id, export_id) in links_to_make {
            self.global_graph.union(import_id, export_id);
        }

        self.global_graph.rebuild();
    }
}
