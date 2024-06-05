use crate::{error::Error, graphics::color::RgbColorCode, Result};
use core::{mem::size_of, slice};

#[derive(Debug)]
#[repr(C, packed)]
pub struct BitmapImageHeader {
    pub type_: [u8; 2],
    pub file_size: u32,
    reserved: [u16; 2],
    pub offset: u32,
}

#[repr(C)]
pub struct BitmapImageInfoHeader {
    pub header_size: u32,
    pub width: i32,
    pub height: i32,
    pub planes: u16,
    pub bits_per_pixel: u16,
    pub compression: u32,
    pub image_size: u32,
    pub x_pixels_per_meter: i32,
    pub y_pixels_per_meter: i32,
    pub colors_used: u32,
    pub colors_important: u32,
}

// TODO: supported RGB (24bits) bitmap only
pub struct BitmapImage<'a> {
    data: &'a [u8],
}

impl<'a> BitmapImage<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn header(&self) -> &BitmapImageHeader {
        unsafe { &*(self.data.as_ptr() as *const BitmapImageHeader) }
    }

    pub fn info_header(&self) -> &BitmapImageInfoHeader {
        let offset = size_of::<BitmapImageHeader>();
        unsafe { &*(self.data.as_ptr().add(offset) as *const BitmapImageInfoHeader) }
    }

    pub fn bitmap(&self) -> Result<&[RgbColorCode]> {
        let color_code_size = size_of::<RgbColorCode>();
        let header = self.header();
        let info_header = self.info_header();
        let offset = header.offset as usize;
        let bitmap_data = &self.data[offset..];
        let num_pixels = bitmap_data.len() / color_code_size;

        if info_header.bits_per_pixel as usize != color_code_size * 8
            || info_header.width as usize * info_header.height as usize * color_code_size
                > bitmap_data.len()
        {
            return Err(Error::Failed("Invalid bitmap data"));
        }

        let casted_data = unsafe {
            slice::from_raw_parts(bitmap_data.as_ptr() as *const RgbColorCode, num_pixels)
        };
        Ok(casted_data)
    }
}
