use super::{DeviceDriverFunction, DeviceDriverInfo};
use crate::{
    acpi,
    addr::VirtualAddress,
    error::Result,
    graphics::{frame_buf, multi_layer},
    idt::{self, GateType, InterruptHandler},
    task,
    util::mutex::Mutex,
};
use alloc::vec::Vec;
use log::{debug, info};

const LVT_TIMER_VIRT_ADDR: VirtualAddress = VirtualAddress::new(0xfee00320);
const INIT_CNT_VIRT_ADDR: VirtualAddress = VirtualAddress::new(0xfee00380);
const CURR_CNT_VIRT_ADDR: VirtualAddress = VirtualAddress::new(0xfee00390);
const DIV_CONF_VIRT_ADDR: VirtualAddress = VirtualAddress::new(0xfee003e0);
const END_OF_INT_REG_ADDR: VirtualAddress = VirtualAddress::new(0xfee000b0);

const DIV_VALUE: DivideValue = DivideValue::By4;
const INT_INTERVAL_MS: usize = 10; // must be >= 10ms

#[allow(dead_code)]
#[derive(Debug)]
#[repr(u8)]
enum DivideValue {
    By1 = 0b1011,
    By2 = 0b0000,
    By4 = 0b0001,
    By8 = 0b0010,
    By16 = 0b0011,
    By32 = 0b1000,
    By64 = 0b1001,
    By128 = 0b1010,
}

static mut LOCAL_APIC_TIMER_DRIVER: Mutex<LocalApicTimerDriver> =
    Mutex::new(LocalApicTimerDriver::new());

struct LocalApicTimerDriver {
    device_driver_info: DeviceDriverInfo,
    tick: usize,
    freq: Option<usize>,
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
        let init_cnt = if let Some(freq) = self.freq {
            (freq / 1000 * INT_INTERVAL_MS) as u32
        } else {
            u32::MAX // -1
        };

        (INIT_CNT_VIRT_ADDR.as_ptr_mut() as *mut u32).write_volatile(init_cnt);
    }

    unsafe fn stop(&self) {
        (INIT_CNT_VIRT_ADDR.as_ptr_mut() as *mut u32).write_volatile(0);
    }

    unsafe fn tick(&self) -> usize {
        if self.freq.is_some() {
            return self.tick;
        }

        let current_cnt = (CURR_CNT_VIRT_ADDR.as_ptr_mut() as *mut u32).read_volatile();
        u32::MAX as usize - current_cnt as usize
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
        let device_name = self.device_driver_info.name;

        // register interrupt handler
        let vec_num = idt::set_handler_dyn_vec(
            InterruptHandler::Normal(poll_int_local_apic_timer),
            GateType::Interrupt,
        )?;
        debug!("{}: Interrupt vector number: 0x{:x}", device_name, vec_num);

        unsafe {
            // calc freq
            self.stop();
            (DIV_CONF_VIRT_ADDR.as_ptr_mut() as *mut u32).write_volatile(DIV_VALUE as u32);
            (LVT_TIMER_VIRT_ADDR.as_ptr_mut() as *mut u32)
                .write_volatile((2 << 16) | vec_num as u32);
            // non masked, periodic
            self.start();
            acpi::pm_timer_wait_ms(1000)?; // wait 1 sec
            let tick = self.tick();
            self.stop();

            assert!(tick > 0);
            debug!(
                "{}: Timer frequency was detected: {}Hz ({:?})",
                device_name, tick, DIV_VALUE
            );

            self.freq = Some(tick);

            // start timer
            self.start();
        }

        self.device_driver_info.attached = true;
        Ok(())
    }

    fn poll_normal(&mut self) -> Result<Self::PollNormalOutput> {
        unimplemented!()
    }

    fn poll_int(&mut self) -> Result<Self::PollInterruptOutput> {
        if !self.device_driver_info.attached {
            return Ok(());
        }

        if self.tick == usize::MAX {
            self.tick = 0;
        } else {
            self.tick += 1;
        }

        let _ = multi_layer::draw_to_frame_buf();
        let _ = frame_buf::apply_shadow_buf();
        let _ = task::poll();

        Ok(())
    }

    fn open(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn close(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn read(&mut self) -> Result<Vec<u8>> {
        unimplemented!()
    }

    fn write(&mut self, _data: &[u8]) -> Result<()> {
        unimplemented!()
    }
}

pub fn get_device_driver_info() -> Result<DeviceDriverInfo> {
    let driver = unsafe { LOCAL_APIC_TIMER_DRIVER.try_lock() }?;
    driver.get_device_driver_info()
}

pub fn probe_and_attach() -> Result<()> {
    let mut driver = unsafe { LOCAL_APIC_TIMER_DRIVER.try_lock() }?;
    driver.probe()?;
    driver.attach(())?;
    info!("{}: Attached!", driver.device_driver_info.name);

    Ok(())
}

pub fn open() -> Result<()> {
    let mut driver = unsafe { LOCAL_APIC_TIMER_DRIVER.try_lock() }?;
    driver.open()
}

pub fn close() -> Result<()> {
    let mut driver = unsafe { LOCAL_APIC_TIMER_DRIVER.try_lock() }?;
    driver.close()
}

pub fn read() -> Result<Vec<u8>> {
    let mut driver = unsafe { LOCAL_APIC_TIMER_DRIVER.try_lock() }?;
    driver.read()
}

pub fn write(data: &[u8]) -> Result<()> {
    let mut driver = unsafe { LOCAL_APIC_TIMER_DRIVER.try_lock() }?;
    driver.write(data)
}

pub fn get_current_ms() -> Result<usize> {
    let driver = unsafe { LOCAL_APIC_TIMER_DRIVER.try_lock() }?;
    let _freq = driver.freq.ok_or("Frequency not set")?;
    let current_tick = unsafe { driver.tick() };
    Ok(current_tick * INT_INTERVAL_MS)
}

extern "x86-interrupt" fn poll_int_local_apic_timer() {
    unsafe {
        let driver = LOCAL_APIC_TIMER_DRIVER.get_force_mut();
        let _ = driver.poll_int();

        // notify end of interrupt
        (END_OF_INT_REG_ADDR.as_ptr_mut() as *mut u32).write_volatile(0);
    }
}
