use super::fat::{
    dir_entry::{Attribute, EntryType, ShortFileNameEntry},
    FatType, FatVolume,
};
use crate::{
    arch::addr::VirtualAddress,
    error::Result,
    fs::fat::dir_entry::LongFileNameEntry,
    util::mutex::{Mutex, MutexError},
};
use alloc::{collections::VecDeque, string::String, vec::Vec};
use log::{error, info};

const PATH_SEPARATOR: &str = "/";

static mut INITRAMFS: Mutex<Initramfs> = Mutex::new(Initramfs::new(2));

#[derive(Debug, Clone)]
pub struct FileMetaData {
    pub name: String,
    pub attr: Attribute,
    pub size: usize,
    pub target_cluster_num: usize,
}

struct Initramfs {
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

    pub fn init(&mut self, fat_volume: FatVolume) {
        if fat_volume.fat_type() != FatType::Fat32 {
            error!("initramfs: FAT12 or FAT16 are not supported");
            return;
        }

        self.root_cluster_num = fat_volume.root_cluster_num();
        self.current_cluster_num = fat_volume.root_cluster_num();

        info!("initramfs: Initialized");

        self.fat_volume = Some(fat_volume);
    }

    pub fn ls(&self) {
        let files = self.scan_current_dir();
        for f in files {
            info!("{:?}", f);
        }
    }

    pub fn cd(&mut self, dir_name: &str) {
        let files = self.scan_current_dir();
        let dir = files
            .iter()
            .find(|f| f.attr == Attribute::Directory && f.name.trim() == dir_name);
        info!("{:?}", dir);

        if dir.is_none() {
            error!("initramfs: The directory \"{}\" does not exist", dir_name);
            return;
        }

        let cluster_num = dir.unwrap().target_cluster_num;

        self.current_cluster_num = if cluster_num != 0 {
            cluster_num
        } else {
            self.root_cluster_num
        };
    }

    pub fn cat(&self, file_name: &str) {
        let (_, data) = match self.get_file(file_name) {
            Some((f, d)) => (f, d),
            None => {
                error!("initramfs: The file \"{}\" does not exist", file_name);
                return;
            }
        };

        info!("\n{}", String::from_utf8_lossy(&data));
    }

    pub fn get_file(&self, file_name: &str) -> Option<(FileMetaData, Vec<u8>)> {
        let files = self.scan_current_dir();
        let file = files
            .iter()
            .find(|f| f.attr == Attribute::Archive && f.name.trim() == file_name);

        if file.is_none() {
            return None;
        }

        let fat_volume = self.fat_volume.as_ref().unwrap();
        let dir_entries = fat_volume.read_chained_dir_entries(file.unwrap().target_cluster_num);
        let mut bytes: Vec<u8> = dir_entries.iter().flat_map(|de| de.raw()).collect();
        bytes.resize(file.unwrap().size, 0);

        Some((file.unwrap().clone(), bytes))
    }

    fn scan_current_dir(&self) -> Vec<FileMetaData> {
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
}

pub fn init(initramfs_start_virt_addr: VirtualAddress) {
    let fat_volume = FatVolume::new(initramfs_start_virt_addr);
    unsafe { INITRAMFS.try_lock() }.unwrap().init(fat_volume);
}

pub fn get_file(file_name: &str) -> Result<Option<(FileMetaData, Vec<u8>)>> {
    if let Ok(initramfs) = unsafe { INITRAMFS.try_lock() } {
        return Ok(initramfs.get_file(file_name));
    }

    Err(MutexError::Locked.into())
}

pub fn ls() -> Result<()> {
    if let Ok(initramfs) = unsafe { INITRAMFS.try_lock() } {
        initramfs.ls();
        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn cd(dir_name: &str) -> Result<()> {
    if let Ok(mut initramfs) = unsafe { INITRAMFS.try_lock() } {
        initramfs.cd(dir_name);
        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn cat(file_name: &str) -> Result<()> {
    if let Ok(initramfs) = unsafe { INITRAMFS.try_lock() } {
        initramfs.cat(file_name);
        return Ok(());
    }

    Err(MutexError::Locked.into())
}
