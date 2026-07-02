//! Runtime directory metadata store.

use std::collections::HashMap;

use crate::vfs::NodeId;

#[derive(Debug, Clone)]
pub struct DirectoryRecord {
    pub node_id: NodeId,
    pub virtual_path: String,
    pub clusters: Vec<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct DirectoryStore {
    by_path: HashMap<String, DirectoryRecord>,
}

impl DirectoryStore {
    pub fn insert_directory(&mut self, virtual_path: String, node_id: NodeId, clusters: Vec<u32>) {
        self.by_path.insert(
            virtual_path.clone(),
            DirectoryRecord {
                node_id,
                virtual_path,
                clusters,
            },
        );
    }

    pub fn directory_clusters(&self, path: &str) -> Option<&[u32]> {
        self.by_path.get(path).map(|record| record.clusters.as_slice())
    }

    pub fn records(&self) -> impl Iterator<Item = &DirectoryRecord> {
        self.by_path.values()
    }

    pub fn remove_subtree(&mut self, root: &str) -> Vec<u32> {
        let removed = self
            .by_path
            .keys()
            .filter(|path| is_same_path_or_child(path, root))
            .cloned()
            .collect::<Vec<_>>();
        removed
            .into_iter()
            .filter_map(|path| self.by_path.remove(&path))
            .flat_map(|record| record.clusters)
            .collect()
    }

    pub fn rename_subtree(&mut self, from: &str, to: &str) {
        let renamed = self
            .by_path
            .iter()
            .filter_map(|(path, record)| {
                remap_virtual_path(path, from, to).map(|new_path| {
                    let mut record = record.clone();
                    record.virtual_path = new_path.clone();
                    (new_path, record)
                })
            })
            .collect::<Vec<_>>();
        self.by_path
            .retain(|path, _| !is_same_path_or_child(path, from));
        for (path, record) in renamed {
            self.by_path.insert(path, record);
        }
    }
}

fn is_same_path_or_child(candidate: &str, root: &str) -> bool {
    candidate == root
        || candidate
            .strip_prefix(root)
            .map(|suffix| suffix.starts_with('/'))
            .unwrap_or(false)
}

fn remap_virtual_path(path: &str, from: &str, to: &str) -> Option<String> {
    if path == from {
        return Some(to.to_string());
    }
    path.strip_prefix(from)
        .filter(|suffix| suffix.starts_with('/'))
        .map(|suffix| format!("{to}{suffix}"))
}
