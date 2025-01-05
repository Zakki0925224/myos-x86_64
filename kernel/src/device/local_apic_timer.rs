use super::{DeviceDriverFunction, DeviceDriverInfo};
use crate::{
    acpi,
    addr::VirtualAddress,
    error::Result,
    graphics::{frame_buf, multi_layer},
    idt::{self, GateType, InterruptHandler},
    task,
};
use core::num::{NonZero, NonZeroUsize};
use log::{debug, info};

const LVT_TIMER_VIRT_ADDR: VirtualAddress = VirtualAddress::new(0xfee00320);
const INIT_CNT_VIRT_ADDR: VirtualAddress = VirtualAddress::new(0xfee00380);
const DIV_CONF_VIRT_ADDR: VirtualAddress = VirtualAddress::new(0xfee003e0);

const END_OF_INT_REG_ADDR: VirtualAddress = VirtualAddress::new(0xfee000b0);

static mut LOCAL_APIC_TIMER_DRIVER: LocalApicTimerDriver = LocalApicTimerDriver::new();

struct LocalApicTimerDriver {
    device_driver_info: DeviceDriverInfo,
    tick: usize,
    freq: Option<NonZero<usize>>,
}

impl LocalApicTimerDriver {
    const fn new() -> Self {
        Self {
            device_driver_info: DeviceDriverInfo::new("local-apic-timer"),
            tick: 0,
            freq: None,
        }
    }

    unsafe fn start(&self) {
        (INIT_CNT_VIRT_ADDR.as_ptr_mut() as *mut u32).write_volatile(0x0010_0000);
    }

    unsafe fn stop(&self) {
        (INIT_CNT_VIRT_ADDR.as_ptr_mut() as *mut u32).write_volatile(0);
    }

    fn inc_tick(&mut self) {
        if self.tick == usize::MAX {
            self.tick = 0;
        }

        self.tick += 1;
    }

    fn tick(&self) -> usize {
        self.tick
    }
}

impl DeviceDriverFunction for LocalApicTimerDriver {
    type AttachInput = ();
    type PollNormalOutput = ();
    type PollInterruptOutput = ();

    fn get_device_driver_info(&self) -> Result<DeviceDriverInfo> {
        Ok(self.device_driver_info.clone())
    }

    fn probe(&mut self) -> Result<()> {
        Ok(())
    }

    fn attach(&mut self, _arg: Self::AttachInput) -> Result<()> {
        // register interrupt handler
        let vec_num = idt::set_handler_dyn_vec(
            InterruptHandler::Normal(poll_int_local_apic_timer),
            GateType::Interrupt,
        )?;
        debug!(
            "{}: Interrupt vector number: 0x{:x}",
            self.device_driver_info.name, vec_num
        );

        unsafe {
            // calc freq
            self.stop();
            (DIV_CONF_VIRT_ADDR.as_ptr_mut() as *mut u32).write_volatile(0b1011);
            (LVT_TIMER_VIRT_ADDR.as_ptr_mut() as *mut u32)
                .write_volatile((2 << 16) | vec_num as u32);
            // non masked, periodic
            self.start();
            acpi::pm_timer_wait_ms(10)?;
            let tick = self.tick();
            self.stop();

            self.freq = NonZeroUsize::new(tick);
            self.tick = 0;

            // start timer
            self.start();
        }

        Ok(())
    }

    fn poll_normal(&mut self) -> Result<Self::PollNormalOutput> {
        unimplemented!()
    }

    fn poll_int(&mut self) -> Result<Self::PollInterruptOutput> {
        self.inc_tick();

        if get_current_ms().is_some() {
            let _ = multi_layer::draw_to_frame_buf();
            let _ = frame_buf::apply_shadow_buf();

            let _ = task::poll();
        }

        Ok(())
    }
}

pub fn get_device_driver_info() -> Result<DeviceDriverInfo> {
    unsafe { LOCAL_APIC_TIMER_DRIVER.get_device_driver_info() }
}

pub fn probe_and_attach() -> Result<()> {
    unsafe {
        LOCAL_APIC_TIMER_DRIVER.probe()?;
        LOCAL_APIC_TIMER_DRIVER.attach(())?;
        info!("{}: Attached!", get_device_driver_info()?.name);
    }

    Ok(())
}

pub fn get_current_tick() -> usize {
    unsafe { LOCAL_APIC_TIMER_DRIVER.tick() }
}

pub fn get_current_ms() -> Option<usize> {
    let freq = unsafe { LOCAL_APIC_TIMER_DRIVER.freq }?;
    Some(get_current_tick() / freq * 10)
}

extern "x86-interrupt" fn poll_int_local_apic_timer() {
    unsafe {
        let _ = LOCAL_APIC_TIMER_DRIVER.poll_int();

        // notify end of interrupt
        (END_OF_INT_REG_ADDR.as_ptr_mut() as *mut u32).write_volatile(0);
    }
}
