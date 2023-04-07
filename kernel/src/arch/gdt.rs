use core::mem::size_of;

use log::info;
use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

use super::{addr::VirtualAddress, asm::{self, DescriptorTableArgs}, idt::GateDescriptor};

static mut GDT: [SegmentDescriptor; GDT_LEN] = [SegmentDescriptor::new(); GDT_LEN];

const GDT_LEN: usize = 8192;

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 4]
pub enum SegmentType
{
    ExecuteRead = 0xa,
    ReadWrite = 0x2,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SegmentDescriptor
{
    limit_low: B16,
    base_low: B16,
    base_mid: B8,
    #[bits = 4]
    seg_type: SegmentType,
    is_not_system_seg: bool,
    dpl: B2,
    p: bool,
    limit_high: B4,
    available: B1,
    is_long_mode: bool,
    default_op_size: B1,
    granularity: B1,
    base_high: B8,
}

impl SegmentDescriptor
{
    pub fn set_code_seg(&mut self, seg_type: SegmentType, dpl: u8, base: u32, limit: u32)
    {
        self.set_base_low(base as u16);
        self.set_base_mid((base >> 16) as u8);
        self.set_base_high((base >> 24) as u8);

        self.set_limit_low(limit as u16);
        self.set_limit_high((limit >> 16) as u8);

        self.set_seg_type(seg_type);
        self.set_is_not_system_seg(true);
        self.set_dpl(dpl);
        self.set_p(true);
        self.set_available(0);
        self.set_is_long_mode(true);
        self.set_default_op_size(0);
        self.set_granularity(1);
    }

    pub fn set_data_seg(&mut self, seg_type: SegmentType, dpl: u8, base: u32, limit: u32)
    {
        self.set_code_seg(seg_type, dpl, base, limit);
        self.set_is_long_mode(false);
        self.set_default_op_size(1);
    }
}

fn set_desc(vec_num: usize, desc: SegmentDescriptor)
{
    unsafe {
        GDT[vec_num] = desc;
    }
}

fn load_gdt()
{
    let limit = (size_of::<[GateDescriptor; GDT_LEN]>() - 1) as u16;
    let base = &unsafe { GDT } as *const _ as u64;
    let args = DescriptorTableArgs { limit, base };
    asm::lgdt(&args);
}

pub fn init()
{
    let mut gdt1 = SegmentDescriptor::new();
    let mut gdt2 = SegmentDescriptor::new();

    gdt1.set_code_seg(SegmentType::ExecuteRead, 0, 0, 0xffff_f);
    gdt2.set_data_seg(SegmentType::ReadWrite, 0, 0, 0xffff_f);

    set_desc(1, gdt1);
    set_desc(2, gdt2);

    load_gdt();

    asm::set_ds(0);
    asm::set_es(0);
    asm::set_fs(0);
    asm::set_gs(0);
    //asm::set_ss(1 << 3);
    asm::set_ss(0);
    asm::set_cs(2 << 3);

    info!("gdt: Initialized GDT (having todo)");
}
