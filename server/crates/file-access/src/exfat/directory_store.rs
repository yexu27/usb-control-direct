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
}
