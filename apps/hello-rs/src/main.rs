#![no_std]
#![no_main]

use libc_rs::*;

#[no_mangle]
pub unsafe fn _start() {
    let s = "Hello world!\n\0";
    printf(s.as_ptr() as *const _);
    exit(0);
}
