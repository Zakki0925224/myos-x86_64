#![no_std]
#![no_main]
#![feature(abi_efiapi)]

mod config;

#[macro_use]
extern crate log;

#[macro_use]
extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use common::{boot_info::BootInfo, graphic_info::{self, GraphicInfo}, mem_desc};
use core::{mem, slice::from_raw_parts_mut};
use uefi::{prelude::*, proto::{console::gop::{GraphicsOutput, PixelFormat}, media::{file::*, fs::SimpleFileSystem}}, table::boot::*, CStr16};
use xmas_elf::ElfFile;

use crate::config::DEFAULT_BOOT_CONFIG;

const PAGE_SIZE: usize = 0x1000;
const KERNEL_BASE_ADDR: usize = 0x10_0000;

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

    // read kernel.elf
    let mut f = open_file(bs, config.kernel_path);
    let buf = load_file_to_mem(bs, &mut f, KERNEL_BASE_ADDR);
    let kernel = ElfFile::new(buf).expect("Failed to parse ELF file");
    //copy_load_segs(&kernel);
    let kernel_entry_point_addr = kernel.header.pt2.entry_point();
    info!("Kernel entry point: 0x{:x}", kernel_entry_point_addr);

    // get memory map
    let mmap_size = bs.memory_map_size().map_size;
    let mmap_buf = Box::leak(vec![0; mmap_size * 2].into_boxed_slice());

    // exit boot service
    info!("Exit boot services");
    let mut mem_map = Vec::with_capacity(128);

    let (_rt, mmap_iter) =
        st.exit_boot_services(handle, mmap_buf).expect("Failed to exit boot services");

    for desc in mmap_iter
    {
        let ty = convert_mem_type(desc.ty);
        let phys_start = desc.phys_start;
        let virt_start = desc.virt_start;
        let page_cnt = desc.page_count;
        let attr = convert_mem_attr(desc.att);

        mem_map.push(mem_desc::MemoryDescriptor { ty, phys_start, virt_start, page_cnt, attr });
    }

    // can't use info!()

    let mem_map_len = mem_map.len();
    let bi = BootInfo::new(mem_map.as_slice(), mem_map_len, graphic_info);

    // https://github.com/uchan-nos/os-from-zero/issues/41
    // not changed when add flag "-z separate-code"
    jump_to_entry(
        kernel_entry_point_addr - 0x1000,
        &bi,
        config.kernel_stack_addr,
        config.kernel_stack_size,
    );

    return Status::SUCCESS;
}

fn open_file(bs: &BootServices, path: &str) -> RegularFile
{
    info!("Opening file: \"{}\"", path);

    let fs = bs.locate_protocol::<SimpleFileSystem>().expect("Failed to get FileSystem");

    let fs = unsafe { &mut *fs.get() };
    let mut buf = [0; 256];
    let path = CStr16::from_str_with_buf(path, &mut buf).expect("Failed to convert path to ucs-2");
    let mut root = fs.open_volume().expect("Failed to open volume");
    let handle =
        root.open(path, FileMode::Read, FileAttribute::empty()).expect("Failed to open file");

    match handle.into_type().expect("Failed to into_type")
    {
        FileType::Regular(r) => return r,
        _ => panic!("Invalid file type"),
    }
}

fn load_file_to_mem(bs: &BootServices, file: &mut RegularFile, addr: usize) -> &'static mut [u8]
{
    let mut info_buf = [0; 256];

    // FIXME: paniced here on real mahcine? (usb boot)
    let info = file.get_info::<FileInfo>(&mut info_buf).expect("Failed to get file info");

    let pages = (info.file_size() as usize + 0xfff) / PAGE_SIZE;
    let mem_start = bs
        .allocate_pages(AllocateType::Address(addr), MemoryType::LOADER_DATA, pages)
        .expect("Failed to allocate pages");
    let buf = unsafe { from_raw_parts_mut(mem_start as *mut u8, pages * PAGE_SIZE) };
    let len = file.read(buf).expect("Failed to read file");

    info!("Loaded {}bytes at 0x{:x}", len, mem_start);

    return &mut buf[..len];
}

fn init_graphic(bs: &BootServices, resolution: Option<(usize, usize)>) -> GraphicInfo
{
    let gop = bs.locate_protocol::<GraphicsOutput>().expect("Failed to get GraphicsOutput");

    let gop = unsafe { &mut *gop.get() };

    if let Some(resolution) = resolution
    {
        let mode = gop
            .modes()
            .find(|mode| mode.info().resolution() == resolution)
            .expect("Graphic mode not found");

        info!("Switching graphic mode...");
        gop.set_mode(&mode).expect("Failed to set graphic mode");
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
    let stacktop = stack_addr + stack_size * PAGE_SIZE as u64;
    let entry_point: extern "sysv64" fn(&BootInfo) =
        unsafe { mem::transmute(entry_base_addr as *const u64) };
    info!("Entering kernel (0x{:x})...", entry_base_addr);
    entry_point(bi);
    info!("Leaved kernel");
}
