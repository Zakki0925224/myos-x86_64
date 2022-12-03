#![no_std]
#![no_main]
#![feature(abi_efiapi)]

mod config;

#[macro_use]
extern crate log;

#[macro_use]
extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use common::{boot_info::BootInfo, graphic_info::{self, GraphicInfo}, mem_desc::{self, UEFI_PAGE_SIZE}};
use core::{mem, slice::from_raw_parts_mut};
use uefi::{prelude::*, proto::{console::gop::{GraphicsOutput, PixelFormat}, media::file::*}, table::boot::*, CStr16};
use xmas_elf::{program, ElfFile};

use crate::config::DEFAULT_BOOT_CONFIG;

#[entry]
fn efi_main(handle: Handle, mut st: SystemTable<Boot>) -> Status
{
    uefi_services::init(&mut st).unwrap();
    let bs = st.boot_services();

    info!("Running bootloader...");

    // load config
    let config = DEFAULT_BOOT_CONFIG;

    // graphic info
    let graphic_info = init_graphic(bs, config.resolution);
    info!("{:?}", graphic_info);

    // load kernel
    let kernel_entry_point_addr = load_elf(bs, handle, config.kernel_path);
    info!("Kernel entry point: 0x{:x}", kernel_entry_point_addr);

    // get memory map
    let mmap_size = bs.memory_map_size().map_size;
    let mmap_buf = Box::leak(vec![0; mmap_size * 2].into_boxed_slice());

    // exit boot service
    info!("Exit boot services");
    let mut mem_map = Vec::with_capacity(128);

    let (_rt, mmap_iter) = st.exit_boot_services(handle, mmap_buf).unwrap();

    for desc in mmap_iter
    {
        let ty = convert_mem_type(desc.ty);
        let phys_start = desc.phys_start;
        let virt_start = desc.virt_start;
        let page_cnt = desc.page_count;
        let attr = convert_mem_attr(desc.att);

        mem_map.push(mem_desc::MemoryDescriptor { ty, phys_start, virt_start, page_cnt, attr });
    }

    let mem_map_len = mem_map.len();
    let bi = BootInfo::new(mem_map.as_slice(), mem_map_len, graphic_info);

    jump_to_entry(kernel_entry_point_addr, &bi, config.kernel_stack_addr, config.kernel_stack_size);

    return Status::SUCCESS;
}

fn load_elf(bs: &BootServices, image: Handle, path: &str) -> u64
{
    // open file
    info!("Opening file: \"{}\"", path);
    let root = bs.get_image_file_system(image).unwrap();
    let mut root = unsafe { &mut *root.interface.get() }.open_volume().unwrap();
    let mut buf = [0; 256];
    let path = CStr16::from_str_with_buf(path, &mut buf).unwrap();
    let file =
        root.open(path, FileMode::Read, FileAttribute::empty()).unwrap().into_type().unwrap();

    let mut file = match file
    {
        FileType::Regular(file) => file,
        FileType::Dir(_) => panic!("Not file: \"{}\"", path),
    };

    let file_info = file.get_boxed_info::<FileInfo>().unwrap();
    let file_size = file_info.file_size() as usize;
    let mut buf = vec![0; file_size];

    file.read(&mut buf).unwrap();

    // load elf
    let elf = ElfFile::new(&buf).unwrap();

    let mut dest_start = usize::MAX;
    let mut dest_end = 0;

    for p in elf.program_iter()
    {
        if p.get_type().unwrap() != program::Type::Load
        {
            continue;
        }

        dest_start = dest_start.min(p.virtual_addr() as usize);
        dest_end = dest_end.max((p.virtual_addr() + p.mem_size()) as usize);
    }

    let pages = (dest_end - dest_start + UEFI_PAGE_SIZE - 1) / UEFI_PAGE_SIZE;
    bs.allocate_pages(AllocateType::Address(dest_start), MemoryType::LOADER_DATA, pages).unwrap();

    for p in elf.program_iter()
    {
        if p.get_type().unwrap() != program::Type::Load
        {
            continue;
        }

        let offset = p.offset() as usize;
        let file_size = p.file_size() as usize;
        let mem_size = p.mem_size() as usize;
        let dest = unsafe { from_raw_parts_mut(p.virtual_addr() as *mut u8, mem_size) };
        dest[..file_size].copy_from_slice(&buf[offset..offset + file_size]);
        dest[file_size..].fill(0);
    }

    return elf.header.pt2.entry_point();
}

fn init_graphic(bs: &BootServices, resolution: Option<(usize, usize)>) -> GraphicInfo
{
    let gop = bs.locate_protocol::<GraphicsOutput>().unwrap();
    let gop = unsafe { &mut *gop.get() };

    if let Some(resolution) = resolution
    {
        let mode = gop.modes().find(|mode| mode.info().resolution() == resolution).unwrap();

        info!("Switching graphic mode...");
        gop.set_mode(&mode).unwrap();
    }

    let mode_info = gop.current_mode_info();
    let res = mode_info.resolution();

    let gi = GraphicInfo {
        resolution: (res.0 as u32, res.1 as u32),
        format: convert_pixel_format(mode_info.pixel_format()),
        stride: mode_info.stride() as u32,
        framebuf_addr: gop.frame_buffer().as_mut_ptr() as u64,
        framebuf_size: gop.frame_buffer().size() as u64,
    };

    return gi;
}

fn convert_pixel_format(pixel_format: PixelFormat) -> graphic_info::PixelFormat
{
    return match pixel_format
    {
        PixelFormat::Rgb => graphic_info::PixelFormat::Rgb,
        PixelFormat::Bgr => graphic_info::PixelFormat::Bgr,
        _ => panic!("Unsupported pixel format"),
    };
}

fn convert_mem_type(mem_type: MemoryType) -> mem_desc::MemoryType
{
    return match mem_type
    {
        MemoryType::RESERVED => mem_desc::MemoryType::Reserved,
        MemoryType::LOADER_CODE => mem_desc::MemoryType::LoaderCode,
        MemoryType::LOADER_DATA => mem_desc::MemoryType::LoaderData,
        MemoryType::BOOT_SERVICES_CODE => mem_desc::MemoryType::BootServicesCode,
        MemoryType::BOOT_SERVICES_DATA => mem_desc::MemoryType::BootServicesData,
        MemoryType::RUNTIME_SERVICES_CODE => mem_desc::MemoryType::RuntimeServicesCode,
        MemoryType::RUNTIME_SERVICES_DATA => mem_desc::MemoryType::RuntimeServicesData,
        MemoryType::CONVENTIONAL => mem_desc::MemoryType::Conventional,
        MemoryType::UNUSABLE => mem_desc::MemoryType::Unusable,
        MemoryType::ACPI_RECLAIM => mem_desc::MemoryType::AcpiReclaim,
        MemoryType::ACPI_NON_VOLATILE => mem_desc::MemoryType::AcpiNonVolatile,
        MemoryType::MMIO => mem_desc::MemoryType::Mmio,
        MemoryType::MMIO_PORT_SPACE => mem_desc::MemoryType::MmioPortSpace,
        MemoryType::PAL_CODE => mem_desc::MemoryType::PalCode,
        MemoryType::PERSISTENT_MEMORY => mem_desc::MemoryType::PersistentMemory,
        MemoryType(value) => mem_desc::MemoryType::Custom(value),
    };
}

fn convert_mem_attr(mem_attr: MemoryAttribute) -> mem_desc::MemoryAttribute
{
    return mem_desc::MemoryAttribute::from_bits_truncate(mem_attr.bits());
}

fn jump_to_entry(entry_base_addr: u64, bi: &BootInfo, stack_addr: u64, stack_size: u64)
{
    let stacktop = stack_addr + stack_size * UEFI_PAGE_SIZE as u64;
    let entry_point: extern "sysv64" fn(*const BootInfo) =
        unsafe { mem::transmute(entry_base_addr) };
    entry_point(bi);
}
