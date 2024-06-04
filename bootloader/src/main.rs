#![no_std]
#![no_main]

mod config;

#[macro_use]
extern crate alloc;

use crate::config::DEFAULT_BOOT_CONFIG;
use alloc::vec::Vec;
use common::{
    boot_info::BootInfo,
    elf::{Elf64, SegmentType},
    graphic_info::{self, GraphicInfo},
    mem_desc::{self, UEFI_PAGE_SIZE},
};
use core::{mem, slice::from_raw_parts_mut};
use log::info;
use uefi::{
    prelude::*,
    proto::{
        console::gop::{GraphicsOutput, PixelFormat},
        media::{file::*, fs::SimpleFileSystem},
    },
    table::boot::*,
    CStr16,
};

#[entry]
fn efi_main(handle: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut st).unwrap();
    let bs = st.boot_services();

    info!("Running bootloader...");

    // load config
    let config = DEFAULT_BOOT_CONFIG;

    // graphic info
    let graphic_info = init_graphic(bs, config.resolution);
    info!("{:?}", graphic_info);

    // load kernel
    let kernel_entry_point_addr = load_elf(bs, config.kernel_path);
    info!("Kernel entry point: 0x{:x}", kernel_entry_point_addr);

    // load initramfs
    let (initramfs_start_virt_addr, initramfs_page_cnt) = load_initramfs(bs, config.initramfs_path);
    //let (initramfs_start_virt_addr, initramfs_page_cnt) = (0, 0);

    // exit boot service and get memory map
    info!("Exit boot services");
    let mut mem_map = Vec::with_capacity(128);

    let (_, map) = st.exit_boot_services(MemoryType::RUNTIME_SERVICES_DATA);

    for desc in map.entries() {
        let ty = convert_mem_type(desc.ty);
        let phys_start = desc.phys_start;
        let virt_start = desc.virt_start;
        let page_cnt = desc.page_count;
        let attr = convert_mem_attr(desc.att);

        mem_map.push(mem_desc::MemoryDescriptor {
            ty,
            phys_start,
            virt_start,
            page_cnt,
            attr,
        });
    }

    let bi = BootInfo::new(
        mem_map.as_slice(),
        graphic_info,
        initramfs_start_virt_addr,
        initramfs_page_cnt,
    );

    jump_to_entry(kernel_entry_point_addr, &bi);

    Status::SUCCESS
}

fn read_file(bs: &BootServices, path: &str) -> RegularFile {
    info!("Opening file: \"{}\"", path);
    let sfs_handle = bs.get_handle_for_protocol::<SimpleFileSystem>().unwrap();
    let mut root = bs
        .open_protocol_exclusive::<SimpleFileSystem>(sfs_handle)
        .unwrap()
        .open_volume()
        .unwrap();
    let mut buf = [0; 256];
    let path = CStr16::from_str_with_buf(path, &mut buf).unwrap();
    let file = root
        .open(path, FileMode::Read, FileAttribute::empty())
        .unwrap()
        .into_type()
        .unwrap();

    match file {
        FileType::Regular(file) => file,
        FileType::Dir(_) => panic!("Not file: \"{}\"", path),
    }
}

fn load_elf(bs: &BootServices, path: &str) -> u64 {
    let mut file = read_file(bs, path);

    let file_info = file.get_boxed_info::<FileInfo>().unwrap();
    let file_size = file_info.file_size() as usize;
    let mut buf = vec![0; file_size];

    file.read(&mut buf).unwrap();

    // load elf
    let elf = Elf64::new(&buf).unwrap();

    let mut dest_start = usize::MAX;
    let mut dest_end = 0;

    let phs = elf.program_headers();

    for p in &phs {
        if p.segment_type() != SegmentType::Load {
            continue;
        }

        dest_start = dest_start.min(p.virt_addr as usize);
        dest_end = dest_end.max((p.virt_addr + p.mem_size) as usize);
    }

    let pages = (dest_end - dest_start + UEFI_PAGE_SIZE - 1) / UEFI_PAGE_SIZE;
    bs.allocate_pages(
        AllocateType::Address(dest_start as u64),
        MemoryType::LOADER_DATA,
        pages,
    )
    .unwrap();

    for p in &phs {
        if p.segment_type() != SegmentType::Load {
            continue;
        }

        let offset = p.offset as usize;
        let file_size = p.file_size as usize;
        let mem_size = p.mem_size as usize;
        let dest = unsafe { from_raw_parts_mut(p.virt_addr as *mut u8, mem_size) };
        dest[..file_size].copy_from_slice(&buf[offset..offset + file_size]);
        dest[file_size..].fill(0);
    }

    info!("Loaded ELF at: 0x{:x}", dest_start);
    elf.header().entry_point
}

fn load_initramfs(bs: &BootServices, path: &str) -> (u64, u64) {
    let mut file = read_file(bs, path);

    let file_info = file.get_boxed_info::<FileInfo>().unwrap();
    let file_size = file_info.file_size() as usize;
    let mut buf = vec![0; file_size];

    file.read(&mut buf).unwrap();

    let pages = (file_size + UEFI_PAGE_SIZE - 1) / UEFI_PAGE_SIZE;
    // TODO: want to use virtual address
    let phys_addr = bs
        .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, pages)
        .unwrap();

    let dest = unsafe { from_raw_parts_mut(phys_addr as *mut u8, pages * UEFI_PAGE_SIZE) };
    dest[..file_size].copy_from_slice(&buf);
    dest[file_size..].fill(0);

    info!("Loaded initramfs at: 0x{:x}", phys_addr);

    (phys_addr, pages as u64)
}

fn init_graphic(bs: &BootServices, resolution: Option<(usize, usize)>) -> GraphicInfo {
    let gop_handle = bs.get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let mut gop = bs
        .open_protocol_exclusive::<GraphicsOutput>(gop_handle)
        .unwrap();

    if let Some(resolution) = resolution {
        let mode = gop
            .modes(bs)
            .find(|mode| mode.info().resolution() == resolution)
            .unwrap();

        info!("Switching graphic mode...");
        gop.set_mode(&mode).unwrap();
    }

    let mode_info = gop.current_mode_info();
    let res = mode_info.resolution();

    GraphicInfo {
        resolution: (res.0 as u32, res.1 as u32),
        format: convert_pixel_format(mode_info.pixel_format()),
        stride: mode_info.stride() as u32,
        framebuf_addr: gop.frame_buffer().as_mut_ptr() as u64,
        framebuf_size: gop.frame_buffer().size() as u64,
    }
}

fn convert_pixel_format(pixel_format: PixelFormat) -> graphic_info::PixelFormat {
    match pixel_format {
        PixelFormat::Rgb => graphic_info::PixelFormat::Rgb,
        PixelFormat::Bgr => graphic_info::PixelFormat::Bgr,
        _ => panic!("Unsupported pixel format"),
    }
}

fn convert_mem_type(mem_type: MemoryType) -> mem_desc::MemoryType {
    match mem_type {
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
        MemoryType(value) => mem_desc::MemoryType::Other(value),
    }
}

fn convert_mem_attr(mem_attr: MemoryAttribute) -> u64 {
    mem_attr.bits()
}

fn jump_to_entry(entry_base_addr: u64, bi: &BootInfo) {
    let entry_point: extern "sysv64" fn(*const BootInfo) =
        unsafe { mem::transmute(entry_base_addr) };
    entry_point(bi);
}
