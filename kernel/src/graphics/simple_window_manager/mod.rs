use super::{
    color::*,
    draw::Draw,
    frame_buf,
    multi_layer::{self, LayerId, LayerPositionInfo},
};
use crate::{
    device::ps2_mouse::MouseEvent, error::Result, fs::file::bitmap::BitmapImage, util::mutex::Mutex,
};
use alloc::{string::String, vec::Vec};
use components::Image;

pub mod components;

static mut SIMPLE_WM: Mutex<Option<SimpleWindowManager>> = Mutex::new(None);

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleWindowManagerError {
    NotInitialized,
    MousePointerLayerWasNotFound,
    TaskbarLayerWasNotFound,
}

#[derive(Debug)]
struct Window {
    pub layer_id: LayerId,
    pub title: String,
}

struct Taskbar {
    pub layer_id: LayerId,
}

struct SimpleWindowManager {
    windows: Vec<Window>,
    taskbar: Option<Taskbar>,
    mouse_pointer: Option<Image>,
    res_x: usize,
    res_y: usize,
}

impl SimpleWindowManager {
    pub fn new(res_x: usize, res_y: usize) -> Self {
        Self {
            windows: Vec::new(),
            taskbar: None,
            mouse_pointer: None,
            res_x,
            res_y,
        }
    }

    pub fn create_mouse_pointer(&mut self, pointer_bmp: &BitmapImage) -> Result<()> {
        self.mouse_pointer = Some(Image::new(pointer_bmp, 0, 0, true)?);

        Ok(())
    }

    pub fn create_taskbar(&mut self) -> Result<()> {
        if let Some(layer_id) = self.taskbar.as_ref().map(|t| &t.layer_id) {
            multi_layer::remove_layer(&layer_id)?;
        }

        let width = self.res_x;
        let height = 30;
        let mut taskbar_layer = multi_layer::create_layer(0, self.res_y - height, width, height)?;
        taskbar_layer.fill(COLOR_SILVER)?;
        let taskbar_layer_id = taskbar_layer.id.clone();
        multi_layer::push_layer(taskbar_layer)?;
        self.taskbar = Some(Taskbar {
            layer_id: taskbar_layer_id,
        });

        Ok(())
    }

    pub fn move_mouse_pointer(&mut self, mouse_event: MouseEvent) -> Result<()> {
        let layer_id = &self
            .mouse_pointer
            .as_ref()
            .ok_or(SimpleWindowManagerError::MousePointerLayerWasNotFound)?
            .layer_id;

        // move mouse pointer
        multi_layer::move_layer(layer_id, mouse_event.x_pos, mouse_event.y_pos)?;

        let LayerPositionInfo {
            x: mouse_x,
            y: mouse_y,
            width: _,
            height: _,
        } = multi_layer::get_layer_pos_info(layer_id)?;

        // drag window
        if mouse_event.left {
            for w in self.windows.iter().rev() {
                let LayerPositionInfo {
                    x,
                    y,
                    width,
                    height,
                } = multi_layer::get_layer_pos_info(&w.layer_id)?;

                if mouse_x >= x && mouse_x <= x + width && mouse_y >= y && mouse_y <= y + height {
                    multi_layer::move_layer(&w.layer_id, mouse_x - x, mouse_y - y)?;
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn create_window(
        &mut self,
        title: String,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<()> {
        let mut window_layer = multi_layer::create_layer(x, y, width, height)?;
        window_layer.fill(COLOR_SILVER)?;
        window_layer.draw_rect(0, 0, width, 20, COLOR_BLUE)?;
        window_layer.draw_string(0, 0, &title, COLOR_WHITE)?;
        let window_layer_id = window_layer.id.clone();
        multi_layer::push_layer(window_layer)?;
        let window = Window {
            layer_id: window_layer_id,
            title,
        };
        self.windows.push(window);
        let _ = self.update_taskbar()?;

        Ok(())
    }

    fn update_taskbar(&mut self) -> Result<()> {
        let layer_id = &self
            .taskbar
            .as_ref()
            .ok_or(SimpleWindowManagerError::TaskbarLayerWasNotFound)?
            .layer_id;

        let s = format!("{:?}", self.windows);
        multi_layer::draw_layer(layer_id, |l| {
            l.fill(COLOR_SILVER)?;
            l.draw_string(0, 0, &s, COLOR_BLACK)?;
            Ok(())
        })?;

        Ok(())
    }
}

pub fn init() -> Result<()> {
    let (res_x, res_y) = frame_buf::get_resolution()?;
    *unsafe { SIMPLE_WM.try_lock() }? = Some(SimpleWindowManager::new(res_x, res_y));
    Ok(())
}

pub fn create_mouse_pointer(pointer_bmp: &BitmapImage) -> Result<()> {
    unsafe { SIMPLE_WM.try_lock() }?
        .as_mut()
        .ok_or(SimpleWindowManagerError::NotInitialized)?
        .create_mouse_pointer(pointer_bmp)
}

pub fn create_taskbar() -> Result<()> {
    unsafe { SIMPLE_WM.try_lock() }?
        .as_mut()
        .ok_or(SimpleWindowManagerError::NotInitialized)?
        .create_taskbar()
}

pub fn move_mouse_pointer(mouse_event: MouseEvent) -> Result<()> {
    unsafe { SIMPLE_WM.try_lock() }?
        .as_mut()
        .ok_or(SimpleWindowManagerError::NotInitialized)?
        .move_mouse_pointer(mouse_event)
}

pub fn create_window(title: String, x: usize, y: usize, width: usize, height: usize) -> Result<()> {
    unsafe { SIMPLE_WM.try_lock() }?
        .as_mut()
        .ok_or(SimpleWindowManagerError::NotInitialized)?
        .create_window(title, x, y, width, height)
}
