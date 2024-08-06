use crate::{
    arch, error,
    qemu::{self, EXIT_FAILURE},
};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{:?}", info.message());
    error!("{:?}", info.location());

    qemu::exit(EXIT_FAILURE);
    loop {
        arch::hlt();
    }
}
