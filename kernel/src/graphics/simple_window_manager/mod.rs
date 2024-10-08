use super::{
    frame_buf,
    multi_layer::{self, LayerPositionInfo},
};
use crate::{
    device::ps2_mouse::MouseEvent,
    error::Result,
    fs::file::bitmap::BitmapImage,
    util::{mutex::Mutex, theme::GLOBAL_THEME},
};
use alloc::{string::String, vec::Vec};
use components::*;

pub mod components;

const MOUSE_POINTER_MOVE_THRESHOLD: isize = 40;

static mut SIMPLE_WM: Mutex<Option<SimpleWindowManager>> = Mutex::new(None);

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleWindowManagerError {
    NotInitialized,
    MousePointerLayerWasNotFound,
    TaskbarLayerWasNotFound,
}

struct SimpleWindowManager {
    windows: Vec<Window>,
    taskbar: Option<Panel>,
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
        self.mouse_pointer = Some(Image::create_and_push(pointer_bmp, 0, 0, true)?);

        Ok(())
    }

    pub fn create_taskbar(&mut self) -> Result<()> {
        let width = self.res_x;
        let height = 30;
        let panel = Panel::create_and_push(0, self.res_y - height, width, height)?;
        panel.draw_fresh()?;
        self.taskbar = Some(panel);
        self.update_taskbar()?;
        Ok(())
    }

    pub fn mouse_pointer_event(&mut self, mouse_event: MouseEvent) -> Result<()> {
        let layer_id = &self
            .mouse_pointer
            .as_ref()
            .ok_or(SimpleWindowManagerError::MousePointerLayerWasNotFound)?
            .layer_id;

        let LayerPositionInfo {
            x: m_x_before,
            y: m_y_before,
            width: m_w,
            height: m_h,
        } = multi_layer::get_layer_pos_info(layer_id)?;

        let rel_x = (mouse_event.rel_x as isize)
            .clamp(-MOUSE_POINTER_MOVE_THRESHOLD, MOUSE_POINTER_MOVE_THRESHOLD);
        let rel_y = (mouse_event.rel_y as isize)
            .clamp(-MOUSE_POINTER_MOVE_THRESHOLD, MOUSE_POINTER_MOVE_THRESHOLD);

        let m_x_after =
            (m_x_before as isize + rel_x).clamp(0, self.res_x as isize - m_w as isize) as usize;
        let m_y_after =
            (m_y_before as isize + rel_y).clamp(0, self.res_y as isize - m_h as isize) as usize;

        // move mouse pointer
        multi_layer::move_layer(layer_id, m_x_after, m_y_after)?;

        if mouse_event.left {
            for w in self.windows.iter_mut().rev() {
                let LayerPositionInfo {
                    x: w_x,
                    y: w_y,
                    width: w_w,
                    height: w_h,
                } = multi_layer::get_layer_pos_info(&w.layer_id)?;

                // drag window event
                if m_x_before >= w_x
                    && m_x_before < w_x + w_w
                    && m_y_before >= w_y
                    && m_y_before < w_y + w_h
                // pointer is in window
                && m_x_before != m_x_after && m_y_before != m_y_after
                // pointer moved
                {
                    let new_w_x =
                        (w_x as isize + m_x_after as isize - m_x_before as isize) as usize;
                    let new_w_y =
                        (w_y as isize + m_y_after as isize - m_y_before as isize) as usize;

                    multi_layer::move_layer(&w.layer_id, new_w_x, new_w_y)?;
                    break;
                }

                // click close button event
                let (cb_x, cb_y) = w.close_button_pos;
                let (cb_w, cb_h) = w.close_button_size;
                if m_x_before >= w_x + cb_x
                    && m_x_before < w_x + cb_x + cb_w
                    && m_y_before >= w_y + cb_y
                    && m_y_before < w_y + cb_y + cb_h
                {
                    w.is_closed = true;
                    self.windows.retain(|w| !w.is_closed);
                    self.update_taskbar()?;
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
        let window = Window::create_and_push(title, x, y, width, height)?;
        window.draw_fresh()?;
        self.windows.push(window);
        let _ = self.update_taskbar();

        Ok(())
    }

    fn update_taskbar(&mut self) -> Result<()> {
        let taskbar = self
            .taskbar
            .as_ref()
            .ok_or(SimpleWindowManagerError::TaskbarLayerWasNotFound)?;
        taskbar.draw_fresh()?;
        let s = format!(
            "{:?}",
            self.windows
                .iter()
                .map(|w| w.title.as_str())
                .collect::<Vec<&str>>()
        );
        multi_layer::draw_layer(&taskbar.layer_id, |l| {
            l.draw_string(0, 0, &s, GLOBAL_THEME.wm_taskbar_color)?;
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

pub fn mouse_pointer_event(mouse_event: MouseEvent) -> Result<()> {
    unsafe { SIMPLE_WM.try_lock() }?
        .as_mut()
        .ok_or(SimpleWindowManagerError::NotInitialized)?
        .mouse_pointer_event(mouse_event)
}

pub fn create_window(title: String, x: usize, y: usize, width: usize, height: usize) -> Result<()> {
    unsafe { SIMPLE_WM.try_lock() }?
        .as_mut()
        .ok_or(SimpleWindowManagerError::NotInitialized)?
        .create_window(title, x, y, width, height)
}
