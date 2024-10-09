use crate::RgbColorCode;

pub const GLOBAL_THEME: Theme = CLASSIC_THEME;

#[allow(unused)]
const COLOR_WHITE: RgbColorCode = RgbColorCode::new(255, 255, 255);
#[allow(unused)]
const COLOR_OLIVE: RgbColorCode = RgbColorCode::new(128, 128, 0);
#[allow(unused)]
const COLOR_YELLOW: RgbColorCode = RgbColorCode::new(255, 255, 0);
#[allow(unused)]
const COLOR_FUCHSIA: RgbColorCode = RgbColorCode::new(255, 0, 255);
#[allow(unused)]
const COLOR_SILVER: RgbColorCode = RgbColorCode::new(192, 192, 192);
#[allow(unused)]
const COLOR_CYAN: RgbColorCode = RgbColorCode::new(0, 255, 255);
#[allow(unused)]
const COLOR_GREEN: RgbColorCode = RgbColorCode::new(0, 255, 0);
#[allow(unused)]
const COLOR_RED: RgbColorCode = RgbColorCode::new(255, 0, 0);
#[allow(unused)]
const COLOR_GRAY: RgbColorCode = RgbColorCode::new(128, 128, 128);
#[allow(unused)]
const COLOR_BLUE: RgbColorCode = RgbColorCode::new(0, 0, 255);
#[allow(unused)]
const COLOR_PURPLE: RgbColorCode = RgbColorCode::new(128, 0, 128);
#[allow(unused)]
const COLOR_BLACK: RgbColorCode = RgbColorCode::new(0, 0, 0);
#[allow(unused)]
const COLOR_NAVY: RgbColorCode = RgbColorCode::new(0, 0, 128);
#[allow(unused)]
const COLOR_TEAL: RgbColorCode = RgbColorCode::new(0, 128, 128);
#[allow(unused)]
const COLOR_MAROON: RgbColorCode = RgbColorCode::new(128, 0, 0);

#[allow(unused)]
const DEFAULT_COLOR_1: RgbColorCode = RgbColorCode::new(0x03, 0x1a, 0x00);
const DEFAULT_COLOR_2: RgbColorCode = RgbColorCode::new(0x12, 0xca, 0x63);

#[allow(unused)]
const CLASSIC_COLOR_1: RgbColorCode = RgbColorCode::new(0x3a, 0x6e, 0xa5);
const CLASSIC_COLOR_2: RgbColorCode = RgbColorCode::new(0xd4, 0xd0, 0xc8);

#[allow(unused)]
const DEFAULT_THEME: Theme = Theme {
    transparent_color: COLOR_BLACK,
    back_color: DEFAULT_COLOR_1,
    fore_color: DEFAULT_COLOR_2,
    sample_rect_colors: [
        COLOR_WHITE,
        COLOR_OLIVE,
        COLOR_YELLOW,
        COLOR_FUCHSIA,
        COLOR_SILVER,
        COLOR_CYAN,
        COLOR_GREEN,
        COLOR_RED,
        COLOR_GRAY,
        COLOR_BLUE,
        COLOR_PURPLE,
        COLOR_BLACK,
        COLOR_NAVY,
        COLOR_TEAL,
        COLOR_MAROON,
        DEFAULT_COLOR_1,
    ],
    log_color_error: COLOR_RED,
    log_color_warn: RgbColorCode::new(253, 126, 0), // orange
    log_color_info: COLOR_CYAN,
    log_color_debug: COLOR_YELLOW,
    log_color_trace: COLOR_GREEN,

    // TODO
    wm_panel_back_color: CLASSIC_COLOR_2,
    wm_panel_fore_color: COLOR_BLACK,
    wm_panel_border_color1: COLOR_WHITE,
    wm_panel_border_color2: RgbColorCode::new(0x79, 0x75, 0x71),
    wm_window_titlebar_back_color: RgbColorCode::new(0x0a, 0x24, 0x6a),
    wm_window_titlebar_fore_color: COLOR_WHITE,
    wm_window_close_button_back_color: COLOR_RED,
    io_buf_default_back_color: COLOR_BLACK,
    io_buf_default_fore_color: COLOR_BLACK,
};

#[allow(unused)]
const CLASSIC_THEME: Theme = Theme {
    transparent_color: COLOR_BLACK,
    back_color: CLASSIC_COLOR_1,
    fore_color: COLOR_WHITE,
    sample_rect_colors: [
        CLASSIC_COLOR_1,
        CLASSIC_COLOR_1,
        CLASSIC_COLOR_1,
        CLASSIC_COLOR_1,
        CLASSIC_COLOR_1,
        CLASSIC_COLOR_1,
        CLASSIC_COLOR_1,
        CLASSIC_COLOR_1,
        CLASSIC_COLOR_1,
        CLASSIC_COLOR_1,
        CLASSIC_COLOR_1,
        CLASSIC_COLOR_1,
        CLASSIC_COLOR_1,
        CLASSIC_COLOR_1,
        CLASSIC_COLOR_1,
        CLASSIC_COLOR_1,
    ],
    log_color_error: COLOR_RED,
    log_color_warn: RgbColorCode::new(0xe7, 0xe7, 0x00),
    log_color_info: RgbColorCode::new(0x00, 0xc0, 0x00),
    log_color_debug: COLOR_WHITE,
    log_color_trace: COLOR_WHITE,
    wm_panel_back_color: CLASSIC_COLOR_2,
    wm_panel_fore_color: COLOR_BLACK,
    wm_panel_border_color1: COLOR_WHITE,
    wm_panel_border_color2: RgbColorCode::new(0x79, 0x75, 0x71),
    wm_window_titlebar_back_color: RgbColorCode::new(0x0a, 0x24, 0x6a),
    wm_window_titlebar_fore_color: COLOR_WHITE,
    wm_window_close_button_back_color: COLOR_RED,
    io_buf_default_back_color: COLOR_BLACK,
    io_buf_default_fore_color: COLOR_BLACK,
};

#[allow(unused)]
pub struct Theme {
    // framebuffer
    pub transparent_color: RgbColorCode,
    pub back_color: RgbColorCode,
    pub fore_color: RgbColorCode,
    pub sample_rect_colors: [RgbColorCode; 16],
    // log
    pub log_color_error: RgbColorCode,
    pub log_color_warn: RgbColorCode,
    pub log_color_info: RgbColorCode,
    pub log_color_debug: RgbColorCode,
    pub log_color_trace: RgbColorCode,
    // simple wm
    pub wm_panel_back_color: RgbColorCode,
    pub wm_panel_fore_color: RgbColorCode,
    pub wm_panel_border_color1: RgbColorCode, // left, top
    pub wm_panel_border_color2: RgbColorCode, // right, bottom
    pub wm_window_titlebar_back_color: RgbColorCode,
    pub wm_window_titlebar_fore_color: RgbColorCode,
    pub wm_window_close_button_back_color: RgbColorCode,
    // io buffer
    pub io_buf_default_back_color: RgbColorCode,
    pub io_buf_default_fore_color: RgbColorCode,
}
