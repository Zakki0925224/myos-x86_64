use crate::{println, util::mutex::Mutex};
use common::boot_info::BootInfo;
use core::arch::asm;

const KERNEL_STACK_SIZE: usize = 1024 * 1024;
pub const USER_STACK_SIZE: usize = 1024 * 1024;

static KERNEL_STACK: KernelStack = KernelStack::new();

static mut KERNEL_CONTEXT: Mutex<Context> = Mutex::new(Context::new());
static mut USER_CONTEXT: Mutex<Context> = Mutex::new(Context::new());

#[repr(align(16))]
struct KernelStack([u8; KERNEL_STACK_SIZE]);

impl KernelStack {
    pub const fn new() -> Self {
        Self([0; KERNEL_STACK_SIZE])
    }
}

pub fn switch_kernel_stack(
    new_entry: extern "sysv64" fn(*const BootInfo) -> !,
    boot_info: *const BootInfo,
) -> ! {
    unsafe {
        asm!(
            "mov rdi, {}",
            "mov rsp, {}",
            "call {}",
            in(reg) boot_info,
            in(reg) KERNEL_STACK.0.as_ptr() as u64 + KERNEL_STACK_SIZE as u64,
            in(reg) new_entry
        );
    }
    unreachable!();
}

#[derive(Debug)]
#[repr(C, align(16))]
// 16 byte alignment for fxsave64 and fxstor64 instructions
pub struct Context {
    /* + 0x00 */ pub cr3: u64,
    /* + 0x08 */ pub rip: u64,
    /* + 0x10 */ pub rflags: u64,
    /* + 0x18 */ reserved: u64,
    /* + 0x20 */ pub cs: u64,
    /* + 0x28 */ pub ss: u64,
    /* + 0x30 */ pub fs: u64,
    /* + 0x38 */ pub gs: u64,
    /* + 0x40 */ pub rax: u64,
    /* + 0x48 */ pub rbx: u64,
    /* + 0x50 */ pub rcx: u64,
    /* + 0x58 */ pub rdx: u64,
    /* + 0x60 */ pub rdi: u64,
    /* + 0x68 */ pub rsi: u64,
    /* + 0x70 */ pub rsp: u64,
    /* + 0x78 */ pub rbp: u64,
    /* + 0x80 */ pub r8: u64,
    /* + 0x88 */ pub r9: u64,
    /* + 0x90 */ pub r10: u64,
    /* + 0x98 */ pub r11: u64,
    /* + 0xa0 */ pub r12: u64,
    /* + 0xa8 */ pub r13: u64,
    /* + 0xb0 */ pub r14: u64,
    /* + 0xb8 */ pub r15: u64,
    /* + 0xc0 */ pub fpu_context: [u8; 512],
}

impl Context {
    pub const fn new() -> Self {
        Self {
            cr3: 0,
            rip: 0,
            rflags: 0,
            reserved: 0,
            cs: 0,
            ss: 0,
            fs: 0,
            gs: 0,
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rdi: 0,
            rsi: 0,
            rsp: 0,
            rbp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            fpu_context: [0; 512],
        }
    }

    // TODO
    pub fn save_context(&mut self) {
        // do not use rust's cpu register wrappers here
        unsafe {
            asm!(
                "pushfq",
                "pop QWORD PTR [rsi + 0x10]", // rflags

                "mov [rsi + 0x20], cs",
                "mov [rsi + 0x28], ss",
                "mov [rsi + 0x30], fs",
                "mov [rsi + 0x38], gs",

                "mov [rsi + 0x40], rax",
                "mov [rsi + 0x48], rbx",
                "mov [rsi + 0x50], rcx",
                "mov [rsi + 0x58], rdx",
                "mov [rsi + 0x60], rdi",
                "mov [rsi + 0x68], rsi",

                "mov [rsi + 0x80], r8",
                "mov [rsi + 0x88], r9",
                "mov [rsi + 0x90], r10",
                "mov [rsi + 0x98], r11",
                "mov [rsi + 0xa0], r12",
                "mov [rsi + 0xa8], r13",
                "mov [rsi + 0xb0], r14",
                "mov [rsi + 0xb8], r15",
                "fxsave64 [rsi + 0xc0]", // fpu_context

                "mov rax, cr3", // use already saved register
                "mov [rsi + 0x00], rax", // cr3

                // "lea r8, [rip + 0f]",
                // "mov [rsi + 0x08], r8", // rip
                in("rsi") self as *mut _,
            );
        }
    }

    pub fn debug(&self) {
        println!("{:?}", self);
    }
}

pub fn save_kernel_context() {
    if let Ok(mut ctx) = unsafe { KERNEL_CONTEXT.try_lock() } {
        ctx.save_context();
        ctx.debug();
    }
}
