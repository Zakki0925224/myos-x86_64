use log::{Level, LevelFilter, Record};

use crate::{
    arch,
    graphics::{color::*, frame_buf_console},
    print,
};

const LOG_COLOR_ERROR: ColorCode = COLOR_RED;
const LOG_COLOR_WARN: ColorCode = ColorCode::new(253, 126, 0); // orange
const LOG_COLOR_INFO: ColorCode = COLOR_CYAN;
const LOG_COLOR_DEBUG: ColorCode = COLOR_YELLOW;
const LOG_COLOR_TRACE: ColorCode = COLOR_GREEN;

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::max()))
        .unwrap();
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let fore_color = match record.level() {
            Level::Error => LOG_COLOR_ERROR,
            Level::Warn => LOG_COLOR_WARN,
            Level::Info => LOG_COLOR_INFO,
            Level::Debug => LOG_COLOR_DEBUG,
            Level::Trace => LOG_COLOR_TRACE,
        };

        let _ = frame_buf_console::set_fore_color(fore_color);

        let local_apic_timer_tick = arch::apic::timer::get_current_tick();

        print!(
            "[tk={}][{}{}]: ",
            local_apic_timer_tick,
            if record.level() == Level::Error || record.level() == Level::Debug {
                ""
            } else {
                " "
            },
            record.level()
        );

        if record.level() == Level::Error {
            print!(
                "{}@{}: ",
                record.file().unwrap_or("Unknown"),
                record.line().unwrap_or(0)
            );
        }

        print!("{:?}\n", record.args());

        let _ = frame_buf_console::reset_fore_color();
    }

    fn flush(&self) {}
}
