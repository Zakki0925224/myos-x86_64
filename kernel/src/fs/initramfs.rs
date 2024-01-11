use crate::{arch::addr::VirtualAddress, fs::fat::dir_entry::LongFileNameEntry, println};

use super::fat::{
    dir_entry::{Attribute, EntryType, ShortFileNameEntry},
    FatType, FatVolume,
};
use alloc::{collections::VecDeque, string::String, vec::Vec};
use lazy_static::lazy_static;
use log::{error, info};
use spin::Mutex;

const PATH_SEPARATOR: &str = "/";

lazy_static! {
    static ref INITRAMFS: Mutex<Initramfs> = Mutex::new(Initramfs::new(2));
}

#[derive(Debug)]
struct File {
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
    pub fn new(init_cluster_num: usize) -> Self {
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
        info!(
            "initramfs: Boot sector: {:?}",
            fat_volume.read_boot_sector()
        );
        info!(
            "initramfs: FS info sector: {:?}",
            fat_volume.read_fs_info_sector()
        );

        self.fat_volume = Some(fat_volume);
    }

    pub fn ls(&self) {
        let files = self.scan_current_dir();
        for f in files {
            println!("{:?}", f);
        }
    }

    pub fn cd(&mut self, dir_name: &str) {
        let files = self.scan_current_dir();
        let dir = files
            .iter()
            .find(|f| f.attr == Attribute::Directory && f.name.trim() == dir_name);
        println!("{:?}", dir);

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
        let files = self.scan_current_dir();
        let file = files
            .iter()
            .find(|f| f.attr == Attribute::Archive && f.name.trim() == file_name);

        if file.is_none() {
            error!("initramfs: The file \"{}\" does not exist", file_name);
            return;
        }

        let fat_volume = self.fat_volume.as_ref().unwrap();
        let dir_entries = fat_volume.read_chained_dir_entries(file.unwrap().target_cluster_num);
        let bytes: Vec<u8> = dir_entries.iter().flat_map(|de| de.raw()).collect();

        println!("{}", String::from_utf8_lossy(&bytes[..file.unwrap().size]));
    }

    fn scan_current_dir(&self) -> Vec<File> {
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

                        let file = File {
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
    INITRAMFS.try_lock().unwrap().init(fat_volume);
}

pub fn ls() {
    if let Some(initramfs) = INITRAMFS.try_lock() {
        initramfs.ls();
    }
}

pub fn cd(dir_path: &str) {
    if let Some(mut initramfs) = INITRAMFS.try_lock() {
        initramfs.cd(dir_path);
    }
}

pub fn cat(file_path: &str) {
    if let Some(initramfs) = INITRAMFS.try_lock() {
        initramfs.cat(file_path);
    }
}
