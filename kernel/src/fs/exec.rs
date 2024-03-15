use super::initramfs;
use crate::{
    arch::task,
    error::Result,
    mem::{bitmap, paging::PAGE_SIZE},
};
use common::elf::{self, Elf64};
use core::mem;
use log::{error, info};

pub fn exec_elf(file_name: &str, args: &[&str]) -> Result<()> {
    info!("exec: args: {:?}", args);

    let (_, elf_data) = match initramfs::get_file(file_name) {
        Ok(res) => match res {
            Some((meta, data)) => (meta, data),
            None => {
                error!("exec: The file \"{}\" does not exist", file_name);
                return Ok(());
            }
        },
        Err(e) => {
            return Err(e);
        }
    };

    let elf64 = match Elf64::new(&elf_data) {
        Ok(e) => e,
        Err(_) => {
            error!("exec: The file \"{}\" is not an executable file", file_name);
            return Ok(());
        }
    };

    let header = elf64.read_header();

    if header.elf_type() != elf::Type::Executable {
        error!("exec: The file \"{}\" is not an executable file", file_name);
        return Ok(());
    }

    if header.machine() != elf::Machine::X8664 {
        error!("exec: Unsupported ISA");
        return Ok(());
    }

    // copy elf data to user frame
    let user_mem_frame_info = bitmap::alloc_mem_frame((elf_data.len() / PAGE_SIZE).max(1))?;
    info!("{:?}", user_mem_frame_info);
    user_mem_frame_info
        .frame_start_virt_addr
        .copy_from_nonoverlapping(elf_data.as_ptr(), elf_data.len());
    user_mem_frame_info.set_permissions_to_user()?;
    info!("{:?}", user_mem_frame_info.get_permissions()?);

    let entry_addr = user_mem_frame_info.frame_start_virt_addr.get() + header.entry_point - 0x1000;

    info!("entry: 0x{:x}", entry_addr);
    let entry: extern "sysv64" fn() = unsafe { mem::transmute(entry_addr as *const ()) };
    task::exec_user_task(entry)?;

    user_mem_frame_info.set_permissions_to_supervisor()?;
    bitmap::dealloc_mem_frame(user_mem_frame_info)?;

    info!("exec: Exited");

    Ok(())
}
