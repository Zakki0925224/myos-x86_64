//PSF font v2
const FONT: &'static [u8] = include_bytes!("../../../third-party/cozette.psf");
const FONT_MAGIC_NUM: u32 = 0x864ab572;
const UNICODE_TABLE_SEPARATOR: u8 = 0xff;

pub struct PsfFont {
    binary_len: usize,
    height: usize,
    width: usize,
    glyphs_len: usize,
    glyph_size: usize,
    has_unicode_table: bool,
    header_size: usize,
    unicode_table_offset: usize,
}

impl PsfFont {
    pub fn new() -> Self {
        fn get_magic_num() -> u32 {
            (FONT[3] as u32) << 24 | (FONT[2] as u32) << 16 | (FONT[1] as u32) << 8 | FONT[0] as u32
        }

        fn get_pixel_height() -> u32 {
            (FONT[27] as u32) << 24
                | (FONT[26] as u32) << 16
                | (FONT[25] as u32) << 8
                | FONT[24] as u32
        }

        fn get_pixel_width() -> u32 {
            (FONT[31] as u32) << 24
                | (FONT[30] as u32) << 16
                | (FONT[29] as u32) << 8
                | FONT[28] as u32
        }

        fn get_glyphs_len() -> u32 {
            (FONT[19] as u32) << 24
                | (FONT[18] as u32) << 16
                | (FONT[17] as u32) << 8
                | FONT[16] as u32
        }

        fn get_glyph_size() -> u32 {
            (FONT[23] as u32) << 24
                | (FONT[22] as u32) << 16
                | (FONT[21] as u32) << 8
                | FONT[20] as u32
        }

        fn has_unicode_table() -> bool {
            let flags = (FONT[15] as u32) << 24
                | (FONT[14] as u32) << 16
                | (FONT[13] as u32) << 8
                | FONT[12] as u32;

            flags == 1
        }

        fn get_header_size() -> u32 {
            (FONT[11] as u32) << 24
                | (FONT[10] as u32) << 16
                | (FONT[9] as u32) << 8
                | FONT[8] as u32
        }

        if get_magic_num() != FONT_MAGIC_NUM {
            panic!("Invalid font binary");
        }

        let binary_len = FONT.len();
        let height = get_pixel_height() as usize;
        let width = get_pixel_width() as usize;
        let glyphs_len = get_glyphs_len() as usize;
        let glyph_size = get_glyph_size() as usize;
        let has_unicode_table = has_unicode_table();
        let header_size = get_header_size() as usize;
        let unicode_table_offset = header_size + glyph_size * glyphs_len;

        Self {
            binary_len,
            height,
            width,
            glyphs_len,
            glyph_size,
            has_unicode_table,
            header_size,
            unicode_table_offset,
        }
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    // ascii char only
    pub fn unicode_char_to_glyph_index(&self, c: char) -> usize {
        if !self.has_unicode_table {
            return c as usize;
        }

        let code_point = c as u8;
        let mut index = 0;

        for i in self.unicode_table_offset..self.binary_len {
            if code_point == FONT[i] {
                break;
            }

            if FONT[i] == UNICODE_TABLE_SEPARATOR {
                index += 1;
            }
        }

        index
    }

    pub fn get_glyph(&self, index: usize) -> Option<&'static [u8]> {
        if index > self.glyphs_len {
            return None;
        }

        let offset = self.header_size + self.glyph_size * index;
        Some(&FONT[offset..offset + self.glyph_size])
    }
}
