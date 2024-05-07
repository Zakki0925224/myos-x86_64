use super::fat::{
    dir_entry::{Attribute, EntryType, ShortFileNameEntry},
    FatType, FatVolume,
};
use crate::{
    error::{Error, Result},
    fs::fat::dir_entry::LongFileNameEntry,
};
use alloc::{collections::VecDeque, string::String, vec::Vec};

#[derive(Debug, Clone)]
pub struct FileMetaData {
    pub name: String,
    pub attr: Attribute,
    pub size: usize,
    pub target_cluster_num: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Initramfs {
    fat_volume: Option<FatVolume>,
    root_cluster_num: usize,
    current_cluster_num: usize,
}

impl Initramfs {
    pub const fn new(init_cluster_num: usize) -> Self {
        Self {
            fat_volume: None,
            root_cluster_num: init_cluster_num,
            current_cluster_num: init_cluster_num,
        }
    }

    pub fn init(&mut self, fat_volume: FatVolume) -> Result<()> {
        if fat_volume.fat_type() != FatType::Fat32 {
            return Err(Error::Failed("FAT 12 or FAT 16 are not supported"));
        }

        self.root_cluster_num = fat_volume.root_cluster_num();
        self.current_cluster_num = fat_volume.root_cluster_num();

        self.fat_volume = Some(fat_volume);
        Ok(())
    }

    pub fn cd(&mut self, dir_name: &str) -> Result<()> {
        let files = self.scan_current_dir();
        let dir = files
            .iter()
            .find(|f| f.attr == Attribute::Directory && f.name.trim() == dir_name);

        if dir.is_none() {
            return Err(Error::Failed("The directory does not exist"));
        }

        let cluster_num = dir.unwrap().target_cluster_num;

        self.current_cluster_num = if cluster_num != 0 {
            cluster_num
        } else {
            self.root_cluster_num
        };

        Ok(())
    }

    pub fn get_file(&self, file_name: &str) -> Result<(FileMetaData, Vec<u8>)> {
        let files = self.scan_current_dir();
        let file = files
            .iter()
            .find(|f| f.attr == Attribute::Archive && f.name.trim() == file_name);

        if file.is_none() {
            return Err(Error::Failed("The file does not exist"));
        }

        let fat_volume = self.fat_volume.as_ref().unwrap();
        let dir_entries = fat_volume.read_chained_dir_entries(file.unwrap().target_cluster_num);
        let mut bytes: Vec<u8> = dir_entries.iter().flat_map(|de| de.raw()).collect();
        bytes.resize(file.unwrap().size, 0);

        Ok((file.unwrap().clone(), bytes))
    }

    pub fn scan_current_dir(&self) -> Vec<FileMetaData> {
        let mut files = Vec::new();

        if self.fat_volume.is_none() {
            return files;
        }

        let mut lf_name_buf = VecDeque::new();
        let fat_volume = self.fat_volume.as_ref().unwrap();
        let dir_entries = fat_volume.read_chained_dir_entries(self.current_cluster_num);

        for i in 0..dir_entries.len() {
            let dir_entry = dir_entries[i];
            let entry_type = dir_entry.entry_type();
            let file_attr = dir_entry.attr();

            // end of not null directories
            if entry_type == EntryType::Null && file_attr.is_none() {
                break;
            }

            // long file name entry
            if let (Some(lf_name), Some(lfn_entry_index)) =
                (dir_entry.lf_name(), dir_entry.lfn_entry_index())
            {
                if lfn_entry_index >= 1 {
                    lf_name_buf.push_front(lf_name);
                    continue;
                }
            }

            match file_attr {
                Some(attr) => match attr {
                    Attribute::Archive | Attribute::Directory => {
                        let file_name = if lf_name_buf.len() > 0 {
                            lf_name_buf.iter().fold(String::new(), |acc, s| acc + s)
                        } else {
                            dir_entry.sf_name().unwrap()
                        };

                        let file = FileMetaData {
                            name: file_name,
                            attr,
                            size: dir_entry.file_size(),
                            target_cluster_num: dir_entry.first_cluster_num(),
                        };

                        files.push(file);
                        lf_name_buf.clear();
                    }
                    _ => (),
                },
                None => (),
            }
        }

        files
    }

    pub fn reset_cwd(&mut self) {
        self.current_cluster_num = self.root_cluster_num;
    }
}
