use core::mem::size_of;
use log::info;
use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

use crate::{
    arch::register::{
        segment::{self, Cs},
        Register,
    },
    util::mutex::Mutex,
};

use super::{
    asm::{self, DescriptorTableArgs},
    idt::GateDescriptor,
};

static mut GDT: Mutex<GlobalDescriptorTable> = Mutex::new(GlobalDescriptorTable::new());

const GDT_LEN: usize = 5;

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 4]
pub enum SegmentType {
    ExecuteRead = 0xa,
    ReadWrite = 0x2,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C, align(8))]
pub struct SegmentDescriptor {
    limit_low: B16,
    base_low: B16,
    base_mid: B8,
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

impl SegmentDescriptor {
    pub fn set_code_seg(&mut self, seg_type: SegmentType, dpl: u8, base: u32, limit: u32) {
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

    pub fn set_data_seg(&mut self, seg_type: SegmentType, dpl: u8, base: u32, limit: u32) {
        self.set_code_seg(seg_type, dpl, base, limit);
        self.set_is_long_mode(false);
        self.set_default_op_size(1);
    }
}

struct GlobalDescriptorTable {
    entries: [SegmentDescriptor; GDT_LEN],
}

impl GlobalDescriptorTable {
    pub const fn new() -> Self {
        Self {
            entries: [SegmentDescriptor::new(); GDT_LEN],
        }
    }

    pub fn set_desc(&mut self, vec_num: usize, desc: SegmentDescriptor) {
        if vec_num >= GDT_LEN {
            return;
        }

        self.entries[vec_num] = desc;
    }

    pub fn load(&self) {
        let limit = (size_of::<GateDescriptor>() * GDT_LEN - 1) as u16;
        let base = self.entries.as_ptr() as u64;

        let args = DescriptorTableArgs { limit, base };
        asm::lgdt(&args);

        //info!("gdt: Loaded GDT: {:?}", args);
    }
}

pub fn init() {
    let mut gdt1 = SegmentDescriptor::new();
    let mut gdt2 = SegmentDescriptor::new();
    let mut gdt3 = SegmentDescriptor::new();
    let mut gdt4 = SegmentDescriptor::new();

    // kernel segments
    gdt1.set_code_seg(SegmentType::ExecuteRead, 0, 0, 0xffff_f);
    gdt2.set_data_seg(SegmentType::ReadWrite, 0, 0, 0xffff_f);

    // user segments
    gdt3.set_data_seg(SegmentType::ReadWrite, 3, 0, 0xffff_f);
    gdt4.set_code_seg(SegmentType::ExecuteRead, 3, 0, 0xffff_f);

    {
        let mut gdt = unsafe { GDT.try_lock() }.unwrap();
        gdt.set_desc(1, gdt1);
        gdt.set_desc(2, gdt2);
        gdt.set_desc(3, gdt3);
        gdt.set_desc(4, gdt4);
        gdt.load();
    }

    segment::set_ds_es_fs_gs(0);
    set_seg_reg_to_kernel();

    info!("gdt: Initialized GDT");
}

pub fn set_seg_reg_to_kernel() {
    segment::set_ss_cs(2 << 3, 1 << 3);
    assert_eq!(Cs::read().raw() >> 3, 1);
}

pub fn set_seg_reg_to_user() {
    segment::set_ss_cs(4 << 3 | 3, (3 << 3) | 3); // RPL = 3
    assert_eq!(Cs::read().raw(), (3 << 3) | 3);
}
