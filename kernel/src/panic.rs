use crate::{
    arch,
    device::panic_screen,
    error,
    qemu::{self, EXIT_FAILURE},
};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{:?}", info.message());
    error!("{:?}", info.location());

    // prevent overwriting by graphics::frame_buf
    arch::disabled_int(|| {
        panic_screen::write_fmt(format_args!("{:?}\n", info.message())).unwrap();
        panic_screen::write_fmt(format_args!("{:?}\n", info.location())).unwrap();

        qemu::exit(EXIT_FAILURE);
        loop {
            arch::hlt();
        }
    });

    unreachable!();
}
