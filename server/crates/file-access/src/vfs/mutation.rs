#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClusterChain {
    pub first_cluster: u32,
    pub clusters: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDataPatch {
    pub virtual_path: String,
    pub offset: u64,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsMutation {
    CreateFile {
        parent: String,
        name: String,
        size: u64,
        valid_data_len: u64,
        chain: Option<ClusterChain>,
        data_patches: Vec<FileDataPatch>,
    },
    CreateDir {
        parent: String,
        name: String,
        chain: Option<ClusterChain>,
    },
    WriteFile {
        virtual_path: String,
        offset: u64,
        data: Vec<u8>,
    },
    Truncate {
        virtual_path: String,
        len: u64,
    },
    Rename {
        from: String,
        to: String,
        kind: NodeKind,
    },
    Delete {
        virtual_path: String,
        kind: NodeKind,
    },
}
