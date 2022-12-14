use log::{Level, LevelFilter, Record, SetLoggerError};

use crate::{graphics::color::*, print, terminal::TERMINAL};

const LOG_COLOR_ERROR: RGBColor = COLOR_RED;
const LOG_COLOR_WARN: RGBColor = RGBColor { r: 253, g: 126, b: 0 }; // orange
const LOG_COLOR_INFO: RGBColor = COLOR_CYAN;
const LOG_COLOR_DEBUG: RGBColor = COLOR_YELLOW;
const LOG_COLOR_TRACE: RGBColor = COLOR_GREEN;

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<(), SetLoggerError>
{
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

struct SimpleLogger;

impl log::Log for SimpleLogger
{
    fn enabled(&self, metadata: &log::Metadata) -> bool { return metadata.level() <= Level::Info; }

    fn log(&self, record: &Record)
    {
        if self.enabled(record.metadata())
        {
            match record.level()
            {
                Level::Error => TERMINAL.lock().set_fore_color(LOG_COLOR_ERROR),
                Level::Warn => TERMINAL.lock().set_fore_color(LOG_COLOR_WARN),
                Level::Info => TERMINAL.lock().set_fore_color(LOG_COLOR_INFO),
                Level::Debug => TERMINAL.lock().set_fore_color(LOG_COLOR_DEBUG),
                Level::Trace => TERMINAL.lock().set_fore_color(LOG_COLOR_TRACE),
            }

            if record.level() == Level::Error || record.level() == Level::Debug
            {
                print!("[{}]: ", record.level());
            }
            else
            {
                print!("[ {}]: ", record.level());
            }

            if record.level() == Level::Error
            {
                print!("{}@{}: ", record.file().unwrap_or("Unknown"), record.line().unwrap_or(0));
            }

            print!("{:?}\n", record.args());

            TERMINAL.lock().reset_fore_color();
        }
    }

    fn flush(&self) {}
}
