use alloc::vec::Vec;

use crate::graphics::color::RgbColorCode;
use core::mem::size_of;

const MAGIC: [u8; 2] = *b"BM";

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

    pub fn is_valid(&self) -> bool {
        self.header().type_ == MAGIC
    }

    pub fn bitmap(&self) -> &[u8] {
        let offset = self.header().offset as usize;
        &self.data[offset..]
    }

    pub fn bitmap_to_rgb_color_code(&self) -> Vec<RgbColorCode> {
        let bitmap = self.bitmap();
        let info_header = self.info_header();
        let width = info_header.width.abs() as usize;
        let height = info_header.height.abs() as usize;
        let bits_per_pixel = info_header.bits_per_pixel as usize / 8;
        let padding = (4 - (width * bits_per_pixel) % 4) % 4;
        let mut data = Vec::new();

        for y in 0..height {
            for x in 0..width {
                let offset = (height - y - 1) as usize
                    * (width * bits_per_pixel + padding) as usize
                    + x as usize * bits_per_pixel as usize;
                let b = bitmap[offset];
                let g = bitmap[offset + 1];
                let r = bitmap[offset + 2];
                data.push(RgbColorCode { r, g, b });
            }
        }

        data
    }
}
