use self::info::{InterruptStackFrame, PageFaultErrorCode};
use super::addr::*;
use crate::{
    arch::{
        asm::{self, DescriptorTableArgs},
        register::control::Cr2,
    },
    bus::usb::xhc,
    device::{ps2_keyboard, ps2_mouse},
    util::mutex::Mutex,
};
use core::mem::size_of;
use log::{error, info};
use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

pub mod info;

static mut IDT: Mutex<InterruptDescriptorTable> = Mutex::new(InterruptDescriptorTable::new());

// idt
const IDT_LEN: usize = 256;
const _VEC_DIVIDE_ERR: usize = 0x00;
const _VEC_DEBUG: usize = 0x01;
const _VEC_NMI_INT: usize = 0x02;
const VEC_BREAKPOINT: usize = 0x03;
const _VEC_OVERFLOW: usize = 0x04;
const _VEC_BOUND_RANGE_EXCEEDED: usize = 0x05;
const _VEC_INVALID_OPCODE: usize = 0x06;
const _VEC_DEVICE_NOT_AVAILABLE: usize = 0x07;
const VEC_DOUBLE_FAULT: usize = 0x08;
const _VEC_INVALID_TSS: usize = 0x0a;
const _VEC_SEG_NOT_PRESENT: usize = 0x0b;
const _VEC_STACK_SEG_FAULT: usize = 0x0c;
const VEC_GENERAL_PROTECTION: usize = 0x0d;
const VEC_PAGE_FAULT: usize = 0x0e;
const _VEC_FLOATING_POINT_ERR: usize = 0x10;
const _VEC_ALIGN_CHECK: usize = 0x11;
const _VEC_MACHINE_CHECK: usize = 0x12;
const _VEC_SIMD_FLOATING_POINT_EX: usize = 0x13;
const _VEC_VIRT_EX: usize = 0x14;
const _VEC_CTRL_PROTECTION_EX: usize = 0x15;
pub const VEC_XHCI_INT: usize = 64;

const END_OF_INT_REG_ADDR: u64 = 0xfee000b0;

// pic
const VEC_PIC_IRQ1: usize = 0x21; // ps/2 keyboard
const VEC_PIC_IRQ12: usize = 0x2c; // ps/2 mouse

const MASTER_PIC_ADDR: IoPortAddress = IoPortAddress::new(0x20);
const SLAVE_PIC_ADDR: IoPortAddress = IoPortAddress::new(0xa0);
const PIC_END_OF_INT_CMD: u8 = 0x20;

pub enum InterruptHandler {
    Normal(extern "x86-interrupt" fn()),
    PageFault(extern "x86-interrupt" fn(InterruptStackFrame, PageFaultErrorCode)),
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 4]
pub enum GateType {
    Interrupt = 0xe,
    Trap = 0xf,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
pub struct GateDescriptor {
    handler_offset_low: B16,
    selector: B16,
    // attributes
    int_stack_table: B3,
    #[skip]
    reserved0: B5,
    gate_type: GateType,
    #[skip]
    reserved1: B1,
    desc_privilege_level: B2,
    present: bool,

    handler_offset_middle: B16,
    handler_offset_high: B32,
    #[skip]
    reserved2: B32,
}

impl GateDescriptor {
    pub fn set_handler(&mut self, handler: InterruptHandler, cs: u16, gate_type: GateType) {
        let handler_addr = match handler {
            InterruptHandler::Normal(handler) => handler as *const () as u64,
            InterruptHandler::PageFault(handler) => handler as *const () as u64,
        };
        self.set_handler_offset_low(handler_addr as u16);
        self.set_handler_offset_middle((handler_addr >> 16) as u16);
        self.set_handler_offset_high((handler_addr >> 32) as u32);
        self.set_selector(cs);
        self.set_gate_type(gate_type);
        self.set_present(true);
    }
}

struct InterruptDescriptorTable {
    entries: [GateDescriptor; IDT_LEN],
}

impl InterruptDescriptorTable {
    pub const fn new() -> Self {
        Self {
            entries: [GateDescriptor::new(); IDT_LEN],
        }
    }

    pub fn set_handler(&mut self, vec_num: usize, handler: InterruptHandler, gate_type: GateType) {
        if vec_num >= IDT_LEN {
            return;
        }

        let mut desc = GateDescriptor::new();
        desc.set_handler(handler, asm::read_cs(), gate_type);
        self.entries[vec_num] = desc;
    }

    pub fn load(&self) {
        let limit = (size_of::<GateDescriptor>() * IDT_LEN - 1) as u16;
        let base = self.entries.as_ptr() as u64;
        let args = DescriptorTableArgs { limit, base };
        asm::lidt(&args);

        //info!("idt: Loaded IDT: {:?}", args);
        asm::sti();
    }
}

fn notify_end_of_int() {
    let virt_addr = VirtualAddress::new(END_OF_INT_REG_ADDR);
    virt_addr.write_volatile(0);
}

fn pic_notify_end_of_int() {
    MASTER_PIC_ADDR.out8(PIC_END_OF_INT_CMD);
    SLAVE_PIC_ADDR.out8(PIC_END_OF_INT_CMD);
}

extern "x86-interrupt" fn breakpoint_handler() {
    panic!("int: BREAKPOINT");
}

extern "x86-interrupt" fn general_protection_fault_handler() {
    panic!("int: GENERAL PROTECTION FAULT");
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    panic!(
        "int: PAGE FAULT, Accessed virtual address: 0x{:x}, Error code: {:?}, Stack frame: {:?}",
        Cr2::read().get(),
        error_code,
        stack_frame
    );
}

extern "x86-interrupt" fn double_fault_handler() {
    panic!("int: DOUBLE FAULT");
}

extern "x86-interrupt" fn xhc_primary_event_ring_handler() {
    if let Err(err) = xhc::on_updated_event_ring() {
        error!("xhc: {:?}", err);
    }

    notify_end_of_int();
}

extern "x86-interrupt" fn ps2_keyboard_handler() {
    ps2_keyboard::receive();
    pic_notify_end_of_int();
}

extern "x86-interrupt" fn ps2_mouse_handler() {
    ps2_mouse::receive();
    pic_notify_end_of_int();
}

pub fn init_pic() {
    // disallow all interrupts
    MASTER_PIC_ADDR.offset(1).out8(0xff);
    SLAVE_PIC_ADDR.offset(1).out8(0xff);

    // mapping IRQ0 - 7 to IDT entries 0x20 - 0x27
    MASTER_PIC_ADDR.offset(0).out8(0x11);
    MASTER_PIC_ADDR.offset(1).out8(0x20);
    MASTER_PIC_ADDR.offset(1).out8(1 << 2);
    MASTER_PIC_ADDR.offset(1).out8(0x1); // none buffer mode

    // mapping IRQ8 - 15 to IDT entries 0x28 - 0x2f
    SLAVE_PIC_ADDR.offset(0).out8(0x11); // edge trigger mode
    SLAVE_PIC_ADDR.offset(1).out8(0x28);
    SLAVE_PIC_ADDR.offset(1).out8(2);
    SLAVE_PIC_ADDR.offset(1).out8(0x1); // none buffer mode

    // mask all
    MASTER_PIC_ADDR.offset(1).out8(0xfb);
    SLAVE_PIC_ADDR.offset(1).out8(0xff);

    // allow interrupts
    MASTER_PIC_ADDR.offset(1).out8(0xf9);
    SLAVE_PIC_ADDR.offset(1).out8(0xef);

    info!("idt: Initialized PIC");
}

pub fn init_idt() {
    let mut idt = unsafe { IDT.try_lock() }.unwrap();
    idt.set_handler(
        VEC_BREAKPOINT,
        InterruptHandler::Normal(breakpoint_handler),
        GateType::Interrupt,
    );
    idt.set_handler(
        VEC_GENERAL_PROTECTION,
        InterruptHandler::Normal(general_protection_fault_handler),
        GateType::Interrupt,
    );
    idt.set_handler(
        VEC_PAGE_FAULT,
        InterruptHandler::PageFault(page_fault_handler),
        GateType::Interrupt,
    );
    idt.set_handler(
        VEC_DOUBLE_FAULT,
        InterruptHandler::Normal(double_fault_handler),
        GateType::Interrupt,
    );
    idt.set_handler(
        VEC_XHCI_INT,
        InterruptHandler::Normal(xhc_primary_event_ring_handler),
        GateType::Interrupt,
    );
    idt.set_handler(
        VEC_PIC_IRQ1,
        InterruptHandler::Normal(ps2_keyboard_handler),
        GateType::Interrupt,
    );
    idt.set_handler(
        VEC_PIC_IRQ12,
        InterruptHandler::Normal(ps2_mouse_handler),
        GateType::Interrupt,
    );
    idt.load();

    info!("idt: Initialized IDT");
}
