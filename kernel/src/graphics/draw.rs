use super::color::ColorCode;
use crate::error::Result;

pub trait Draw {
    fn draw_rect(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        color_code: ColorCode,
    ) -> Result<()>;
    fn fill(&self, color_code: ColorCode) -> Result<()>;
    fn copy(&self, x: usize, y: usize, to_x: usize, to_y: usize) -> Result<()>;
    fn read(&self, x: usize, y: usize) -> Result<ColorCode>;
    fn write(&self, x: usize, y: usize, color_code: ColorCode) -> Result<()>;
}
