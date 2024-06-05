use crate::{
    arch::addr::IoPortAddress,
    error::Result,
    util::{
        fifo::Fifo,
        mutex::{Mutex, MutexError},
    },
};
use common::graphic_info::GraphicInfo;

const PS2_DATA_REG_ADDR: IoPortAddress = IoPortAddress::new(0x60);
const PS2_CMD_AND_STATE_REG_ADDR: IoPortAddress = IoPortAddress::new(0x64);

static mut MOUSE: Mutex<Mouse> = Mutex::new(Mouse::new());

#[derive(Default, Debug)]
pub struct MouseEvent {
    pub middle: bool,
    pub right: bool,
    pub left: bool,
    pub x_pos: usize,
    pub y_pos: usize,
}

struct Mouse {
    x_max: usize,
    y_max: usize,
    x: usize,
    y: usize,
    data_buf: Fifo<u8, 128>,
    data_0: Option<u8>,
    data_1: Option<u8>,
    data_2: Option<u8>,
}

impl Mouse {
    pub const fn new() -> Self {
        Self {
            x_max: 0,
            y_max: 0,
            x: 0,
            y: 0,
            data_buf: Fifo::new(0),
            data_0: None,
            data_1: None,
            data_2: None,
        }
    }

    pub fn init(&mut self, graphic_info: &GraphicInfo) {
        let (res_x, res_y) = graphic_info.resolution;

        self.x_max = res_x as usize;
        self.y_max = res_y as usize;
    }

    pub fn receive(&mut self, data: u8) -> Result<()> {
        self.data_buf.enqueue(data)?;
        Ok(())
    }

    pub fn read(&mut self) -> Result<Option<MouseEvent>> {
        fn is_valid_data_0(data: u8) -> bool {
            data & 0xc8 == 0x08
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
            let move_x = -(data_1 as i8);
            let move_y = data_2 as i8;

            // x
            if move_x < 0 {
                let move_x = -move_x as usize;
                if self.x >= move_x {
                    self.x -= move_x;
                }
            } else {
                let move_x = move_x as usize;
                if self.x + move_x < self.x_max {
                    self.x += move_x;
                }
            }

            // y
            if move_y < 0 {
                let move_y = -move_y as usize;
                if self.y >= move_y {
                    self.y -= move_y;
                }
            } else {
                let move_y = move_y as usize;
                if self.y + move_y < self.y_max {
                    self.y += move_y;
                }
            }

            return Ok(Some(MouseEvent {
                middle: button_m,
                right: button_r,
                left: button_l,
                x_pos: self.x,
                y_pos: self.y,
            }));
        }

        Ok(None)
    }

    fn reset_data_buf(&mut self) {
        self.data_buf = Fifo::new(0);
    }
}

pub fn init(graphic_info: &GraphicInfo) -> Result<()> {
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

    if let Ok(mut mouse) = unsafe { MOUSE.try_lock() } {
        mouse.init(graphic_info);
        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn receive() -> Result<()> {
    let data = PS2_DATA_REG_ADDR.in8();

    if let Ok(mut mouse) = unsafe { MOUSE.try_lock() } {
        return mouse.receive(data);
    }

    Err(MutexError::Locked.into())
}

pub fn update() -> Result<Option<MouseEvent>> {
    if let Ok(mut mouse) = unsafe { MOUSE.try_lock() } {
        return mouse.read();
    }

    Err(MutexError::Locked.into())
}

fn wait_ready() {
    while PS2_CMD_AND_STATE_REG_ADDR.in8() & 0x2 != 0 {
        continue;
    }
}
