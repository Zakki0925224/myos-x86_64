//PSF font
const FONT_MAGIC_NUM: u32 = 0x864ab572;

struct PsfHeader
{
    pub magic: u32,
    pub version: u32,
    pub header_size: u32,
    pub flags: u32,
    pub glyph_len: u32,
    pub glyph_size: u32,
    pub glyph_height: u32,
    pub glyph_width: u32,
}
