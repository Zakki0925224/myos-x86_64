use log::{Level, LevelFilter, Record};

use crate::{
    graphics::{color::*, frame_buf_console::FRAME_BUF_CONSOLE},
    print,
};

const LOG_COLOR_ERROR: RgbColor = COLOR_RED;
const LOG_COLOR_WARN: RgbColor = RgbColor::new(253, 126, 0); // orange
const LOG_COLOR_INFO: RgbColor = COLOR_CYAN;
const LOG_COLOR_DEBUG: RgbColor = COLOR_YELLOW;
const LOG_COLOR_TRACE: RgbColor = COLOR_GREEN;

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .unwrap();
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        return metadata.level() <= Level::Info;
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if let Some(mut frame_buf_console) = FRAME_BUF_CONSOLE.try_lock() {
            let frame_buf_console = match frame_buf_console.as_mut() {
                Some(f) => f,
                None => return,
            };

            match record.level() {
                Level::Error => frame_buf_console.set_fore_color(LOG_COLOR_ERROR),
                Level::Warn => frame_buf_console.set_fore_color(LOG_COLOR_WARN),
                Level::Info => frame_buf_console.set_fore_color(LOG_COLOR_INFO),
                Level::Debug => frame_buf_console.set_fore_color(LOG_COLOR_DEBUG),
                Level::Trace => frame_buf_console.set_fore_color(LOG_COLOR_TRACE),
            }
        }

        if record.level() == Level::Error || record.level() == Level::Debug {
            print!("[{}]: ", record.level());
        } else {
            print!("[ {}]: ", record.level());
        }

        if record.level() == Level::Error {
            print!(
                "{}@{}: ",
                record.file().unwrap_or("Unknown"),
                record.line().unwrap_or(0)
            );
        }

        print!("{:?}\n", record.args());

        if let Some(mut frame_buf_console) = FRAME_BUF_CONSOLE.try_lock() {
            let frame_buf_console = match frame_buf_console.as_mut() {
                Some(f) => f,
                None => return,
            };

            frame_buf_console.reset_fore_color();
        }
    }

    fn flush(&self) {}
}
