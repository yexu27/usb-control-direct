//! 受控虚拟文件系统索引。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::types::ControlledEntry;
use crate::vfs::node::{NodeId, VfsNode, VfsNodeKind};

#[derive(Debug, Clone)]
pub struct VfsIndex {
    root_id: NodeId,
    next_id: u64,
    nodes: HashMap<NodeId, VfsNode>,
    path_index: HashMap<String, NodeId>,
    mount_root: PathBuf,
}

impl VfsIndex {
    pub fn from_controlled_tree(
        mount_root: &Path,
        entries: &[ControlledEntry],
    ) -> Result<Self, std::io::Error> {
        let root_id = NodeId(1);
        let mut index = VfsIndex {
            root_id,
            next_id: 2,
            nodes: HashMap::new(),
            path_index: HashMap::new(),
            mount_root: mount_root.to_path_buf(),
        };
        index.nodes.insert(
            root_id,
            VfsNode {
                id: root_id,
                parent: None,
                name: String::new(),
                virtual_path: "/".to_string(),
                real_path: mount_root.to_path_buf(),
                kind: VfsNodeKind::Directory,
                size: 0,
                is_virus: false,
                exec_type: None,
                extension: String::new(),
                is_autorun_target: false,
                is_autorun_inf: false,
                is_root_shell_script: false,
                children: Vec::new(),
            },
        );
        index.path_index.insert("/".to_string(), root_id);

        for entry in entries {
            index.insert_controlled_entry(root_id, "/", entry)?;
        }
        Ok(index)
    }

    pub fn root_id(&self) -> NodeId {
        self.root_id
    }

    pub fn node(&self, id: NodeId) -> Option<&VfsNode> {
        self.nodes.get(&id)
    }

    pub fn lookup_path(&self, path: &str) -> Option<NodeId> {
        self.path_index.get(path).copied()
    }

    pub fn mount_root(&self) -> &Path {
        &self.mount_root
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = &VfsNode> {
        self.nodes.values()
    }

    fn insert_controlled_entry(
        &mut self,
        parent: NodeId,
        parent_path: &str,
        entry: &ControlledEntry,
    ) -> Result<NodeId, std::io::Error> {
        let id = NodeId(self.next_id);
        self.next_id += 1;

        let virtual_path = if parent_path == "/" {
            format!("/{}", entry.virtual_name)
        } else {
            format!("{}/{}", parent_path, entry.virtual_name)
        };

        if self.path_index.contains_key(&virtual_path) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("duplicate virtual path: {}", virtual_path),
            ));
        }

        let node = VfsNode {
            id,
            parent: Some(parent),
            name: entry.virtual_name.clone(),
            virtual_path: virtual_path.clone(),
            real_path: entry.real_path.clone(),
            kind: if entry.is_dir {
                VfsNodeKind::Directory
            } else {
                VfsNodeKind::File
            },
            size: if entry.is_virus { 0 } else { entry.file_size },
            is_virus: entry.is_virus,
            exec_type: entry.exec_type,
            extension: entry.extension.clone(),
            is_autorun_target: entry.is_autorun_target,
            is_autorun_inf: entry.is_autorun_inf,
            is_root_shell_script: entry.is_root_shell_script,
            children: Vec::new(),
        };

        self.nodes.insert(id, node);
        self.path_index.insert(virtual_path.clone(), id);
        self.nodes.get_mut(&parent).unwrap().children.push(id);

        for child in &entry.children {
            self.insert_controlled_entry(id, &virtual_path, child)?;
        }

        Ok(id)
    }
}

pub fn node_to_controlled_entry(node: &VfsNode) -> ControlledEntry {
    ControlledEntry {
        real_path: node.real_path.clone(),
        virtual_name: node.name.clone(),
        file_size: node.size,
        is_dir: node.is_dir(),
        is_virus: node.is_virus,
        exec_type: node.exec_type,
        extension: node.extension.clone(),
        is_autorun_target: node.is_autorun_target,
        is_autorun_inf: node.is_autorun_inf,
        is_root_shell_script: node.is_root_shell_script,
        children: vec![],
    }
}
