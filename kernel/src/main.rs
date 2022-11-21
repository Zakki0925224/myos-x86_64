// TODO: implement global allocator

#![no_std]
#![no_main]
#![feature(start)]
//#![feature(alloc_error_handler)]

mod arch;
mod console;
mod device;
mod graphics;

//extern crate alloc;

use arch::asm;
use common::boot_info::BootInfo;
use console::CONSOLE;
use core::panic::PanicInfo;
use device::serial::{self, SERIAL};
use graphics::GRAPHICS;

use crate::graphics::color::COLOR_RED;

#[no_mangle]
#[start]
pub extern "sysv64" fn kernel_main(boot_info: &BootInfo) -> !
{
    let graphic_info = &boot_info.graphic_info;
    GRAPHICS.lock().init(
        (graphic_info.resolution.0 as usize, graphic_info.resolution.1 as usize),
        graphic_info.format,
        graphic_info.framebuf_addr,
        graphic_info.framebuf_size as usize,
        graphic_info.stride as usize,
    );

    SERIAL.lock().init(serial::IO_PORT_COM1);
    CONSOLE.lock().init();

    println!("Hello world!");

    for i in 0..1000
    {
        println!("{}", i);
    }

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
    CONSOLE.lock().set_fore_color(COLOR_RED);
    println!("{:?}", info);
    CONSOLE.lock().reset_fore_color();

    loop
    {
        asm::hlt();
    }
}
