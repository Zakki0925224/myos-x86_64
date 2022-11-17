//PSF font
static FONT: &[u8] = include_bytes!("../../../third-party/cozette.psf");
const FONT_MAGIC_NUM: u32 = 0x864ab572;
const UNICODE_TABLE_SEPARATOR: u8 = 0xff; // PSF v2
const MAX_HEIGHT: usize = 16;
const MAX_WIDTH: usize = 8;

fn get_magic_num() -> u32
{
    return (FONT[3] as u32) << 24
        | (FONT[2] as u32) << 16
        | (FONT[1] as u32) << 8
        | FONT[0] as u32;
}

fn get_pixel_height() -> u32
{
    return (FONT[27] as u32) << 24
        | (FONT[26] as u32) << 16
        | (FONT[25] as u32) << 8
        | FONT[24] as u32;
}

fn get_pixel_width() -> u32
{
    return (FONT[31] as u32) << 24
        | (FONT[30] as u32) << 16
        | (FONT[29] as u32) << 8
        | FONT[28] as u32;
}

fn get_glyphs_len() -> u32
{
    return (FONT[19] as u32) << 24
        | (FONT[18] as u32) << 16
        | (FONT[17] as u32) << 8
        | FONT[16] as u32;
}

fn get_glyph_size() -> u32
{
    return (FONT[23] as u32) << 24
        | (FONT[22] as u32) << 16
        | (FONT[21] as u32) << 8
        | FONT[20] as u32;
}

fn has_unicode_table() -> bool
{
    let flags = (FONT[15] as u32) << 24
        | (FONT[14] as u32) << 16
        | (FONT[13] as u32) << 8
        | FONT[12] as u32;

    return flags == 1;
}

fn get_header_size() -> u32
{
    return (FONT[11] as u32) << 24
        | (FONT[10] as u32) << 16
        | (FONT[9] as u32) << 8
        | FONT[8] as u32;
}

pub struct PsfFont
{
    pub max_height: usize,
    pub max_width: usize,
    pub height: usize,
    pub width: usize,
    pub glyphs_len: usize,
    pub glyph_size: usize,
    pub has_unicode_table: bool,
    header_size: usize,
    unicode_table_offset: usize,
}

impl PsfFont
{
    pub fn new() -> Self
    {
        if self::get_magic_num() != FONT_MAGIC_NUM
        {
            panic!("Invalid font binary");
        }

        let max_height = MAX_HEIGHT;
        let max_width = MAX_WIDTH;
        let height = self::get_pixel_height() as usize;
        let width = self::get_pixel_width() as usize;
        let glyphs_len = self::get_glyphs_len() as usize;
        let glyph_size = self::get_glyph_size() as usize;
        let has_unicode_table = self::has_unicode_table();
        let header_size = self::get_header_size() as usize;
        let unicode_table_offset = header_size + glyph_size * glyphs_len;

        if height > max_height || width > max_width
        {
            panic!("Invalid font size");
        }

        return Self {
            max_height,
            max_width,
            height,
            width,
            glyphs_len,
            glyph_size,
            has_unicode_table,
            header_size,
            unicode_table_offset,
        };
    }

    fn unicode_char_to_glyph_index(&self, c: char) -> usize
    {
        if !self.has_unicode_table
        {
            return c as u32 as usize;
        }

        let bytes: [u8; 4] = (c as u32).to_be_bytes();
        let mut index = self.unicode_table_offset;
        let mut font_bytes_offset = index;
        let mut i = 0;

        while font_bytes_offset + bytes.len() < FONT.len()
        {
            while i < bytes.len()
            {
                if bytes[i] == 0
                {
                    i += 1;
                    continue;
                }

                if bytes[i] != FONT[font_bytes_offset]
                {
                    break;
                }

                i += 1;
                font_bytes_offset += 1;
            }

            if FONT[font_bytes_offset] == UNICODE_TABLE_SEPARATOR
            {
                index += 1;
            }

            i = 0;
            font_bytes_offset += 1;
        }

        return index;
    }

    pub fn get_glyph(&self, index: usize) -> Option<&[u8]>
    {
        //let index = self.unicode_char_to_glyph_index(c);

        if index > self.glyphs_len
        {
            return None;
        }

        let offset = self.header_size + self.glyph_size * index;

        return Some(&FONT[offset..offset + self.glyph_size]);
    }
}
