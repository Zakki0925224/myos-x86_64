use super::color::ColorCode;
use crate::error::Result;

pub trait Draw {
    fn draw_rect(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        color_code: ColorCode,
    ) -> Result<()>;
    fn draw_string(
        &mut self,
        x: usize,
        y: usize,
        s: &str,
        fore_color_code: ColorCode,
        back_color_code: ColorCode,
    ) -> Result<()>;
    fn draw_font(
        &mut self,
        x: usize,
        y: usize,
        c: char,
        fore_color_code: ColorCode,
        back_color_code: ColorCode,
    ) -> Result<()>;
    fn fill(&mut self, color_code: ColorCode) -> Result<()>;
    fn copy(&mut self, x: usize, y: usize, to_x: usize, to_y: usize) -> Result<()>;
    fn read(&self, x: usize, y: usize) -> Result<ColorCode>;
    fn write(&mut self, x: usize, y: usize, color_code: ColorCode) -> Result<()>;
}
