use crate::{arch, graphics::frame_buf_console, print, theme::GLOBAL_THEME};
use log::{Level, LevelFilter, Record};

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
            Level::Error => GLOBAL_THEME.log_color_error,
            Level::Warn => GLOBAL_THEME.log_color_warn,
            Level::Info => GLOBAL_THEME.log_color_info,
            Level::Debug => GLOBAL_THEME.log_color_debug,
            Level::Trace => GLOBAL_THEME.log_color_trace,
        };

        let _ = frame_buf_console::set_fore_color(fore_color);

        let ms = arch::apic::timer::get_current_ms().unwrap_or(0);
        print!(
            "[{:06}.{:03}][{}{}]: ",
            ms / 1000,
            ms % 1000,
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
