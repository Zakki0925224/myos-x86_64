use crate::arch::addr::VirtualAddress;

use super::fat::FatVolume;

pub fn init(initramfs_start_virt_addr: VirtualAddress) {
    let fat_volume = FatVolume::new(initramfs_start_virt_addr);
    fat_volume.debug();
}
