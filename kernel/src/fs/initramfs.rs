use crate::arch::addr::VirtualAddress;

use super::fat::{FatType, FatVolume};

pub fn init(initramfs_start_virt_addr: VirtualAddress) {
    let fat_volume = FatVolume::new(initramfs_start_virt_addr);
    if fat_volume.fat_type() != FatType::Fat32 {
        panic!("FAT12 or FAT16 are not supported");
    }

    fat_volume.debug();
}
