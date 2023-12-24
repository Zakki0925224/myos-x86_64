use core::mem::size_of;

use self::{boot_sector::BootSector, dir_entry::DirectoryEntry, fs_info_sector::FsInfoSector};
use crate::{arch::addr::VirtualAddress, println};

pub mod boot_sector;
pub mod dir_entry;
pub mod file_allocation_table;
pub mod fs_info_sector;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FatType {
    Fat12,
    Fat16,
    Fat32,
}

pub struct FatVolume {
    volume_start_virt_addr: VirtualAddress,
}

impl FatVolume {
    pub fn new(volume_start_virt_addr: VirtualAddress) -> Self {
        Self {
            volume_start_virt_addr,
        }
    }

    pub fn read_boot_sector(&self) -> BootSector {
        self.volume_start_virt_addr.read_volatile()
    }

    pub fn read_fs_info_sector(&self) -> Option<FsInfoSector> {
        match self.fat_type() {
            FatType::Fat32 => {
                let boot_sector = self.read_boot_sector();
                let fat32_other_field = match boot_sector.fat32_other_field() {
                    Some(f) => f,
                    None => return None,
                };

                Some(
                    self.volume_start_virt_addr
                        .offset(
                            fat32_other_field.fs_info_sector_num() * boot_sector.bytes_per_sector(),
                        )
                        .read_volatile(),
                )
            }
            _ => None,
        }
    }

    pub fn fat_type(&self) -> FatType {
        let boot_sector = self.read_boot_sector();
        boot_sector.fat_type()
    }

    pub fn max_dir_entry_num(&self) -> usize {
        let boot_sector = self.read_boot_sector();
        boot_sector.data_clusters()
            * boot_sector.sectors_per_cluster()
            * boot_sector.bytes_per_sector()
            / size_of::<DirectoryEntry>()
    }

    pub fn read_dir_entry(&self, entry_num: usize) -> Option<DirectoryEntry> {
        if entry_num >= self.max_dir_entry_num() {
            return None;
        }

        let boot_sector = self.read_boot_sector();
        let data_area_start_offset =
            boot_sector.data_start_sector() * boot_sector.bytes_per_sector();
        let target_dir_entry_offset =
            data_area_start_offset + entry_num * size_of::<DirectoryEntry>();
        Some(
            self.volume_start_virt_addr
                .offset(target_dir_entry_offset)
                .read_volatile(),
        )
    }

    // TODO: get root dir entry num
    //pub fn read_root_dir_entry(&self) -> DirectoryEntry {}

    pub fn debug(&self) {
        //println!("{:?}", self.read_boot_sector());
        //println!("{:?}", self.read_fs_info_sector());
        println!("max dir entry num: {}", self.max_dir_entry_num());
        for i in 0..self.max_dir_entry_num() {
            let dir_entry = self.read_dir_entry(i).unwrap();
            if dir_entry.attr().is_none() {
                continue;
            }

            println!(
                "{}: name: {:?}, attr: {:?}, type: {:?}, first_cluster: {}",
                i,
                dir_entry.name(),
                dir_entry.attr(),
                dir_entry.entry_type(),
                dir_entry.first_cluster_num()
            );
        }
    }
}
