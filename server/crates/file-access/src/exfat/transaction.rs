//! Runtime exFAT write transactions.

use crate::exfat::sector_owner::SectorOwner;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationCommitState {
    PendingWrite,
    ParsedMutation,
    PolicyChecked,
    RealFsCommitting,
    RealFsCommitted,
    RuntimeCommitted,
    Failed,
    RebuildRuntimeIfNeeded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionWrite {
    Fat { sector: u64, data: Vec<u8> },
    Bitmap { sector: u64, data: Vec<u8> },
    Directory { sector: u64, owner: SectorOwner, data: Vec<u8> },
    FileData { sector: u64, owner: SectorOwner, data: Vec<u8> },
    FreeCluster { sector: u64, owner: SectorOwner, data: Vec<u8> },
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
    state: MutationCommitState,
    writes: Vec<TransactionWrite>,
}

impl PendingTransaction {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            state: MutationCommitState::PendingWrite,
            writes: Vec::new(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn state(&self) -> MutationCommitState {
        self.state
    }

    pub fn writes(&self) -> &[TransactionWrite] {
        &self.writes
    }

    pub fn record_write(&mut self, write: TransactionWrite) {
        self.writes.push(write);
    }

    pub fn transition(&mut self, state: MutationCommitState) {
        self.state = state;
    }

    pub fn clear(&mut self) {
        self.writes.clear();
        self.state = MutationCommitState::PendingWrite;
    }

    pub fn retain_deferred_data_writes(&mut self) {
        self.writes.retain(|write| {
            matches!(
                write,
                TransactionWrite::FileData { .. } | TransactionWrite::FreeCluster { .. }
            )
        });
        self.state = MutationCommitState::PendingWrite;
    }

    pub fn is_empty(&self) -> bool {
        self.writes.is_empty()
    }
}
