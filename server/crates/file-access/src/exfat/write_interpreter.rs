//! Interpret sector writes using runtime exFAT ownership.

use crate::exfat::sector_owner::SectorOwner;
use crate::exfat::transaction::{PendingTransaction, TransactionWrite};

#[derive(Debug, Default, Clone)]
pub struct WriteInterpreter;

impl WriteInterpreter {
    pub fn new() -> Self {
        Self
    }

    pub fn record_sector_write(
        &self,
        tx: &mut PendingTransaction,
        sector: u64,
        owner: SectorOwner,
        data: &[u8],
    ) -> Result<(), std::io::Error> {
        match owner {
            SectorOwner::Mbr
            | SectorOwner::BootRegion
            | SectorOwner::BackupBootRegion
            | SectorOwner::Reserved
            | SectorOwner::OutOfRange => Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "禁止修改 exFAT 关键区域",
            )),
            SectorOwner::Unknown => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "无法解释未知 sector owner 的写入",
            )),
            SectorOwner::Fat => {
                tx.record_write(TransactionWrite::Fat {
                    sector,
                    data: data.to_vec(),
                });
                Ok(())
            }
            SectorOwner::AllocationBitmap => {
                tx.record_write(TransactionWrite::Bitmap {
                    sector,
                    data: data.to_vec(),
                });
                Ok(())
            }
            SectorOwner::RootDirectory | SectorOwner::DirectoryData { .. } => {
                tx.record_write(TransactionWrite::Directory {
                    sector,
                    owner,
                    data: data.to_vec(),
                });
                Ok(())
            }
            SectorOwner::FileData { .. } | SectorOwner::AllocatedZero { .. } => {
                tx.record_write(TransactionWrite::FileData {
                    sector,
                    owner,
                    data: data.to_vec(),
                });
                Ok(())
            }
            SectorOwner::FreeCluster { .. } => {
                tx.record_write(TransactionWrite::FreeCluster {
                    sector,
                    owner,
                    data: data.to_vec(),
                });
                Ok(())
            }
            SectorOwner::UpcaseTable => Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "禁止修改 exFAT upcase table",
            )),
        }
    }
}
