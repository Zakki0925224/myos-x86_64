#![no_std]
#![no_main]
#![feature(start)]
#![feature(abi_x86_interrupt)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

mod arch;
mod bus;
mod device;
mod env;
mod graphics;
mod mem;
mod terminal;
mod util;

extern crate alloc;

use alloc::alloc::Layout;
use arch::asm;
use common::boot_info::BootInfo;
use core::panic::PanicInfo;
use device::serial::{self, SERIAL};
use graphics::GRAPHICS;
use log::*;
use terminal::TERMINAL;

use crate::{arch::{gdt, idt}, device::xhci::host::XhcDriver, util::logger};

#[no_mangle]
#[start]
pub extern "sysv64" fn kernel_main(boot_info: *const BootInfo) -> !
{
    let boot_info = unsafe { boot_info.read() };

    // initialize graphics
    let graphic_info = &boot_info.graphic_info;
    GRAPHICS.lock().init(
        (graphic_info.resolution.0 as usize, graphic_info.resolution.1 as usize),
        graphic_info.format,
        graphic_info.framebuf_addr,
        graphic_info.framebuf_size as usize,
        graphic_info.stride as usize,
    );

    // initialize kerenl terminal
    SERIAL.lock().init(serial::IO_PORT_COM1);
    TERMINAL.lock().init();
    logger::init().unwrap();
    info!("terminal: Initialized kernel terminal");

    // initialize IDT
    idt::init();
    gdt::init();

    // initialize memory management
    mem::init(boot_info.get_mem_map());

    // initialize pci
    bus::init();

    // initialize devices
    let mut xhci = XhcDriver::new();
    if let Some(driver) = xhci.as_mut()
    {
        driver.init();
        driver.start();
        driver.scan_ports();
        driver.alloc_slots();
    }

    env::print_info();

    loop
    {
        asm::hlt();
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> !
{
    panic!("Allocation error: {:?}", layout);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    error!("{:?}", info.message());
    error!("{:?}", info.location());

    loop
    {
        asm::hlt();
    }
}
