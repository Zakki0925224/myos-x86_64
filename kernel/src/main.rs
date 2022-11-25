// TODO: implement global allocator

#![no_std]
#![no_main]
#![feature(start)]
#![feature(abi_x86_interrupt)]
//#![feature(alloc_error_handler)]

mod arch;
mod device;
mod env;
mod graphics;
mod terminal;
mod util;

//extern crate alloc;

use arch::asm;
use common::boot_info::BootInfo;
use core::panic::PanicInfo;
use device::serial::{self, SERIAL};
use graphics::GRAPHICS;
use log::*;
use terminal::TERMINAL;

use crate::{arch::idt, util::logger};

#[no_mangle]
#[start]
pub extern "sysv64" fn kernel_main(boot_info: &BootInfo) -> !
{
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
    info!("Initialized kernel terminal");

    // initialize IDT
    idt::init();

    env::print_info();

    loop
    {
        //asm::cli();

        //asm::hlt();
    }
}

// #[alloc_error_handler]
// fn alloc_error_handler(layout: alloc::alloc::Layout) -> !
// {
//     panic!("Allocation error: {:?}", layout);
// }

#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    error!("{:?}", info);

    loop
    {
        asm::hlt();
    }
}
