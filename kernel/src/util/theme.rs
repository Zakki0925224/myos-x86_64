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
const D_COLOR_1: RgbColorCode = RgbColorCode::new(0x03, 0x1a, 0x00);

const D_COLOR_2: RgbColorCode = RgbColorCode::new(0x12, 0xca, 0x63);

// nord colors: https://www.nordtheme.com/
#[allow(unused)]
const PN_COLOR_1: RgbColorCode = RgbColorCode::new(0x2e, 0x34, 0x40);
#[allow(unused)]
const PN_COLOR_2: RgbColorCode = RgbColorCode::new(0x3b, 0x42, 0x52);
#[allow(unused)]
const PN_COLOR_3: RgbColorCode = RgbColorCode::new(0x43, 0x4c, 0x5e);
#[allow(unused)]
const PN_COLOR_4: RgbColorCode = RgbColorCode::new(0x4c, 0x56, 0x6a);
#[allow(unused)]
const SS_COLOR_1: RgbColorCode = RgbColorCode::new(0xd8, 0xde, 0xe9);
#[allow(unused)]
const SS_COLOR_2: RgbColorCode = RgbColorCode::new(0xe5, 0xe9, 0xf0);
#[allow(unused)]
const SS_COLOR_3: RgbColorCode = RgbColorCode::new(0xec, 0xef, 0xf4);
#[allow(unused)]
const FR_COLOR_1: RgbColorCode = RgbColorCode::new(0x8f, 0xbc, 0xbb);
#[allow(unused)]
const FR_COLOR_2: RgbColorCode = RgbColorCode::new(0x88, 0xc0, 0xd0);
#[allow(unused)]
const FR_COLOR_3: RgbColorCode = RgbColorCode::new(0x81, 0xa1, 0xc1);
#[allow(unused)]
const FR_COLOR_4: RgbColorCode = RgbColorCode::new(0x5e, 0x81, 0xac);
#[allow(unused)]
const AU_COLOR_1: RgbColorCode = RgbColorCode::new(0xbf, 0x61, 0x6a); // red
#[allow(unused)]
const AU_COLOR_2: RgbColorCode = RgbColorCode::new(0xd0, 0x87, 0x70); // orange
#[allow(unused)]
const AU_COLOR_3: RgbColorCode = RgbColorCode::new(0xeb, 0xcb, 0x8b); // yellow
#[allow(unused)]
const AU_COLOR_4: RgbColorCode = RgbColorCode::new(0xa3, 0xbe, 0x8c); // green
#[allow(unused)]
const AU_COLOR_5: RgbColorCode = RgbColorCode::new(0xb4, 0x8e, 0xad); // purple

#[allow(unused)]
const C_COLOR_1: RgbColorCode = RgbColorCode::new(0x3a, 0x6e, 0xa5);
const C_COLOR_2: RgbColorCode = RgbColorCode::new(0xd4, 0xd0, 0xc8);

#[allow(unused)]
const DEFAULT_THEME: Theme = Theme {
    transparent_color: COLOR_BLACK,
    back_color: D_COLOR_1,
    fore_color: D_COLOR_2,
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
        D_COLOR_1,
    ],
    log_color_error: COLOR_RED,
    log_color_warn: RgbColorCode::new(253, 126, 0), // orange
    log_color_info: COLOR_CYAN,
    log_color_debug: COLOR_YELLOW,
    log_color_trace: COLOR_GREEN,
    wm_taskbar_color: SS_COLOR_1,
    wm_panel_back_color: SS_COLOR_2,
    wm_panel_border_color: SS_COLOR_3,
    wm_window_back_color: PN_COLOR_1,
    wm_window_border_color: PN_COLOR_4,
    wm_window_titlebar_back_color: PN_COLOR_2,
    wm_window_titlebar_fore_color: SS_COLOR_1,
    wm_window_close_button_back_color: AU_COLOR_1,
    io_buf_default_back_color: COLOR_BLACK,
    io_buf_default_fore_color: COLOR_BLACK,
};

#[allow(unused)]
const NORD_THEME: Theme = Theme {
    transparent_color: RgbColorCode::new(0, 0, 0),
    back_color: PN_COLOR_1,
    fore_color: SS_COLOR_1,
    sample_rect_colors: [
        PN_COLOR_1, PN_COLOR_2, PN_COLOR_3, PN_COLOR_4, SS_COLOR_1, SS_COLOR_2, SS_COLOR_3,
        FR_COLOR_1, FR_COLOR_2, FR_COLOR_3, FR_COLOR_4, AU_COLOR_1, AU_COLOR_2, AU_COLOR_3,
        AU_COLOR_4, AU_COLOR_5,
    ],
    log_color_error: AU_COLOR_1,
    log_color_warn: AU_COLOR_2,
    log_color_info: AU_COLOR_4,
    log_color_debug: AU_COLOR_3,
    log_color_trace: AU_COLOR_2,
    wm_taskbar_color: SS_COLOR_1,
    wm_panel_back_color: SS_COLOR_2,
    wm_panel_border_color: SS_COLOR_3,
    wm_window_back_color: PN_COLOR_1,
    wm_window_border_color: PN_COLOR_4,
    wm_window_titlebar_back_color: PN_COLOR_2,
    wm_window_titlebar_fore_color: SS_COLOR_1,
    wm_window_close_button_back_color: AU_COLOR_1,
    io_buf_default_back_color: COLOR_BLACK,
    io_buf_default_fore_color: COLOR_BLACK,
};

#[allow(unused)]
const CLASSIC_THEME: Theme = Theme {
    transparent_color: COLOR_BLACK,
    back_color: C_COLOR_1,
    fore_color: COLOR_WHITE,
    sample_rect_colors: [
        C_COLOR_1, C_COLOR_1, C_COLOR_1, C_COLOR_1, C_COLOR_1, C_COLOR_1, C_COLOR_1, C_COLOR_1,
        C_COLOR_1, C_COLOR_1, C_COLOR_1, C_COLOR_1, C_COLOR_1, C_COLOR_1, C_COLOR_1, C_COLOR_1,
    ],
    log_color_error: COLOR_WHITE,
    log_color_warn: COLOR_WHITE,
    log_color_info: COLOR_WHITE,
    log_color_debug: COLOR_WHITE,
    log_color_trace: COLOR_WHITE,
    wm_taskbar_color: C_COLOR_2,
    wm_panel_back_color: C_COLOR_2,
    wm_panel_border_color: COLOR_WHITE,
    wm_window_back_color: C_COLOR_2,
    wm_window_border_color: COLOR_WHITE,
    wm_window_titlebar_back_color: RgbColorCode::new(0x0d, 0x0d, 0xa1),
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
    pub wm_taskbar_color: RgbColorCode,
    pub wm_panel_back_color: RgbColorCode,
    pub wm_panel_border_color: RgbColorCode,
    pub wm_window_back_color: RgbColorCode,
    pub wm_window_border_color: RgbColorCode,
    pub wm_window_titlebar_back_color: RgbColorCode,
    pub wm_window_titlebar_fore_color: RgbColorCode,
    pub wm_window_close_button_back_color: RgbColorCode,
    // io buffer
    pub io_buf_default_back_color: RgbColorCode,
    pub io_buf_default_fore_color: RgbColorCode,
}
