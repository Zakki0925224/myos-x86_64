#![no_std]
#![no_main]
#![feature(start)]
#![feature(alloc_error_handler)]

mod arch;
mod device;
mod graphics;

extern crate alloc;

use arch::asm;
use common::boot_info::BootInfo;
use core::panic::PanicInfo;
use device::serial::{self, SerialPort};
use graphics::{color::RGBColor, Graphics};

#[no_mangle]
#[start]
pub extern "sysv64" fn kernel_main(boot_info: &BootInfo) -> !
{
    let graphics = Graphics::new(boot_info.graphic_info);

    let mut serial = SerialPort::new(serial::IO_PORT_COM1);
    serial.init();
    serial.send_data(b'H').unwrap();
    serial.send_data(b'e').unwrap();
    serial.send_data(b'l').unwrap();
    serial.send_data(b'l').unwrap();
    serial.send_data(b'o').unwrap();
    serial.send_data(b'!').unwrap();
    serial.send_data(b'\n').unwrap();

    graphics.clear(0xffffffff);
    graphics.draw_rect(100, 100, 100, 100, RGBColor::new(0, 255, 0)).unwrap();

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

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> !
{
    panic!("Allocation error: {:?}", layout);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> !
{
    loop
    {
        asm::hlt();
    }
}
