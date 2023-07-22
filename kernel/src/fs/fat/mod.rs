use crate::arch::addr::{Address, VirtualAddress};

use self::{
    boot_sector::{BootSector, BootSectorOtherField},
    fs_info_sector::FsInfoSector,
};

pub mod boot_sector;
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
}
