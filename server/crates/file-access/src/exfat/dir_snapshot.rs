use std::collections::HashMap;

use crate::exfat::directory_parser::{parse_entry_sets, ParsedDirectoryEntry};

#[derive(Debug, Clone)]
pub struct DirectorySnapshot {
    pub virtual_path: String,
    entries_by_name: HashMap<String, ParsedDirectoryEntry>,
    entries_by_offset: HashMap<usize, ParsedDirectoryEntry>,
}

impl DirectorySnapshot {
    pub fn parse(
        virtual_path: impl Into<String>,
        data: &[u8],
    ) -> Result<Self, std::io::Error> {
        let entries = parse_entry_sets(data)?;
        let mut entries_by_name = HashMap::new();
        let mut entries_by_offset = HashMap::new();
        for entry in entries {
            entries_by_offset.insert(entry.entry_offset, entry.clone());
            entries_by_name.insert(entry.name.clone(), entry);
        }
        Ok(Self {
            virtual_path: virtual_path.into(),
            entries_by_name,
            entries_by_offset,
        })
    }

    pub fn get_by_name(&self, name: &str) -> Option<&ParsedDirectoryEntry> {
        self.entries_by_name.get(name)
    }

    pub fn get_by_offset(&self, offset: usize) -> Option<&ParsedDirectoryEntry> {
        self.entries_by_offset.get(&offset)
    }

    pub fn entries(&self) -> impl Iterator<Item = &ParsedDirectoryEntry> {
        self.entries_by_name.values()
    }
}
