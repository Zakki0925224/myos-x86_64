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
mod serial;
mod util;

extern crate alloc;

use alloc::alloc::Layout;
use arch::asm;
use common::boot_info::BootInfo;
use core::panic::PanicInfo;
use log::*;

use crate::arch::{gdt, idt};

#[no_mangle]
#[start]
pub extern "sysv64" fn kernel_main(boot_info: *const BootInfo) -> ! {
    let boot_info = unsafe { boot_info.read() };

    // initialize frame buffer, serial, terminal, logger
    graphics::init(boot_info.graphic_info);

    // initialize GDT (TODO: not working correctly)
    //gdt::init();
    // initialize IDT
    idt::init();

    // initialize memory management
    mem::init(boot_info.get_mem_map());

    // initialize pci
    bus::init();

    // initialize device drivers
    device::init();

    env::print_info();

    loop {
        asm::hlt();
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{:?}", info.message());
    error!("{:?}", info.location());

    loop {
        asm::hlt();
    }
}
