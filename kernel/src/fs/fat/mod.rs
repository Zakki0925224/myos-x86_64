use core::mem::size_of;

use self::{boot_sector::BootSector, dir_entry::DirectoryEntry, fs_info_sector::FsInfoSector};
use crate::{
    arch::addr::{Address, VirtualAddress},
    println,
};

pub mod boot_sector;
pub mod dir_entry;
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
        return Self {
            volume_start_virt_addr,
        };
    }

    pub fn read_boot_sector(&self) -> BootSector {
        return self.volume_start_virt_addr.read_volatile();
    }

    pub fn read_fs_info_sector(&self) -> Option<FsInfoSector> {
        return match self.fat_type() {
            FatType::Fat32 => {
                let boot_sector = self.read_boot_sector();
                let fat32_other_field = unsafe { boot_sector.other_field.fat32 };

                Some(
                    self.volume_start_virt_addr
                        .offset(
                            fat32_other_field.fs_info_sector_num() * boot_sector.bytes_per_sector(),
                        )
                        .read_volatile(),
                )
            }
            _ => None,
        };
    }

    pub fn fat_type(&self) -> FatType {
        let boot_sector = self.read_boot_sector();

        return boot_sector.fat_type();
    }

    pub fn read_root_dir_entry(&self) -> DirectoryEntry {
        let boot_sector = self.read_boot_sector();
        let root_dir_start_sector = boot_sector.root_dir_start_sector();
        let bytes_per_sector = boot_sector.bytes_per_sector();

        let offset = root_dir_start_sector * bytes_per_sector;
        println!(
            "root dir start sector {} * bytes per sector {}",
            root_dir_start_sector, bytes_per_sector
        );

        return self.volume_start_virt_addr.offset(offset).read_volatile();
    }

    pub fn read_dir_entry(&self, entry_num: usize) -> Option<DirectoryEntry> {
        // if entry_num > self.dir_entry_max_num() {
        //     return None;
        // }

        let boot_sector = self.read_boot_sector();
        let data_start_sector = boot_sector.data_start_sector();
        let bytes_per_sector = boot_sector.bytes_per_sector();
        let sectors_per_cluster = boot_sector.sectors_per_cluster();

        let offset = (data_start_sector + entry_num * sectors_per_cluster) * bytes_per_sector;
        println!("offset: 0x{:x}", offset);

        return Some(self.volume_start_virt_addr.offset(offset).read_volatile());
    }
}
