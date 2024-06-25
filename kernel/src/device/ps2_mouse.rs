use crate::{
    arch::addr::IoPortAddress,
    error::Result,
    println,
    util::{fifo::Fifo, mutex::Mutex},
};

const PS2_DATA_REG_ADDR: IoPortAddress = IoPortAddress::new(0x60);
const PS2_CMD_AND_STATE_REG_ADDR: IoPortAddress = IoPortAddress::new(0x64);

static mut MOUSE: Mutex<Mouse> = Mutex::new(Mouse::new());

#[derive(Default, Debug)]
pub struct MouseEvent {
    pub middle: bool,
    pub right: bool,
    pub left: bool,
    pub rel_x: isize,
    pub rel_y: isize,
}

struct Mouse {
    data_buf: Fifo<u8, 128>,
    data_0: Option<u8>,
    data_1: Option<u8>,
    data_2: Option<u8>,
}

impl Mouse {
    pub const fn new() -> Self {
        Self {
            data_buf: Fifo::new(0),
            data_0: None,
            data_1: None,
            data_2: None,
        }
    }

    pub fn receive(&mut self, data: u8) -> Result<()> {
        if self.data_buf.enqueue(data).is_err() {
            self.data_buf.reset_ptr();
            self.data_buf.enqueue(data)?;
        }

        Ok(())
    }

    pub fn get_event(&mut self) -> Result<Option<MouseEvent>> {
        fn is_valid_data_0(data: u8) -> bool {
            data & 0x08 != 0
        }

        let data = self.data_buf.dequeue()?;
        if data == 0xfa {
            self.data_0 = None;
            self.data_1 = None;
            self.data_2 = None;
            return Ok(None);
        }

        if self.data_0.is_none() && is_valid_data_0(data) {
            self.data_0 = Some(data);
        } else if self.data_1.is_none() {
            self.data_1 = Some(data);
        } else if self.data_2.is_none() {
            self.data_2 = Some(data);
        } else if is_valid_data_0(data) {
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

            if x_of || y_of {
                return Ok(None);
            }

            let rel_x = -(data_1 as isize - (((data_0 as isize) << 4) & 0x100));
            let rel_y = data_2 as isize - (((data_0 as isize) << 3) & 0x100);

            //println!("{}:{}", rel_x, rel_y);

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
}

pub fn init() {
    // send next wrote byte to ps/2 secondary port
    PS2_CMD_AND_STATE_REG_ADDR.out8(0xd4);
    wait_ready();

    // init mouse
    PS2_DATA_REG_ADDR.out8(0xff);
    wait_ready();

    PS2_CMD_AND_STATE_REG_ADDR.out8(0xd4);
    wait_ready();

    // start streaming
    PS2_DATA_REG_ADDR.out8(0xf4);
    wait_ready();
}

pub fn receive() -> Result<()> {
    let data = PS2_DATA_REG_ADDR.in8();
    unsafe { MOUSE.try_lock() }?.receive(data)
}

pub fn get_event() -> Result<Option<MouseEvent>> {
    unsafe { MOUSE.try_lock() }?.get_event()
}

fn wait_ready() {
    while PS2_CMD_AND_STATE_REG_ADDR.in8() & 0x2 != 0 {
        continue;
    }
}
