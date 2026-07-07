//! 受控虚拟文件系统索引。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::policy::evaluate_access;
use crate::types::{blocked_placeholder_bytes, AccessDecision, ControlledEntry, PolicySnapshot};
use crate::vfs::mutation::FsMutation;
use crate::vfs::node::{NodeId, VfsFileView, VfsNode, VfsNodeKind};

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
        snapshot: &PolicySnapshot,
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
                file_view: None,
                first_cluster: Some(2),
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
            index.insert_controlled_entry(root_id, "/", entry, snapshot)?;
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

    pub fn set_first_cluster(
        &mut self,
        virtual_path: &str,
        first_cluster: Option<u32>,
    ) -> Result<(), std::io::Error> {
        let id = self.lookup_path(virtual_path).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "node not found")
        })?;
        self.nodes.get_mut(&id).unwrap().first_cluster = first_cluster;
        Ok(())
    }

    pub fn apply_mutation(&mut self, mutation: &FsMutation) -> Result<(), std::io::Error> {
        match mutation {
            FsMutation::CreateDir { parent, name, chain } => {
                let first_cluster = chain.as_ref().map(|c| c.first_cluster);
                self.create_runtime_node(parent, name, VfsNodeKind::Directory, 0, first_cluster)
            }
            FsMutation::CreateFile {
                parent,
                name,
                size,
                chain,
                ..
            } => {
                let first_cluster = chain.as_ref().map(|c| c.first_cluster);
                self.create_runtime_node(parent, name, VfsNodeKind::File, *size, first_cluster)
            }
            FsMutation::Rename { from, to, .. } => self.rename_path(from, to),
            FsMutation::Delete { virtual_path, .. } => self.delete_path(virtual_path),
            FsMutation::Truncate { virtual_path, len } => {
                let id = self.lookup_path(virtual_path).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "node not found")
                })?;
                self.nodes.get_mut(&id).unwrap().size = *len;
                Ok(())
            }
            FsMutation::RewriteFile {
                virtual_path,
                size,
                chain,
                ..
            } => {
                let id = self.lookup_path(virtual_path).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "node not found")
                })?;
                let node = self.nodes.get_mut(&id).unwrap();
                node.size = *size;
                node.first_cluster = chain.as_ref().map(|c| c.first_cluster);
                Ok(())
            }
            FsMutation::WriteFile { .. } => Ok(()),
        }
    }

    fn create_runtime_node(
        &mut self,
        parent_path: &str,
        name: &str,
        kind: VfsNodeKind,
        size: u64,
        first_cluster: Option<u32>,
    ) -> Result<(), std::io::Error> {
        validate_absolute_path(parent_path)?;
        validate_name(name)?;
        let parent = self.lookup_path(parent_path).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "parent path not found")
        })?;
        if !self.nodes.get(&parent).unwrap().is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "parent is not a directory",
            ));
        }

        let virtual_path = join_virtual_path(parent_path, name);
        if self.path_index.contains_key(&virtual_path) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "virtual path already exists",
            ));
        }

        let id = NodeId(self.next_id);
        self.next_id += 1;
        let real_path = real_path_for_virtual(&self.mount_root, &virtual_path);
        let file_view = if kind == VfsNodeKind::File {
            Some(VfsFileView::RealFile)
        } else {
            None
        };
        self.nodes.insert(
            id,
            VfsNode {
                id,
                parent: Some(parent),
                name: name.to_string(),
                virtual_path: virtual_path.clone(),
                real_path,
                kind,
                size,
                file_view,
                first_cluster,
                is_virus: false,
                exec_type: None,
                extension: extension_for(name),
                is_autorun_target: false,
                is_autorun_inf: false,
                is_root_shell_script: false,
                children: Vec::new(),
            },
        );
        self.path_index.insert(virtual_path, id);
        self.nodes.get_mut(&parent).unwrap().children.push(id);
        Ok(())
    }

    fn rename_path(&mut self, from: &str, to: &str) -> Result<(), std::io::Error> {
        validate_absolute_path(from)?;
        validate_absolute_path(to)?;
        if from == "/" || to == "/" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "cannot rename root",
            ));
        }
        if self.path_index.contains_key(to) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "target path already exists",
            ));
        }

        let id = self.lookup_path(from).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "source path not found")
        })?;
        let (new_parent_path, new_name) = split_parent_name(to)?;
        let new_parent = self.lookup_path(new_parent_path).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "target parent not found")
        })?;
        if !self.nodes.get(&new_parent).unwrap().is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "target parent is not a directory",
            ));
        }

        let old_parent = self.nodes.get(&id).and_then(|node| node.parent).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "source has no parent")
        })?;
        self.nodes
            .get_mut(&old_parent)
            .unwrap()
            .children
            .retain(|child| *child != id);
        self.nodes.get_mut(&new_parent).unwrap().children.push(id);

        let ids = self.collect_subtree_ids(id);
        let old_prefix = from.to_string();
        let new_prefix = to.to_string();
        for node_id in ids {
            let node = self.nodes.get_mut(&node_id).unwrap();
            self.path_index.remove(&node.virtual_path);
            let suffix = node
                .virtual_path
                .strip_prefix(&old_prefix)
                .unwrap_or_default();
            node.virtual_path = format!("{}{}", new_prefix, suffix);
            node.real_path = real_path_for_virtual(&self.mount_root, &node.virtual_path);
            if node_id == id {
                node.parent = Some(new_parent);
                node.name = new_name.to_string();
                node.extension = extension_for(new_name);
            }
            self.path_index.insert(node.virtual_path.clone(), node_id);
        }
        Ok(())
    }

    fn delete_path(&mut self, virtual_path: &str) -> Result<(), std::io::Error> {
        validate_absolute_path(virtual_path)?;
        if virtual_path == "/" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "cannot delete root",
            ));
        }
        let id = self.lookup_path(virtual_path).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "path not found")
        })?;
        if let Some(parent) = self.nodes.get(&id).and_then(|node| node.parent) {
            self.nodes
                .get_mut(&parent)
                .unwrap()
                .children
                .retain(|child| *child != id);
        }
        let mut ids = self.collect_subtree_ids(id);
        ids.sort_by_key(|node_id| std::cmp::Reverse(self.nodes[node_id].virtual_path.len()));
        for node_id in ids {
            if let Some(node) = self.nodes.remove(&node_id) {
                self.path_index.remove(&node.virtual_path);
            }
        }
        Ok(())
    }

    fn collect_subtree_ids(&self, root: NodeId) -> Vec<NodeId> {
        let mut ids = Vec::new();
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            ids.push(id);
            if let Some(node) = self.nodes.get(&id) {
                stack.extend(node.children.iter().copied());
            }
        }
        ids
    }

    fn insert_controlled_entry(
        &mut self,
        parent: NodeId,
        parent_path: &str,
        entry: &ControlledEntry,
        snapshot: &PolicySnapshot,
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

        let decision = if entry.is_dir {
            AccessDecision::Allow
        } else {
            evaluate_access(entry, snapshot)
        };
        let placeholder_size = blocked_placeholder_bytes().len() as u64;
        let (size, file_view) = if entry.is_dir {
            (0, None)
        } else {
            match decision {
                AccessDecision::Allow => (entry.file_size, Some(VfsFileView::RealFile)),
                AccessDecision::Deny(reason) => (
                    placeholder_size,
                    Some(VfsFileView::BlockedPlaceholder {
                        reason,
                        real_size: entry.file_size,
                        placeholder_size,
                    }),
                ),
            }
        };

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
            size,
            file_view,
            first_cluster: None,
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
            self.insert_controlled_entry(id, &virtual_path, child, snapshot)?;
        }

        Ok(id)
    }
}

fn validate_absolute_path(path: &str) -> Result<(), std::io::Error> {
    if !path.starts_with('/') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "virtual path must be absolute",
        ));
    }
    if path.contains("/../") || path.ends_with("/..") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "virtual path escapes root",
        ));
    }
    Ok(())
}

fn validate_name(name: &str) -> Result<(), std::io::Error> {
    if name.is_empty() || name.contains('/') || name == "." || name == ".." {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid node name",
        ));
    }
    Ok(())
}

fn split_parent_name(path: &str) -> Result<(&str, &str), std::io::Error> {
    let trimmed = path.trim_end_matches('/');
    let (parent, name) = trimmed.rsplit_once('/').ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid virtual path")
    })?;
    validate_name(name)?;
    Ok((if parent.is_empty() { "/" } else { parent }, name))
}

fn join_virtual_path(parent: &str, name: &str) -> String {
    if parent == "/" {
        format!("/{}", name)
    } else {
        format!("{}/{}", parent, name)
    }
}

fn real_path_for_virtual(mount_root: &Path, virtual_path: &str) -> PathBuf {
    let mut real = mount_root.to_path_buf();
    for part in virtual_path.split('/').filter(|part| !part.is_empty()) {
        real.push(part);
    }
    real
}

fn extension_for(name: &str) -> String {
    name.rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .unwrap_or_default()
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
