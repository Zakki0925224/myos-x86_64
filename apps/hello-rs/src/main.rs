#![no_std]
#![no_main]

use libm_rs::*;

#[no_mangle]
pub unsafe fn _start() {
    printf("Hello world!\n\0".as_ptr() as *const _);
    sys_exit(0);
}
