use core::arch::asm;

use super::register::{control::Cr3, segment::*, Register};

#[derive(Debug)]
pub struct Context {
    pub cr3: u64,
    pub rip: u64,
    pub rflags: u64,
    pub cs: u16,
    pub ss: u16,
    pub fs: u16,
    pub gs: u16,
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub fpu_context: [u8; 512],
}

impl Default for Context {
    fn default() -> Self {
        Self {
            cr3: 0,
            rip: 0,
            rflags: 0,
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
}

impl Context {
    pub fn save_context() -> Self {
        let mut ctx = Self::default();

        ctx.cr3 = Cr3::read().raw();
        //ctx.rip =
        //ctx.rflags =
        ctx.cs = Cs::read().raw();
        ctx.ss = Ss::read().raw();
        ctx.fs = Fs::read().raw();
        ctx.gs = Gs::read().raw();

        unsafe {
            asm!(
                "mov {}, rax",
                "mov {}, rbx",
                "mov {}, rcx",
                "mov {}, rdx",
                "mov {}, rdi",
                "mov {}, rsi",
                "mov {}, rsp",
                "mov {}, rbp",
                "mov {}, r8",
                "mov {}, r9",
                "mov {}, r10",
                "mov {}, r11",
                "mov {}, r12",
                "mov {}, r13",
                "mov {}, r14",
                "mov {}, r15",
                "fxsave [{}]",
                out(reg) ctx.rax,
                out(reg) ctx.rbx,
                out(reg) ctx.rcx,
                out(reg) ctx.rdx,
                out(reg) ctx.rdi,
                out(reg) ctx.rsi,
                out(reg) ctx.rsp,
                out(reg) ctx.rbp,
                out(reg) ctx.r8,
                out(reg) ctx.r9,
                out(reg) ctx.r10,
                out(reg) ctx.r11,
                out(reg) ctx.r12,
                out(reg) ctx.r13,
                out(reg) ctx.r14,
                out(reg) ctx.r15,
                in(reg) &mut ctx.fpu_context
            );
        }

        ctx
    }
}
