#![no_std]
#![no_main]
#![feature(start)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::{arch::asm ,panic::PanicInfo};
use common::boot_info::BootInfo;

#[no_mangle]
#[start]
pub extern "C" fn kernel_main(_boot_info: &BootInfo) -> !
{
    unsafe { asm!("int3"); }
    loop
    {
        unsafe { asm!("hlt"); }
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
    loop {};
}