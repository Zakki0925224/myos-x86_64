#[derive(Debug)]
pub struct BootConfig<'a> {
    pub kernel_path: &'a str,
    pub initramfs_path: &'a str,
    pub resolution: Option<(usize, usize)>,
}

pub const DEFAULT_BOOT_CONFIG: BootConfig = BootConfig {
    kernel_path: "\\EFI\\myos\\kernel.elf",
    initramfs_path: "initramfs.img",
    resolution: Some((800, 600)),
};
