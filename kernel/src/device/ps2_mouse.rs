use super::{DeviceDriverFunction, DeviceDriverInfo};
use crate::{
    arch::{self, addr::IoPortAddress},
    error::{Error, Result},
    idt,
    util::{fifo::Fifo, mutex::Mutex},
};
use log::info;

const PS2_DATA_REG_ADDR: IoPortAddress = IoPortAddress::new(0x60);
const PS2_CMD_AND_STATE_REG_ADDR: IoPortAddress = IoPortAddress::new(0x64);

static mut PS2_MOUSE_DRIVER: Mutex<Ps2MouseDriver> = Mutex::new(Ps2MouseDriver::new());

#[derive(Default, Debug)]
pub struct MouseEvent {
    pub middle: bool,
    pub right: bool,
    pub left: bool,
    pub rel_x: i16,
    pub rel_y: i16,
}

struct Ps2MouseDriver {
    device_driver_info: DeviceDriverInfo,
    data_buf: Fifo<u8, 128>,
    data_0: Option<u8>,
    data_1: Option<u8>,
    data_2: Option<u8>,
}

impl Ps2MouseDriver {
    const fn new() -> Self {
        Self {
            device_driver_info: DeviceDriverInfo::new("ps2-mouse"),
            data_buf: Fifo::new(0),
            data_0: None,
            data_1: None,
            data_2: None,
        }
    }

    fn receive(&mut self, data: u8) -> Result<()> {
        self.data_buf.enqueue(data)
    }

    fn get_event(&mut self) -> Result<Option<MouseEvent>> {
        fn is_data0_valid(data_0: u8) -> bool {
            data_0 & 0x08 != 0
        }

        let data = self.data_buf.dequeue()?;

        if data == 0xfa {
            self.data_0 = None;
            self.data_1 = None;
            self.data_2 = None;
            return Ok(None);
        }

        if self.data_0.is_none() && is_data0_valid(data) {
            self.data_0 = Some(data);
        } else if self.data_1.is_none() {
            self.data_1 = Some(data);
        } else if self.data_2.is_none() {
            self.data_2 = Some(data);
        } else if is_data0_valid(data) {
            self.data_0 = Some(data);
            self.data_1 = None;
            self.data_2 = None;
        }

        if let (Some(data_0), Some(data_1), Some(data_2)) = (self.data_0, self.data_1, self.data_2)
        {
            let button_m = data_0 & 0x4 != 0;
            let button_r = data_0 & 0x2 != 0;
            let button_l = data_0 & 0x1 != 0;
            let x_of = data_0 & 0x40 != 0;
            let y_of = data_0 & 0x80 != 0;
            let x_sign = data_0 & 0x10 != 0;
            let y_sign = data_0 & 0x20 != 0;

            if x_of || y_of {
                return Ok(None);
            }

            let mut rel_x = data_1 as i16;
            let mut rel_y = data_2 as i16;

            if x_sign {
                rel_x |= !0xff;
            }

            if y_sign {
                rel_y |= !0xff;
            }

            rel_y = -rel_y;

            let e = MouseEvent {
                middle: button_m,
                right: button_r,
                left: button_l,
                rel_x,
                rel_y,
            };

            return Ok(Some(e));
        }

        Ok(None)
    }

    fn wait_ready(&self) {
        while PS2_CMD_AND_STATE_REG_ADDR.in8() & 0x2 != 0 {
            continue;
        }
    }
}

impl DeviceDriverFunction for Ps2MouseDriver {
    type PollNormalOutput = Option<MouseEvent>;
    type PollInterruptOutput = ();

    fn get_device_driver_info(&self) -> Result<DeviceDriverInfo> {
        Ok(self.device_driver_info.clone())
    }

    fn probe(&mut self) -> Result<()> {
        Ok(())
    }

    fn attach(&mut self) -> Result<()> {
        // send next wrote byte to ps/2 secondary port
        PS2_CMD_AND_STATE_REG_ADDR.out8(0xd4);
        self.wait_ready();

        // init mouse
        PS2_DATA_REG_ADDR.out8(0xff);
        self.wait_ready();

        PS2_CMD_AND_STATE_REG_ADDR.out8(0xd4);
        self.wait_ready();

        // start streaming
        PS2_DATA_REG_ADDR.out8(0xf4);
        self.wait_ready();

        self.device_driver_info.attached = true;
        Ok(())
    }

    fn poll_normal(&mut self) -> Result<Self::PollNormalOutput> {
        if !self.device_driver_info.attached {
            return Err(Error::Failed("Device driver is not attached"));
        }

        self.get_event()
    }

    fn poll_int(&mut self) -> Result<Self::PollInterruptOutput> {
        if !self.device_driver_info.attached {
            return Err(Error::Failed("Device driver is not attached"));
        }

        let data = PS2_DATA_REG_ADDR.in8();
        self.receive(data)?;

        Ok(())
    }
}

pub fn get_device_driver_info() -> Result<DeviceDriverInfo> {
    let driver = unsafe { PS2_MOUSE_DRIVER.try_lock() }?;
    driver.get_device_driver_info()
}

pub fn probe_and_attach() -> Result<()> {
    arch::disabled_int(|| {
        let mut driver = unsafe { PS2_MOUSE_DRIVER.try_lock() }?;
        driver.probe()?;
        driver.attach()?;
        info!("{}: Attached!", driver.get_device_driver_info()?.name);
        Result::Ok(())
    })
}

pub fn poll_normal() -> Result<Option<MouseEvent>> {
    let mouse_event = arch::disabled_int(|| {
        let mut driver = unsafe { PS2_MOUSE_DRIVER.try_lock() }?;
        driver.poll_normal()
    })?;
    Ok(mouse_event)
}

pub extern "x86-interrupt" fn poll_int_ps2_mouse_driver() {
    if let Ok(mut driver) = unsafe { PS2_MOUSE_DRIVER.try_lock() } {
        let _ = driver.poll_int();
    }
    idt::pic_notify_end_of_int();
}
