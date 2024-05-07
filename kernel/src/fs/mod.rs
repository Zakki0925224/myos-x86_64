use log::{error, info};

use crate::{
    arch::addr::VirtualAddress,
    fs::{fat::FatVolume, initramfs::Initramfs, vfs::FileSystem},
};

pub mod exec;
pub mod fat;
pub mod initramfs;
pub mod vfs;

pub fn init(initramfs_virt_addr: VirtualAddress) {
    if let Err(err) = vfs::init() {
        error!("fs: Failed to initialized VFS: {:?}", err);
    }
    info!("fs: Initialized VFS");

    let fat_volume = FatVolume::new(initramfs_virt_addr);
    let mut initramfs = Initramfs::new(2);

    if let Err(err) = initramfs.init(fat_volume) {
        error!("fs: Failed to initialized initramfs: {:?}", err);
    }
    info!("fs: Initialized initramfs");

    if let Err(err) = vfs::mount("/mnt/initramfs", FileSystem::Initramfs(initramfs)) {
        error!("fs: Failed to mount initramfs to VFS: {:?}", err);
    }
    info!("fs: Mounted initramfs to VFS");
}
