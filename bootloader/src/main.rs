#![no_std]
#![no_main]
#![feature(abi_efiapi)]

mod config;

#[macro_use]
extern crate log;

use core::{slice::{from_raw_parts_mut, self}, mem};
use common::boot_info::{BootInfo, GraphicInfo};
use uefi::{prelude::*, proto::{media::{file::*, fs::SimpleFileSystem}, console::gop::GraphicsOutput}, CStr16, table::boot::*};
use xmas_elf::{ElfFile, program::Type};

use crate::config::DEFAULT_BOOT_CONFIG;

const PAGE_SIZE: usize = 0x1000;
const KERNEL_BASE_ADDR: usize = 0x100000;

#[entry]
fn efi_main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status
{
    uefi_services::init(&mut system_table).unwrap();
    let bs = system_table.boot_services();

    info!("Running bootloader...");

    // load config
    let config = DEFAULT_BOOT_CONFIG;

    // graphic info
    let graphic_info = init_graphic(bs, config.resolution);

    // read kernel.elf
    let mut f = open_file(bs, config.kernel_path);
    let buf = load_file_to_mem(bs, &mut f, KERNEL_BASE_ADDR);
    let kernel = ElfFile::new(buf).expect("Failed to parse ELF file");
    //copy_load_segs(&kernel);
    let kernel_entry_point_addr = kernel.header.pt2.entry_point();
    info!("Kernel entry point: 0x{:x}", kernel_entry_point_addr);

    let bi = BootInfo
    {
        graphic_info: graphic_info
    };

    info!("{:?}", bi);

    // https://github.com/uchan-nos/os-from-zero/issues/41
    // not changed when add flag "-z separate-code"
    jump_to_entry(kernel_entry_point_addr - 0x1000, &bi, config.kernel_stack_addr, config.kernel_stack_size);

    return Status::SUCCESS;
}

fn open_file(bs: &BootServices, path: &str) -> RegularFile
{
    info!("Opening file: \"{}\"", path);

    let fs = bs
        .locate_protocol::<SimpleFileSystem>()
        .expect("Failed to get FileSystem");

    let fs = unsafe { &mut *fs.get() };
    let mut buf = [0; 256];
    let path = CStr16::from_str_with_buf(path, &mut buf).expect("Failed to convert path to ucs-2");
    let mut root = fs.open_volume().expect("Failed to open volume");
    let handle =root
        .open(path, FileMode::Read, FileAttribute::empty())
        .expect("Failed to open file");

    match handle.into_type().expect("Failed to into_type")
    {
        FileType::Regular(r) => return r,
        _ => panic!("Invalid file type")
    }
}

fn load_file_to_mem(bs: &BootServices, file: &mut RegularFile, addr: usize) -> &'static mut [u8]
{
    let mut info_buf = [0; 256];

    let info = file
        .get_info::<FileInfo>(&mut info_buf)
        .expect("Failed to get file info");

    let pages = (info.file_size() as usize + 0xfff) / PAGE_SIZE;
    let mem_start = bs
        .allocate_pages(AllocateType::Address(addr), MemoryType::LOADER_DATA, pages)
        .expect("Failed to allocate pages");
    let buf = unsafe { from_raw_parts_mut(mem_start as *mut u8, pages * PAGE_SIZE) };
    let len = file.read(buf).expect("Failed to read file");

    info!("Loaded {}bytes at 0x{:x}", len, mem_start);

    return &mut buf[..len];
}

fn copy_load_segs(elf: &ElfFile)
{
    let mut i = 0;

    loop
    {
        let header = elf.program_header(i);
        if let Err(_) = header
        {
            break;
        }

        match header.unwrap().get_type()
        {
            Err(_) => break,
            Ok(ht) =>
            {
                if ht != Type::Load
                {
                    i += 1;
                    continue;
                }
            }
        }

        let vaddr = header.unwrap().virtual_addr();
        let offset = header.unwrap().offset() as usize;
        let fsize = header.unwrap().file_size() as usize;
        let msize = header.unwrap().mem_size() as usize;
        let dest = unsafe { slice::from_raw_parts_mut(vaddr as *mut u8, msize) };
        dest[..fsize].copy_from_slice(&elf.input[offset..offset + fsize]);
        dest[fsize..].fill(0);

        i += 1;
    }
}

fn init_graphic(bs: &BootServices, resolution: Option<(usize, usize)>) -> GraphicInfo
{
    let gop = bs
        .locate_protocol::<GraphicsOutput>()
        .expect("Failed to get GraphicsOutput");

    let gop = unsafe { &mut *gop.get() };

    if let Some(resolution) = resolution
    {
        let mode = gop.modes()
            .find(|mode| mode.info().resolution() == resolution)
            .expect("Graphic mode not found");

        info!("Switching graphic mode...");
        gop.set_mode(&mode).expect("Failed to set graphic mode");
    }

    let gi = GraphicInfo
    {
        mode: gop.current_mode_info(),
        framebuf_addr: gop.frame_buffer().as_mut_ptr() as u64,
        framebuf_size: gop.frame_buffer().size() as u64
    };

    return gi;
}

fn jump_to_entry(entry_base_addr: u64, bi: &BootInfo, stack_addr: u64, stack_size: u64)
{
    let stacktop = stack_addr + stack_size * PAGE_SIZE as u64;
    let entry_point: extern "C" fn(&BootInfo) = unsafe { mem::transmute(entry_base_addr as *const u64) };
    info!("Entering kernel (0x{:x})...", entry_base_addr);
    entry_point(bi);
    info!("Leaved kernel");
}