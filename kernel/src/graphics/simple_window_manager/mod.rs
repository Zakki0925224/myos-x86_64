use super::{
    frame_buf,
    multi_layer::{LayerId, LayerPositionInfo},
};
use crate::{
    device::ps2_mouse::MouseEvent, error::Result, fs::file::bitmap::BitmapImage,
    theme::GLOBAL_THEME, util::mutex::Mutex,
};
use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
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
    fn new(res_x: usize, res_y: usize) -> Self {
        Self {
            windows: Vec::new(),
            taskbar: None,
            mouse_pointer: None,
            res_x,
            res_y,
        }
    }

    fn create_mouse_pointer(&mut self, pointer_bmp: &BitmapImage) -> Result<()> {
        self.mouse_pointer = Some(Image::create_and_push(pointer_bmp, 0, 0, true)?);

        Ok(())
    }

    fn create_taskbar(&mut self) -> Result<()> {
        let width = self.res_x;
        let height = 30;
        let mut panel = Panel::create_and_push(0, self.res_y - height, width, height)?;
        panel.draw_fresh()?;
        self.taskbar = Some(panel);
        self.update_taskbar()?;
        Ok(())
    }

    fn mouse_pointer_event(&mut self, mouse_event: MouseEvent) -> Result<()> {
        let mouse_pointer = self
            .mouse_pointer
            .as_mut()
            .ok_or(SimpleWindowManagerError::MousePointerLayerWasNotFound)?;

        let LayerPositionInfo {
            x: m_x_before,
            y: m_y_before,
            width: m_w,
            height: m_h,
        } = mouse_pointer.get_layer_pos_info()?;

        let rel_x = (mouse_event.rel_x as isize)
            .clamp(-MOUSE_POINTER_MOVE_THRESHOLD, MOUSE_POINTER_MOVE_THRESHOLD);
        let rel_y = (mouse_event.rel_y as isize)
            .clamp(-MOUSE_POINTER_MOVE_THRESHOLD, MOUSE_POINTER_MOVE_THRESHOLD);

        let m_x_after =
            (m_x_before as isize + rel_x).clamp(0, self.res_x as isize - m_w as isize) as usize;
        let m_y_after =
            (m_y_before as isize + rel_y).clamp(0, self.res_y as isize - m_h as isize) as usize;

        // move mouse pointer
        mouse_pointer.move_by_root(m_x_after, m_y_after)?;

        if mouse_event.left {
            for w in self.windows.iter_mut().rev() {
                let LayerPositionInfo {
                    x: w_x,
                    y: w_y,
                    width: w_w,
                    height: w_h,
                } = w.get_layer_pos_info()?;

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
                        (w_x as isize + m_x_after as isize - m_x_before as isize).max(0) as usize;
                    let new_w_y =
                        (w_y as isize + m_y_after as isize - m_y_before as isize).max(0) as usize;

                    w.move_by_root(new_w_x, new_w_y)?;
                    break;
                }

                // click close button event
                if w.is_close_button_clickable(m_x_before, m_y_before)? {
                    w.is_closed = true;
                    self.windows.retain(|w| !w.is_closed);
                    self.update_taskbar()?;
                    break;
                }
            }
        }

        Ok(())
    }

    fn create_window(
        &mut self,
        title: String,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<LayerId> {
        let mut window = Window::create_and_push(title, x, y, width, height)?;

        let button1 = Button::create_and_push_without_pos("button 1".to_string(), 100, 25)?;
        let button2 = Button::create_and_push_without_pos("button 2".to_string(), 100, 25)?;
        let button3 = Button::create_and_push_without_pos("button 3".to_string(), 100, 25)?;
        let button4 = Button::create_and_push_without_pos("button 4".to_string(), 100, 25)?;
        let button5 = Button::create_and_push_without_pos("button 5".to_string(), 100, 25)?;
        let button6 = Button::create_and_push_without_pos("button 6".to_string(), 100, 25)?;
        let button7 = Button::create_and_push_without_pos("button 7".to_string(), 100, 25)?;
        let label = Label::create_and_push_without_pos(
            "[32] Sed ut perspiciatis, unde omnis iste natus error sit voluptatem\naccusantium doloremque laudantium, totam rem aperiam eaque ipsa, quae\nab illo inventore veritatis et quasi architecto beatae vitae dicta sunt,\nexplicabo.\nNemo enim ipsam voluptatem, quia voluptas sit, aspernatur aut\nodit aut fugit, sed quia consequuntur magni dolores eos, qui ratione\nvoluptatem sequi nesciunt, neque porro quisquam est, qui dolorem ipsum,\nquia dolor sit, amet, consectetur, adipisci velit, sed quia non numquam\neius modi tempora incidunt, ut labore et dolore magnam aliquam quaerat\nvoluptatem.".to_string(),
            GLOBAL_THEME.fore_color,
            GLOBAL_THEME.back_color,
        )?;

        window.push_child(Box::new(button1))?;
        window.push_child(Box::new(button2))?;
        window.push_child(Box::new(button3))?;
        window.push_child(Box::new(button4))?;
        window.push_child(Box::new(button5))?;
        window.push_child(Box::new(button6))?;
        window.push_child(Box::new(button7))?;
        window.push_child(Box::new(label))?;

        window.draw_fresh()?;
        let layer_id = window.layer_id_clone();
        self.windows.push(window);
        let _ = self.update_taskbar();

        Ok(layer_id)
    }

    fn destroy_window(&mut self, layer_id: &LayerId) -> Result<()> {
        self.windows
            .retain(|w| w.layer_id_clone().get() != layer_id.get());

        let _ = self.update_taskbar();
        Ok(())
    }

    fn update_taskbar(&mut self) -> Result<()> {
        let taskbar = self
            .taskbar
            .as_mut()
            .ok_or(SimpleWindowManagerError::TaskbarLayerWasNotFound)?;
        taskbar.draw_fresh()?;
        let s = format!(
            "{:?}",
            self.windows
                .iter()
                .map(|w| w.title())
                .collect::<Vec<&str>>()
        );
        taskbar.draw_string(7, 7, &s)?;

        Ok(())
    }
}

pub fn init() -> Result<()> {
    let (res_x, res_y) = frame_buf::get_resolution()?;
    *unsafe { SIMPLE_WM.get_force_mut() } = Some(SimpleWindowManager::new(res_x, res_y));
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

pub fn destroy_window(layer_id: &LayerId) -> Result<()> {
    unsafe { SIMPLE_WM.try_lock() }?
        .as_mut()
        .ok_or(SimpleWindowManagerError::NotInitialized)?
        .destroy_window(layer_id)
}

pub fn mouse_pointer_event(mouse_event: MouseEvent) -> Result<()> {
    unsafe { SIMPLE_WM.try_lock() }?
        .as_mut()
        .ok_or(SimpleWindowManagerError::NotInitialized)?
        .mouse_pointer_event(mouse_event)
}

pub fn create_window(
    title: String,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> Result<LayerId> {
    unsafe { SIMPLE_WM.try_lock() }?
        .as_mut()
        .ok_or(SimpleWindowManagerError::NotInitialized)?
        .create_window(title, x, y, width, height)
}
