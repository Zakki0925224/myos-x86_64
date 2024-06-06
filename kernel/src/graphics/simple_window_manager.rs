use super::{color::COLOR_SILVER, draw::Draw, multi_layer};
use crate::{
    device::ps2_mouse::MouseEvent,
    error::{Error, Result},
    fs::file::bitmap::BitmapImage,
    util::mutex::{Mutex, MutexError},
};
use alloc::vec::Vec;

static mut SIMPLE_WM: Mutex<Option<SimpleWindowManager>> = Mutex::new(None);

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleWindowManagerError {
    NotInitialized,
    MousePointerLayerWasNotFound,
}

struct Window {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    layer_id: usize,
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
        multi_layer::move_layer(layer_id, mouse_event.x_pos, mouse_event.y_pos)?;

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
