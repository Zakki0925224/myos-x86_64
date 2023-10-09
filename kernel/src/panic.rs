use core::panic::PanicInfo;

use crate::{arch::asm, error};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{:?}", info.message());
    error!("{:?}", info.location());

    loop {
        asm::hlt();
    }
}
