#![no_std]
#![no_main]

use libm_rs::*;

#[no_mangle]
pub unsafe fn _start() {
    let s = "Hello world!\n\0";
    printf(s.as_ptr() as *const _);
    sys_exit(0);
}
