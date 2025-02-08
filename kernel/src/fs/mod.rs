use crate::{arch::addr::VirtualAddress, fs::vfs::FileSystem};
use common::kernel_config::KernelConfig;
use fat::{volume::FatVolume, Fat};
use log::{error, info};

pub mod exec;
pub mod fat;
pub mod file;
pub mod path;
pub mod vfs;

pub fn init(initramfs_virt_addr: VirtualAddress, kernel_config: &KernelConfig) {
    if let Err(err) = vfs::init() {
        error!("fs: Failed to initialized VFS: {:?}", err);
    }
    info!("fs: VFS initialized");

    let fat_volume = FatVolume::new(initramfs_virt_addr);
    let fat_fs = Fat::new(fat_volume);

    if let Err(err) = vfs::mount_fs(&"/mnt/initramfs".into(), FileSystem::Fat(fat_fs)) {
        error!("fs: Failed to mount initramfs to VFS: {:?}", err);
    }
    info!("fs: Mounted initramfs to VFS");

    let dirname = kernel_config.init_cwd_path.into();
    if let Err(err) = vfs::chdir(&dirname) {
        error!("fs: Failed to chdir to {}: {:?}", dirname, err);
    }
}
