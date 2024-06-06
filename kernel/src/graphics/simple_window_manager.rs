use super::{
    color::{COLOR_BLUE, COLOR_SILVER},
    draw::Draw,
    multi_layer::{self, LayerPositionInfo},
};
use crate::{
    device::ps2_mouse::MouseEvent,
    error::{Error, Result},
    fs::file::bitmap::BitmapImage,
    util::mutex::{Mutex, MutexError},
};
use alloc::{string::String, vec::Vec};

static mut SIMPLE_WM: Mutex<Option<SimpleWindowManager>> = Mutex::new(None);

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleWindowManagerError {
    NotInitialized,
    MousePointerLayerWasNotFound,
}

struct Window {
    // x: usize,
    // y: usize,
    // width: usize,
    // height: usize,
    pub layer_id: usize,
    pub title: String,
}

struct SimpleWindowManager {
    windows: Vec<Window>,
    mouse_pointer_layer_id: Option<usize>,
}

impl SimpleWindowManager {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            mouse_pointer_layer_id: None,
        }
    }

    pub fn create_mouse_pointer_layer(&mut self, pointer_bmp: &BitmapImage) -> Result<()> {
        if let Some(layer_id) = self.mouse_pointer_layer_id {
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
        self.mouse_pointer_layer_id = Some(pointer_layer_id);

        Ok(())
    }

    pub fn move_mouse_pointer(&mut self, mouse_event: MouseEvent) -> Result<()> {
        let layer_id = match self.mouse_pointer_layer_id {
            Some(id) => id,
            None => return Err(SimpleWindowManagerError::MousePointerLayerWasNotFound.into()),
        };

        // drag window
        if mouse_event.left {
            let LayerPositionInfo {
                x: mouse_x,
                y: mouse_y,
                width: _,
                height: _,
            } = multi_layer::get_layer_pos_info(layer_id)?;

            for w in self.windows.iter().rev() {
                let LayerPositionInfo {
                    x,
                    y,
                    width,
                    height,
                } = multi_layer::get_layer_pos_info(w.layer_id)?;

                if mouse_x >= x && mouse_x <= x + width && mouse_y >= y && mouse_y <= y + height {
                    multi_layer::move_layer(w.layer_id, mouse_event.x_pos, mouse_event.y_pos)?;
                    break;
                }
            }
        }

        multi_layer::move_layer(layer_id, mouse_event.x_pos, mouse_event.y_pos)?;

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
        let window_layer_id = window_layer.id;
        multi_layer::push_layer(window_layer)?;
        let window = Window {
            layer_id: window_layer_id,
            title,
        };
        self.windows.push(window);

        Ok(())
    }
}

pub fn init() -> Result<()> {
    if let Ok(mut simple_wm) = unsafe { SIMPLE_WM.try_lock() } {
        *simple_wm = Some(SimpleWindowManager::new());
        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn create_mouse_pointer_layer(pointer_bmp: &BitmapImage) -> Result<()> {
    if let Ok(mut simple_wm) = unsafe { SIMPLE_WM.try_lock() } {
        simple_wm
            .as_mut()
            .ok_or(SimpleWindowManagerError::NotInitialized)?
            .create_mouse_pointer_layer(pointer_bmp)
    } else {
        Err(MutexError::Locked.into())
    }
}

pub fn move_mouse_pointer(mouse_event: MouseEvent) -> Result<()> {
    if let Ok(mut simple_wm) = unsafe { SIMPLE_WM.try_lock() } {
        simple_wm
            .as_mut()
            .ok_or(SimpleWindowManagerError::NotInitialized)?
            .move_mouse_pointer(mouse_event)
    } else {
        Err(MutexError::Locked.into())
    }
}

pub fn create_window(title: String, x: usize, y: usize, width: usize, height: usize) -> Result<()> {
    if let Ok(mut simple_wm) = unsafe { SIMPLE_WM.try_lock() } {
        simple_wm
            .as_mut()
            .ok_or(SimpleWindowManagerError::NotInitialized)?
            .create_window(title, x, y, width, height)
    } else {
        Err(MutexError::Locked.into())
    }
}
