use crate::{
    arch::{
        addr::VirtualAddress,
        apic,
        gdt::*,
        register::{model_specific::*, Register},
        task,
    },
    device::console,
    env,
    error::*,
    fs::{
        self,
        vfs::{self, file_desc::FileDescriptorNumber},
    },
    graphics::{multi_layer::LayerId, simple_window_manager},
    mem::{bitmap, paging::PAGE_SIZE},
    print, util,
};
use alloc::{ffi::CString, string::*, vec::Vec};
use common::libm::{Stat, Utsname};
use core::{arch::asm, slice};
use log::*;

#[naked]
extern "sysv64" fn asm_syscall_handler() {
    unsafe {
        asm!(
            "push rbp",
            "push rcx",
            "push r11",
            "mov rcx, r10", // rcx was updated by syscall instruction
            "mov rbp, rsp",
            "and rsp, -16",
            "call syscall_handler",
            "mov rsp, rbp",
            "pop r11",
            "pop rcx",
            "pop rbp",
            "sysretq",
            options(noreturn)
        );
    }
}

#[no_mangle]
extern "sysv64" fn syscall_handler(
    arg0: u64, // (sysv abi) rdi
    arg1: u64, // (sysv abi) rsi
    arg2: u64, // (sysv abi) rdx
    arg3: u64, // (sysv abi) rcx from r10
    arg4: u64, // (sysv abi) r8
    arg5: u64, // (sysv abi) r9
) -> i64 /* rax */ {
    //let args = [arg0, arg1, arg2, arg3, arg4, arg5];
    //info!("syscall: Called!(args: {:?})", args);

    match arg0 {
        // read syscall
        0 => {
            let fd = match FileDescriptorNumber::new_val(arg1 as i64) {
                Ok(fd) => fd,
                Err(err) => {
                    error!("syscall: read: {:?}", err);
                    return -1;
                }
            };
            let buf_addr = arg2.into();
            let buf_len = arg3 as usize;
            if let Err(err) = sys_read(fd, buf_addr, buf_len) {
                error!("syscall: read: {:?}", err);
                return -1;
            }
        }
        // write syscall
        1 => {
            let fd = match FileDescriptorNumber::new_val(arg1 as i64) {
                Ok(fd) => fd,
                Err(err) => {
                    error!("syscall: write: {:?}", err);
                    return -1;
                }
            };
            let s_ptr = arg2 as *const u8;
            let s_len = arg3 as usize;
            if let Err(err) = sys_write(fd, s_ptr, s_len) {
                error!("syscall: write: {:?}", err);
                return -1;
            }
        }
        // open syscall
        2 => {
            let filename_ptr = arg1 as *const u8;
            let fd = match sys_open(filename_ptr) {
                Ok(fd) => fd,
                Err(err) => {
                    error!("syscall: open: {:?}", err);
                    return -1;
                }
            };
            return fd.get() as i64;
        }
        // close syscall
        3 => {
            let fd = match FileDescriptorNumber::new_val(arg1 as i64) {
                Ok(fd) => fd,
                Err(err) => {
                    error!("syscall: close: {:?}", err);
                    return -1;
                }
            };
            if let Err(err) = sys_close(fd) {
                error!("syscall: close: {:?}", err);
                return -1;
            }
        }
        // exit syscall
        4 => {
            let status = arg1;
            sys_exit(status);
            unreachable!();
        }
        // sbrk syscall
        5 => {
            let len = arg1 as usize;
            let addr = match sys_sbrk(len) {
                Ok(addr) => addr.get(),
                Err(err) => {
                    error!("syscall: sbrk: {:?}", err);
                    return 0; // return null address
                }
            };
            return addr as i64;
        }
        // uname syscall
        6 => {
            if let Err(err) = sys_uname(arg1.into()) {
                error!("syscall: uname: {:?}", err);
                return -1;
            }
        }
        // break syscall
        7 => {
            sys_break();
            unreachable!();
        }
        // stat syscall
        8 => {
            let fd = match FileDescriptorNumber::new_val(arg1 as i64) {
                Ok(fd) => fd,
                Err(err) => {
                    error!("syscall: read: {:?}", err);
                    return -1;
                }
            };

            if let Err(err) = sys_stat(fd, arg2.into()) {
                error!("syscall: stat: {:?}", err);
                return -1;
            }
        }
        // uptime syscall
        9 => {
            let uptime = sys_uptime();
            return uptime as i64;
        }
        // exec syscall
        10 => {
            let args_ptr = arg1 as *const u8;
            if let Err(err) = sys_exec(args_ptr) {
                error!("syscall: exec: {:?}", err);
                return -1;
            }
        }
        // getcwd syscall
        11 => {
            let buf_addr = arg1.into();
            let buf_len = arg2 as usize;
            if let Err(err) = sys_getcwd(buf_addr, buf_len) {
                error!("syscall: getcwd: {:?}", err);
                return -1;
            }
        }
        // chdir syscall
        12 => {
            let path_ptr = arg1 as *const u8;
            if let Err(err) = sys_chdir(path_ptr) {
                error!("syscall: chdir: {:?}", err);
                return -1;
            }
        }
        // create_window syscall
        13 => {
            let title_ptr = arg1 as *const u8;
            let x_pos = arg2 as usize;
            let y_pos = arg3 as usize;
            let width = arg4 as usize;
            let height = arg5 as usize;

            match sys_create_window(title_ptr, x_pos, y_pos, width, height) {
                Ok(wd) => return wd.get() as i64,
                Err(err) => {
                    error!("syscall: create_window: {:?}", err);
                    return -1;
                }
            }
        }
        // destroy_window syscall
        14 => {
            let wd = match LayerId::new_val(arg1 as i64) {
                Ok(wd) => wd,
                Err(err) => {
                    error!("syscall: destroy_window: {:?}", err);
                    return -1;
                }
            };

            if let Err(err) = sys_destroy_window(wd) {
                error!("syscall: destroy_window: {:?}", err);
                return -1;
            }
        }
        // getcwdenames syscall
        15 => {
            let buf_addr = arg1.into();
            let buf_len = arg2 as usize;
            if let Err(err) = sys_getcwdenames(buf_addr, buf_len) {
                error!("syscall: getcwdenames: {:?}", err);
                return -1;
            }
        }
        num => {
            error!("syscall: Syscall number 0x{:x} is not defined", num);
            return -1;
        }
    }

    0
}

fn sys_read(fd: FileDescriptorNumber, buf_addr: VirtualAddress, buf_len: usize) -> Result<()> {
    match fd {
        FileDescriptorNumber::STDOUT | FileDescriptorNumber::STDERR => {
            return Err(Error::Failed("fd is not defined"));
        }
        FileDescriptorNumber::STDIN => {
            let mut input_s = None;
            while input_s.is_none() {
                if !console::is_ready_get_line() {
                    super::hlt();
                    continue;
                }

                super::disabled_int(|| {
                    input_s = console::get_line()?;
                    Result::Ok(())
                })?;
                break;
            }

            let c_s = CString::new(input_s.unwrap())
                .unwrap()
                .into_bytes_with_nul();
            buf_addr.copy_from_nonoverlapping(c_s.as_ptr(), buf_len);
        }
        fd => {
            let data = vfs::read_file(&fd)?;

            if buf_len < data.len() {
                return Err(Error::Failed("buffer is too small"));
            }

            buf_addr.copy_from_nonoverlapping(data.as_ptr(), data.len());
        }
    }

    Ok(())
}

fn sys_write(fd: FileDescriptorNumber, s_ptr: *const u8, s_len: usize) -> Result<()> {
    let s_slice = unsafe { slice::from_raw_parts(s_ptr, s_len) };
    let s = String::from_utf8_lossy(s_slice).to_string();

    match fd {
        FileDescriptorNumber::STDOUT => {
            print!("{}", s);
        }
        _ => return Err(Error::Failed("fd is not defined")),
    }

    Ok(())
}

fn sys_open(filename_ptr: *const u8) -> Result<FileDescriptorNumber> {
    let filename = unsafe { util::cstring::from_cstring_ptr(filename_ptr) };
    vfs::open_file(&filename)
}

fn sys_close(fd: FileDescriptorNumber) -> Result<()> {
    vfs::close_file(&fd)
}

fn sys_exit(status: u64) {
    task::return_task(status);
}

fn sys_sbrk(len: usize) -> Result<VirtualAddress> {
    let mem_frame_info = bitmap::alloc_mem_frame((len / PAGE_SIZE).max(1))?;
    mem_frame_info.set_permissions_to_user()?;
    let virt_addr = mem_frame_info.frame_start_virt_addr()?;
    info!(
        "syscall: sbrk: allocated {} bytes at 0x{:x}",
        mem_frame_info.frame_size,
        virt_addr.get()
    );
    task::push_allocated_mem_frame_info_for_user_task(mem_frame_info)?;
    Ok(virt_addr)
}

fn sys_uname(buf_addr: VirtualAddress) -> Result<()> {
    let sysname = env::OS_NAME.as_bytes();
    let nodename = "nodename".as_bytes();
    let release = "release".as_bytes();
    let version = env::ENV_VERSION.as_bytes();
    let machine = "x86_64".as_bytes();
    let domainname = "domainname".as_bytes();

    let mut utsname = Utsname::default();
    utsname.sysname[..sysname.len()].copy_from_slice(sysname);
    utsname.nodename[..nodename.len()].copy_from_slice(nodename);
    utsname.release[..release.len()].copy_from_slice(release);
    utsname.version[..version.len()].copy_from_slice(version);
    utsname.machine[..machine.len()].copy_from_slice(machine);
    utsname.domainname[..domainname.len()].copy_from_slice(domainname);
    buf_addr.copy_from_nonoverlapping(&utsname as *const Utsname, 1);
    Ok(())
}

fn sys_break() {
    task::debug_user_task();
    super::int3();
}

fn sys_stat(fd: FileDescriptorNumber, buf_addr: VirtualAddress) -> Result<()> {
    let size = match fd {
        FileDescriptorNumber::STDIN
        | FileDescriptorNumber::STDOUT
        | FileDescriptorNumber::STDERR => 0,
        fd => vfs::read_file(&fd)?.len() as u64, // FIXME
    };

    let stat = Stat { size };
    buf_addr.copy_from_nonoverlapping(&stat as *const Stat, 1);
    Ok(())
}

fn sys_uptime() -> u64 {
    apic::timer::get_current_ms().unwrap_or(0) as u64
}

fn sys_exec(args_ptr: *const u8) -> Result<()> {
    let args = unsafe { util::cstring::from_cstring_ptr(args_ptr) };
    let args: Vec<&str> = args.split(' ').collect();
    fs::exec::exec_elf(args[0], &args[1..])?;

    Ok(())
}

fn sys_getcwd(buf_addr: VirtualAddress, buf_len: usize) -> Result<()> {
    let cwd = vfs::cwd_path()?;
    let cwd_s = CString::new(cwd.as_str()).unwrap().into_bytes_with_nul();

    if buf_len < cwd_s.len() {
        return Err(Error::Failed("Buffer is too small"));
    }

    buf_addr.copy_from_nonoverlapping(cwd_s.as_ptr(), cwd_s.len());

    Ok(())
}

fn sys_chdir(path_ptr: *const u8) -> Result<()> {
    let path = unsafe { util::cstring::from_cstring_ptr(path_ptr) };
    vfs::chdir(&path)?;
    Ok(())
}

fn sys_create_window(
    title_ptr: *const u8,
    x_pos: usize,
    y_pos: usize,
    width: usize,
    height: usize,
) -> Result<LayerId> {
    let title = unsafe { util::cstring::from_cstring_ptr(title_ptr) };
    simple_window_manager::create_window(title, x_pos, y_pos, width, height)
}

fn sys_destroy_window(wd: LayerId) -> Result<()> {
    simple_window_manager::destroy_window(&wd)
}

fn sys_getcwdenames(buf_addr: VirtualAddress, buf_len: usize) -> Result<()> {
    let entry_names = fs::vfs::cwd_entry_names()?;
    let entry_names_s: Vec<u8> = entry_names
        .iter()
        .map(|n| CString::new(n.as_str()).unwrap().into_bytes_with_nul())
        .flatten()
        .collect();

    if buf_len < entry_names_s.len() {
        return Err(Error::Failed("Buffer is too small"));
    }

    buf_addr.copy_from_nonoverlapping(entry_names_s.as_ptr(), entry_names_s.len());

    Ok(())
}

pub fn enable() {
    let mut efer = ExtendedFeatureEnableRegister::read();
    efer.set_syscall_enable(true);
    efer.write();
    assert_eq!(ExtendedFeatureEnableRegister::read().syscall_enable(), true);

    let asm_syscall_handler_addr = asm_syscall_handler as *const () as u64;
    let mut lstar = LongModeSystemCallTargetAddressRegister::read();
    lstar.set_target_addr(asm_syscall_handler_addr);
    lstar.write();
    assert_eq!(
        LongModeSystemCallTargetAddressRegister::read().target_addr(),
        asm_syscall_handler_addr
    );

    let target_addr =
        ((KERNEL_MODE_CS_VALUE as u64) << 32) | ((KERNEL_MODE_SS_VALUE as u64 | 3) << 48);
    let mut star = SystemCallTargetAddressRegister::read();
    star.set_target_addr(target_addr); // set CS and SS to kernel segment
    star.write();
    assert_eq!(
        SystemCallTargetAddressRegister::read().target_addr(),
        target_addr
    );

    let mut fmask = SystemCallFlagMaskRegister::read();
    fmask.set_value(0);
    fmask.write();
    assert_eq!(SystemCallFlagMaskRegister::read().value(), 0);

    info!("syscall: Enabled syscall");
}
