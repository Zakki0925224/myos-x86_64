use crate::{error::Result, graphics::color::RgbColorCode, println};
use alloc::vec::Vec;
use core::mem::size_of;

const MAGIC: [u8; 3] = *b"GIF";
const IMAGE_BLOCK_MAGIC: u8 = 0x2c;
const EXT_BLOCK_MAGIC: u8 = 0x21;
const GRAPHIC_CTRL_EXT_MAGIC: u8 = 0xf9;
const COMMENT_EXT_MAGIC: u8 = 0xfe;
const PLAIN_TEXT_EXT_MAGIC: u8 = 0x01;
const APPLICATION_EXT_MAGIC: u8 = 0xff;

#[derive(Debug)]
#[repr(C, packed)]
pub struct Header {
    magic: [u8; 3],
    version: [u8; 3],
    width: u16,
    height: u16,
    flags: u8, // GCTF(1), CR(3), SF(1), SGCT(3)
    bg_color_index: u8,
    pixel_aspect_ratio: u8,
}

impl Header {
    pub fn global_color_table_flag(&self) -> bool {
        (self.flags & 0x80) != 0
    }

    pub fn color_res(&self) -> u8 {
        (self.flags & 0x70) >> 4
    }

    pub fn sort_flag(&self) -> bool {
        (self.flags & 0x08) != 0
    }

    pub fn global_color_table_size(&self) -> usize {
        let n = (self.flags & 0x07) + 1;
        // 2^n
        1 << n
    }
}

pub struct ImageBlock {}

#[derive(Debug)]
pub enum Block {
    Image,
    GraphicControlExtension,
    CommentExtension,
    PlainTextExtension,
    ApplicationExtension,
}

pub struct GifImage<'a> {
    data: &'a [u8],
}

impl<'a> GifImage<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn header(&self) -> &Header {
        unsafe { &*(self.data.as_ptr() as *const Header) }
    }

    pub fn is_valid(&self) -> bool {
        self.header().magic == MAGIC
    }

    pub fn global_color_table(&self) -> Vec<RgbColorCode> {
        let header = self.header();
        let offset = size_of::<Header>();
        let size = header.global_color_table_size();
        let rgb_size = size_of::<RgbColorCode>();

        let mut buf = Vec::with_capacity(rgb_size * size);
        for i in 0..size {
            let r = self.data[offset + i * rgb_size];
            let g = self.data[offset + i * rgb_size + 1];
            let b = self.data[offset + i * rgb_size + 2];
            buf.push(RgbColorCode::new(r, g, b));
        }

        buf
    }

    pub fn blocks(&self) -> Result<Vec<Block>> {
        let header = self.header();
        let mut offset =
            size_of::<Header>() + size_of::<RgbColorCode>() * header.global_color_table_size();

        let mut blocks = Vec::new();

        while offset < self.data.len() {
            match [self.data[offset], self.data[offset + 1]] {
                [IMAGE_BLOCK_MAGIC, _] => {
                    blocks.push(Block::Image);
                }
                [EXT_BLOCK_MAGIC, GRAPHIC_CTRL_EXT_MAGIC] => {
                    blocks.push(Block::GraphicControlExtension);
                }
                [EXT_BLOCK_MAGIC, COMMENT_EXT_MAGIC] => {
                    blocks.push(Block::CommentExtension);
                }
                [EXT_BLOCK_MAGIC, PLAIN_TEXT_EXT_MAGIC] => {
                    blocks.push(Block::PlainTextExtension);
                }
                [EXT_BLOCK_MAGIC, APPLICATION_EXT_MAGIC] => {
                    blocks.push(Block::ApplicationExtension);
                }
                _ => break,
            }
            // TODO
            break;

            while self.data[offset] != 0x0 {
                offset += 1;
            }

            if offset != self.data.len() - 1 {
                offset += 1;
            }
        }

        Ok(blocks)
    }
}
