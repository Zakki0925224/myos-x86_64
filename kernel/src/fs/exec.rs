use super::initramfs;
use common::elf::{self, Elf64};
use core::mem;
use log::{error, info};

pub fn exec_elf(file_name: &str, args: &[&str]) {
    info!("exec: args: {:?}", args);

    let (_, elf_data) = match initramfs::get_file(file_name) {
        Ok(res) => match res {
            Some((meta, data)) => (meta, data),
            None => {
                error!("exec: The file \"{}\" does not exist", file_name);
                return;
            }
        },
        Err(_) => {
            error!("exec: Initramfs is locked");
            return;
        }
    };

    let elf64 = match Elf64::new(&elf_data) {
        Ok(e) => e,
        Err(_) => {
            error!("exec: The file \"{}\" is not an executable file", file_name);
            return;
        }
    };

    let header = elf64.read_header();

    if header.elf_type() != elf::Type::Executable {
        error!("exec: The file \"{}\" is not an executable file", file_name);
        return;
    }

    if header.machine() != elf::Machine::X8664 {
        error!("exec: Unsupported ISA");
        return;
    }

    info!(
        "entry: 0x{:x}",
        elf_data.as_ptr() as u64 + header.entry_point
    );
    let entry_point: extern "sysv64" fn() -> i32 =
        unsafe { mem::transmute(elf_data.as_ptr().offset(header.entry_point as isize)) };

    let ret = entry_point();
    info!("exec: Exited ({})", ret);
}
