use super::color::RgbColorCode;
use crate::error::Result;

pub trait Draw {
    fn draw_rect(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        color_code: RgbColorCode,
    ) -> Result<()>;
    fn fill(&mut self, color_code: RgbColorCode) -> Result<()>;
    fn copy(&mut self, x: usize, y: usize, to_x: usize, to_y: usize) -> Result<()>;
    fn read(&self, x: usize, y: usize) -> Result<RgbColorCode>;
    fn write(&mut self, x: usize, y: usize, color_code: RgbColorCode) -> Result<()>;
}
