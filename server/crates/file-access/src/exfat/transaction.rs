//! Runtime exFAT write transactions.

use crate::exfat::sector_owner::SectorOwner;
use crate::vfs::mutation::FsMutation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionWrite {
    Fat {
        sector: u64,
        data: Vec<u8>,
    },
    Bitmap {
        sector: u64,
        data: Vec<u8>,
    },
    Directory {
        sector: u64,
        owner: SectorOwner,
        data: Vec<u8>,
    },
    FileData {
        sector: u64,
        owner: SectorOwner,
        data: Vec<u8>,
    },
    FreeCluster {
        sector: u64,
        owner: SectorOwner,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingReason {
    WaitingForDirectoryData { sector: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionError {
    UnknownDirectoryOwner {
        sector: u64,
    },
    MissingDirectoryImage {
        parent: String,
    },
    DirectoryWriteBeforeStart {
        parent: String,
        sector: u64,
    },
    UnsupportedDirectoryRewrite {
        parent: String,
    },
    BlockedPlaceholderRewrite {
        virtual_path: String,
    },
    UnresolvedClusterChain {
        first_cluster: u32,
        data_length: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommittedMetadataUpdate {
    Sector { sector: u64, data: Vec<u8> },
}

#[derive(Debug, Clone)]
pub struct ResolvedTransaction {
    pub mutations: Vec<FsMutation>,
}

#[derive(Debug, Clone)]
pub enum ResolveStatus {
    Complete(ResolvedTransaction),
    Incomplete(PendingReason),
    Invalid(TransactionError),
}

impl TransactionWrite {
    pub fn sector(&self) -> u64 {
        match self {
            TransactionWrite::Fat { sector, .. }
            | TransactionWrite::Bitmap { sector, .. }
            | TransactionWrite::Directory { sector, .. }
            | TransactionWrite::FileData { sector, .. }
            | TransactionWrite::FreeCluster { sector, .. } => *sector,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingTransaction {
    id: u64,
    writes: Vec<TransactionWrite>,
}

impl PendingTransaction {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            writes: Vec::new(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn writes(&self) -> &[TransactionWrite] {
        &self.writes
    }

    pub fn record_write(&mut self, write: TransactionWrite) {
        self.writes.push(write);
    }

    pub fn retain_deferred_data_writes(&mut self) {
        self.writes.retain(|write| {
            matches!(
                write,
                TransactionWrite::FileData { .. } | TransactionWrite::FreeCluster { .. }
            )
        });
    }

    pub fn is_empty(&self) -> bool {
        self.writes.is_empty()
    }
}
