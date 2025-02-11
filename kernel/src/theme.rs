use crate::{graphics::frame_buf_console, println, ColorCode};

pub const GLOBAL_THEME: Theme = LEGACY_THEME;

const BASE_WHITE: ColorCode = ColorCode::new_rgb(0xff, 0xff, 0xff);
const BASE_RED: ColorCode = ColorCode::new_rgb(0xff, 0, 0);
const BASE_BLACK: ColorCode = ColorCode::new_rgb(0, 0, 0);

const LEGACY_BLACK: ColorCode = BASE_BLACK;
const LEGACY_DARK_GREEN: ColorCode = ColorCode::new_rgb(0x00, 0x55, 0x00);
const LEGACY_GREEN: ColorCode = ColorCode::new_rgb(0x00, 0xaa, 0x00);
const LEGACY_BRIGHT_GREEN: ColorCode = ColorCode::new_rgb(0x00, 0xff, 0x00);
const LEGACY_BLUE: ColorCode = ColorCode::new_rgb(0x00, 0x00, 0xff);
const LEGACY_MODERATE_BLUE: ColorCode = ColorCode::new_rgb(0x00, 0x55, 0xaa);
const LEGACY_LIGHT_BLUE: ColorCode = ColorCode::new_rgb(0x00, 0xaa, 0xff);
const LEGACY_CYAN: ColorCode = ColorCode::new_rgb(0x00, 0xff, 0xff);
const LEGACY_RED: ColorCode = BASE_RED;
const LEGACY_ORANGE: ColorCode = ColorCode::new_rgb(0xff, 0x55, 0x00);
const LEGACY_YELLOW_ORANGE: ColorCode = ColorCode::new_rgb(0xff, 0xaa, 0x00);
const LEGACY_YELLOW: ColorCode = ColorCode::new_rgb(0xff, 0xff, 0x00);
const LEGACY_MAGENTA: ColorCode = ColorCode::new_rgb(0xff, 0x00, 0xff);
const LEGACY_BRIGHT_MAGENTA: ColorCode = ColorCode::new_rgb(0xff, 0x55, 0xff);
const LEGACY_SOFT_MAGENTA: ColorCode = ColorCode::new_rgb(0xff, 0xaa, 0xff);
const LEGACY_WHITE: ColorCode = BASE_WHITE;

#[allow(unused)]
const LEGACY_THEME: Theme = Theme {
    back_color: ColorCode::new_rgb(0x03, 0x1a, 0x00),
    fore_color: LEGACY_GREEN,
    sample_rect_colors: [
        LEGACY_BLACK,
        LEGACY_DARK_GREEN,
        LEGACY_GREEN,
        LEGACY_BRIGHT_GREEN,
        LEGACY_BLUE,
        LEGACY_MODERATE_BLUE,
        LEGACY_LIGHT_BLUE,
        LEGACY_CYAN,
        LEGACY_RED,
        LEGACY_ORANGE,
        LEGACY_YELLOW_ORANGE,
        LEGACY_YELLOW,
        LEGACY_MAGENTA,
        LEGACY_BRIGHT_MAGENTA,
        LEGACY_SOFT_MAGENTA,
        LEGACY_WHITE,
    ],
    log_color_error: LEGACY_RED,
    log_color_warn: LEGACY_ORANGE,
    log_color_info: LEGACY_CYAN,
    log_color_debug: LEGACY_YELLOW,
    log_color_trace: LEGACY_MAGENTA,
    wm_component_back_color: LEGACY_BLACK,
    wm_component_fore_color: LEGACY_GREEN,
    wm_component_border_color1: LEGACY_GREEN,
    wm_component_border_color2: LEGACY_GREEN,
    wm_component_border_flat: true,
    wm_window_titlebar_back_color: LEGACY_BLACK,
    wm_window_titlebar_fore_color: LEGACY_GREEN,
    io_buf_default_back_color: LEGACY_BLACK,
    io_buf_default_fore_color: LEGACY_BLACK,
};

#[allow(unused)]
const CLASSIC_BACK: ColorCode = ColorCode::new_rgb(0x3a, 0x6e, 0xa5);
const CLASSIC_FORE: ColorCode = ColorCode::new_rgb(0xd4, 0xd0, 0xc8);

#[allow(unused)]
const CLASSIC_THEME: Theme = Theme {
    back_color: CLASSIC_BACK,
    fore_color: BASE_WHITE,
    sample_rect_colors: [
        CLASSIC_BACK,
        CLASSIC_BACK,
        CLASSIC_BACK,
        CLASSIC_BACK,
        CLASSIC_BACK,
        CLASSIC_BACK,
        CLASSIC_BACK,
        CLASSIC_BACK,
        CLASSIC_BACK,
        CLASSIC_BACK,
        CLASSIC_BACK,
        CLASSIC_BACK,
        CLASSIC_BACK,
        CLASSIC_BACK,
        CLASSIC_BACK,
        CLASSIC_BACK,
    ],
    log_color_error: BASE_RED,
    log_color_warn: ColorCode::new_rgb(0xe7, 0xe7, 0x00),
    log_color_info: ColorCode::new_rgb(0x00, 0xc0, 0x00),
    log_color_debug: BASE_WHITE,
    log_color_trace: BASE_WHITE,
    wm_component_back_color: CLASSIC_FORE,
    wm_component_fore_color: BASE_BLACK,
    wm_component_border_color1: BASE_WHITE,
    wm_component_border_color2: ColorCode::new_rgb(0x79, 0x75, 0x71),
    wm_component_border_flat: false,
    wm_window_titlebar_back_color: ColorCode::new_rgb(0x0a, 0x24, 0x6a),
    wm_window_titlebar_fore_color: BASE_WHITE,
    io_buf_default_back_color: BASE_BLACK,
    io_buf_default_fore_color: BASE_BLACK,
};

#[allow(unused)]
const EGA_BLACK: ColorCode = BASE_BLACK;
const EGA_BLUE: ColorCode = ColorCode::new_rgb(0x00, 0x00, 0xaa);
const EGA_GREEN: ColorCode = ColorCode::new_rgb(0x00, 0xaa, 0x00);
const EGA_CYAN: ColorCode = ColorCode::new_rgb(0x00, 0xaa, 0xaa);
const EGA_RED: ColorCode = ColorCode::new_rgb(0xaa, 0x00, 0x00);
const EGA_MAGENTA: ColorCode = ColorCode::new_rgb(0xaa, 0x00, 0xaa);
const EGA_BROWN: ColorCode = ColorCode::new_rgb(0xaa, 0x55, 0x00);
const EGA_LIGHT_GRAY: ColorCode = ColorCode::new_rgb(0xaa, 0xaa, 0xaa);
const EGA_DARK_GRAY: ColorCode = ColorCode::new_rgb(0x55, 0x55, 0x55);
const EGA_LIGHT_BLUE: ColorCode = ColorCode::new_rgb(0x55, 0x55, 0xff);
const EGA_LIGHT_GREEN: ColorCode = ColorCode::new_rgb(0x55, 0xff, 0x55);
const EGA_LIGHT_CYAN: ColorCode = ColorCode::new_rgb(0x55, 0xff, 0xff);
const EGA_LIGHT_RED: ColorCode = ColorCode::new_rgb(0xff, 0x55, 0x55);
const EGA_LIGHT_MAGENTA: ColorCode = ColorCode::new_rgb(0xff, 0x55, 0xff);
const EGA_YELLOW: ColorCode = ColorCode::new_rgb(0xff, 0xff, 0x55);
const EGA_WHITE: ColorCode = BASE_WHITE;

#[allow(unused)]
const EGA_THEME: Theme = Theme {
    back_color: EGA_BLACK,
    fore_color: EGA_LIGHT_GREEN,
    sample_rect_colors: [
        EGA_BLACK,
        EGA_BLUE,
        EGA_GREEN,
        EGA_CYAN,
        EGA_RED,
        EGA_MAGENTA,
        EGA_BROWN,
        EGA_LIGHT_GRAY,
        EGA_DARK_GRAY,
        EGA_LIGHT_BLUE,
        EGA_LIGHT_GREEN,
        EGA_LIGHT_CYAN,
        EGA_LIGHT_RED,
        EGA_LIGHT_MAGENTA,
        EGA_YELLOW,
        EGA_WHITE,
    ],
    log_color_error: EGA_LIGHT_RED,
    log_color_warn: EGA_BROWN,
    log_color_info: EGA_LIGHT_CYAN,
    log_color_debug: EGA_YELLOW,
    log_color_trace: EGA_LIGHT_MAGENTA,
    wm_component_back_color: EGA_BLACK,
    wm_component_fore_color: EGA_LIGHT_GREEN,
    wm_component_border_color1: EGA_LIGHT_GREEN,
    wm_component_border_color2: EGA_LIGHT_GREEN,
    wm_component_border_flat: true,
    wm_window_titlebar_back_color: EGA_BLACK,
    wm_window_titlebar_fore_color: EGA_LIGHT_GREEN,
    io_buf_default_back_color: EGA_BLACK,
    io_buf_default_fore_color: EGA_BLACK,
};

#[allow(unused)]
pub struct Theme {
    // framebuffer
    pub back_color: ColorCode,
    pub fore_color: ColorCode,
    pub sample_rect_colors: [ColorCode; 16],
    // log
    pub log_color_error: ColorCode,
    pub log_color_warn: ColorCode,
    pub log_color_info: ColorCode,
    pub log_color_debug: ColorCode,
    pub log_color_trace: ColorCode,
    // simple wm
    pub wm_component_back_color: ColorCode,
    pub wm_component_fore_color: ColorCode,
    pub wm_component_border_color1: ColorCode, // left, top
    pub wm_component_border_color2: ColorCode, // right, bottom
    pub wm_component_border_flat: bool,
    pub wm_window_titlebar_back_color: ColorCode,
    pub wm_window_titlebar_fore_color: ColorCode,
    // io buffer
    pub io_buf_default_back_color: ColorCode,
    pub io_buf_default_fore_color: ColorCode,
}

#[allow(unused)]
pub fn debug_global_theme() {
    let theme = &GLOBAL_THEME;

    let _ = frame_buf_console::set_fore_color(theme.back_color);
    println!("back_color: {:?}", theme.back_color);

    let _ = frame_buf_console::set_fore_color(theme.fore_color);
    println!("fore_color: {:?}", theme.fore_color);

    for (i, color_code) in theme.sample_rect_colors.iter().enumerate() {
        let _ = frame_buf_console::set_fore_color(*color_code);
        println!("sample_rect_colors[{}]: {:?}", i, *color_code);
    }

    let _ = frame_buf_console::set_fore_color(theme.log_color_error);
    println!("log_color_error: {:?}", theme.log_color_error);

    let _ = frame_buf_console::set_fore_color(theme.log_color_warn);
    println!("log_color_warn: {:?}", theme.log_color_warn);

    let _ = frame_buf_console::set_fore_color(theme.log_color_info);
    println!("log_color_info: {:?}", theme.log_color_info);

    let _ = frame_buf_console::set_fore_color(theme.log_color_debug);
    println!("log_color_debug: {:?}", theme.log_color_debug);

    let _ = frame_buf_console::set_fore_color(theme.log_color_trace);
    println!("log_color_trace: {:?}", theme.log_color_trace);

    let _ = frame_buf_console::set_fore_color(theme.wm_component_back_color);
    println!(
        "wm_component_back_color: {:?}",
        theme.wm_component_back_color
    );

    let _ = frame_buf_console::set_fore_color(theme.wm_component_fore_color);
    println!(
        "wm_component_fore_color: {:?}",
        theme.wm_component_fore_color
    );

    let _ = frame_buf_console::set_fore_color(theme.wm_component_border_color1);
    println!(
        "wm_component_border_color1: {:?}",
        theme.wm_component_border_color1
    );

    let _ = frame_buf_console::set_fore_color(theme.wm_component_border_color2);
    println!(
        "wm_component_border_color2: {:?}",
        theme.wm_component_border_color2
    );

    let _ = frame_buf_console::set_fore_color(theme.wm_window_titlebar_back_color);
    println!(
        "wm_window_titlebar_back_color: {:?}",
        theme.wm_window_titlebar_back_color
    );

    let _ = frame_buf_console::set_fore_color(theme.wm_window_titlebar_fore_color);
    println!(
        "wm_window_titlebar_fore_color: {:?}",
        theme.wm_window_titlebar_fore_color
    );

    let _ = frame_buf_console::set_fore_color(theme.io_buf_default_back_color);
    println!(
        "io_buf_default_back_color: {:?}",
        theme.io_buf_default_back_color
    );

    let _ = frame_buf_console::set_fore_color(theme.io_buf_default_fore_color);
    println!(
        "io_buf_default_fore_color: {:?}",
        theme.io_buf_default_fore_color
    );

    let _ = frame_buf_console::reset_fore_color();
}
