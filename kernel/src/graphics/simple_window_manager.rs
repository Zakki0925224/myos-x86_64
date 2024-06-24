use super::{
    color::*,
    draw::Draw,
    frame_buf,
    multi_layer::{self, LayerPositionInfo},
};
use crate::{
    device::ps2_mouse::MouseEvent,
    error::{Error, Result},
    fs::file::bitmap::BitmapImage,
    util::mutex::Mutex,
};
use alloc::{string::String, vec::Vec};

static mut SIMPLE_WM: Mutex<Option<SimpleWindowManager>> = Mutex::new(None);

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleWindowManagerError {
    NotInitialized,
    MousePointerLayerWasNotFound,
    TaskbarLayerWasNotFound,
}

#[derive(Debug)]
struct Window {
    pub layer_id: usize,
    pub title: String,
}

struct Taskbar {
    pub layer_id: usize,
}

struct MousePointer {
    pub layer_id: usize,
}

struct SimpleWindowManager {
    windows: Vec<Window>,
    taskbar: Option<Taskbar>,
    mouse_pointer: Option<MousePointer>,
}

impl SimpleWindowManager {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            taskbar: None,
            mouse_pointer: None,
        }
    }

    pub fn create_mouse_pointer(&mut self, pointer_bmp: &BitmapImage) -> Result<()> {
        if let Some(layer_id) = self.mouse_pointer.as_ref().map(|m| m.layer_id) {
            multi_layer::remove_layer(layer_id)?;
        }

        if !pointer_bmp.is_valid() {
            return Err(Error::Failed("Invalid bitmap image"));
        }

        let mut pointer_layer = multi_layer::create_layer_from_bitmap_image(0, 0, pointer_bmp)
            .unwrap_or({
                let mut layer = multi_layer::create_layer(0, 0, 5, 14)?;
                layer.fill(COLOR_SILVER)?;
                layer
            });
        pointer_layer.always_on_top = true;
        let pointer_layer_id = pointer_layer.id;
        multi_layer::push_layer(pointer_layer)?;
        self.mouse_pointer = Some(MousePointer {
            layer_id: pointer_layer_id,
        });

        Ok(())
    }

    pub fn create_taskbar(&mut self) -> Result<()> {
        if let Some(layer_id) = self.taskbar.as_ref().map(|t| t.layer_id) {
            multi_layer::remove_layer(layer_id)?;
        }

        let (res_x, res_y) = frame_buf::get_resolution()?;

        let width = res_x;
        let height = 30;
        let mut taskbar_layer = multi_layer::create_layer(0, res_y - height, width, height)?;
        taskbar_layer.fill(COLOR_SILVER)?;
        let taskbar_layer_id = taskbar_layer.id;
        multi_layer::push_layer(taskbar_layer)?;
        self.taskbar = Some(Taskbar {
            layer_id: taskbar_layer_id,
        });

        Ok(())
    }

    pub fn move_mouse_pointer(&mut self, mouse_event: MouseEvent) -> Result<()> {
        let layer_id = self
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
                } = multi_layer::get_layer_pos_info(w.layer_id)?;

                if mouse_x >= x && mouse_x <= x + width && mouse_y >= y && mouse_y <= y + height {
                    multi_layer::move_layer(w.layer_id, mouse_x - x, mouse_y - y)?;
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
        let window_layer_id = window_layer.id;
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
        let layer_id = self
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
    *unsafe { SIMPLE_WM.try_lock() }? = Some(SimpleWindowManager::new());
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
