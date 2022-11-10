#[derive(Debug)]
pub struct BootConfig<'a>
{
    pub kernel_stack_addr: u64,
    pub kernel_stack_size: u64,
    pub kernel_path: &'a str,
    pub resolution: Option<(usize, usize)>,
}

pub const DEFAULT_BOOT_CONFIG: BootConfig = BootConfig {
    kernel_stack_addr: 0xffff_ff01_0000_0000,
    kernel_stack_size: 512,
    kernel_path: "\\EFI\\myos\\kernel.elf",
    resolution: Some((800, 600)),
};
