use log::{Level, LevelFilter, Record, SetLoggerError};

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

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        return metadata.level() <= Level::Info;
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match record.level() {
                Level::Error => FRAME_BUF_CONSOLE.lock().set_fore_color(LOG_COLOR_ERROR),
                Level::Warn => FRAME_BUF_CONSOLE.lock().set_fore_color(LOG_COLOR_WARN),
                Level::Info => FRAME_BUF_CONSOLE.lock().set_fore_color(LOG_COLOR_INFO),
                Level::Debug => FRAME_BUF_CONSOLE.lock().set_fore_color(LOG_COLOR_DEBUG),
                Level::Trace => FRAME_BUF_CONSOLE.lock().set_fore_color(LOG_COLOR_TRACE),
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

            FRAME_BUF_CONSOLE.lock().reset_fore_color();
        }
    }

    fn flush(&self) {}
}
