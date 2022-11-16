//PSF font
static FONT: &[u8] = include_bytes!("../../../third-party/cozette.psf");
const FONT_MAGIC_NUM: u32 = 0x864ab572;

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

pub struct PsfFont
{
    pub height: usize,
    pub width: usize,
    pub glyphs_len: usize,
    pub glyph_size: usize,
    pub has_unicode_table: bool,
}

impl PsfFont
{
    pub fn new() -> Self
    {
        if self::get_magic_num() != FONT_MAGIC_NUM
        {
            panic!("Invalid font binary");
        }

        let height = self::get_pixel_height() as usize;
        let width = self::get_pixel_width() as usize;
        let glyphs_len = self::get_glyphs_len() as usize;
        let glyph_size = self::get_glyph_size() as usize;
        let has_unicode_table = self::has_unicode_table();

        return Self { height, width, glyphs_len, glyph_size, has_unicode_table };
    }
}
