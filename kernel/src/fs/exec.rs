use super::{path::Path, vfs};
use crate::{arch::task, dwarf, error::Result};
use common::elf::Elf64;
use log::{error, info};

pub fn exec_elf(elf_path: &Path, args: &[&str]) -> Result<()> {
    let fd_num = vfs::open_file(elf_path)?;
    let elf_data = vfs::read_file(&fd_num)?;
    let elf64 = match Elf64::new(&elf_data) {
        Ok(e) => e,
        Err(err) => return Err(err.into()),
    };

    vfs::close_file(&fd_num)?;

    let dwarf = match dwarf::parse(&elf64) {
        Ok(d) => Some(d),
        Err(err) => {
            error!("exec: Failed to parse DWARF: {:?}", err);
            None
        }
    };

    let exit_code = task::exec_user_task(elf64, elf_path, args, dwarf)?;
    info!("exec: Exited (code: {})", exit_code);

    Ok(())
}
