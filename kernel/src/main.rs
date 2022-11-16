// TODO: implement global allocator

#![no_std]
#![no_main]
#![feature(start)]
//#![feature(alloc_error_handler)]

mod arch;
mod device;
mod graphics;

//extern crate alloc;

use arch::asm;
use common::boot_info::BootInfo;
use core::{panic::PanicInfo, ptr::read_volatile};
use device::serial::{self, SerialPort};
use graphics::{color::*, font::{self, PsfFont}, Graphics};

#[no_mangle]
#[start]
pub extern "sysv64" fn kernel_main(boot_info: &BootInfo) -> !
{
    let graphic_info = &boot_info.graphic_info;
    let graphics = Graphics::new(
        (graphic_info.resolution.0 as usize, graphic_info.resolution.1 as usize),
        graphic_info.format,
        graphic_info.framebuf_addr,
        graphic_info.framebuf_size as usize,
        graphic_info.stride as usize,
    );

    let mut serial = SerialPort::new(serial::IO_PORT_COM1);
    serial.init();
    serial.send_data(b'H').unwrap();
    serial.send_data(b'e').unwrap();
    serial.send_data(b'l').unwrap();
    serial.send_data(b'l').unwrap();
    serial.send_data(b'o').unwrap();
    serial.send_data(b'!').unwrap();
    serial.send_data(b'\n').unwrap();

    graphics.clear(&RGBColor::new(0, 0, 0));
    graphics.draw_rect(0, 0, 20, 20, &RGBColor::new(255, 255, 255));
    graphics.draw_rect(20, 0, 20, 20, &RGBColor::new(128, 128, 0));
    graphics.draw_rect(40, 0, 20, 20, &RGBColor::new(255, 255, 0));
    graphics.draw_rect(60, 0, 20, 20, &RGBColor::new(255, 0, 255));
    graphics.draw_rect(80, 0, 20, 20, &RGBColor::new(192, 192, 192));
    graphics.draw_rect(100, 0, 20, 20, &RGBColor::new(0, 255, 255));
    graphics.draw_rect(120, 0, 20, 20, &RGBColor::new(0, 255, 0));
    graphics.draw_rect(140, 0, 20, 20, &RGBColor::new(255, 0, 0));
    graphics.draw_rect(160, 0, 20, 20, &RGBColor::new(128, 128, 128));
    graphics.draw_rect(180, 0, 20, 20, &RGBColor::new(0, 0, 255));
    graphics.draw_rect(200, 0, 20, 20, &RGBColor::new(0, 255, 0));
    graphics.draw_rect(220, 0, 20, 20, &RGBColor::new(128, 0, 128));
    graphics.draw_rect(240, 0, 20, 20, &RGBColor::new(0, 0, 0));
    graphics.draw_rect(260, 0, 20, 20, &RGBColor::new(0, 0, 128));
    graphics.draw_rect(280, 0, 20, 20, &RGBColor::new(0, 128, 128));
    graphics.draw_rect(300, 0, 20, 20, &RGBColor::new(128, 0, 0));

    let font = PsfFont::new();

    if (font.has_unicode_table)
    {
        serial.send_data(b'U').unwrap();
        serial.send_data(b'T').unwrap();
    }

    loop
    {
        //asm::cli();

        if let Ok(data) = serial.receive_data()
        {
            serial.send_data(data).unwrap();
        }

        //asm::hlt();
    }
}

// #[alloc_error_handler]
// fn alloc_error_handler(layout: alloc::alloc::Layout) -> !
// {
//     panic!("Allocation error: {:?}", layout);
// }

#[panic_handler]
fn panic(_info: &PanicInfo) -> !
{
    loop
    {
        asm::hlt();
    }
}
