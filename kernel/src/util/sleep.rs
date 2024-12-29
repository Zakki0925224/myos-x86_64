use crate::device;

pub fn sleep_ms(ms: usize) {
    let start_ms = match device::local_apic_timer::get_current_ms() {
        Some(ms) => ms,
        None => return,
    };

    loop {
        let current_ms = device::local_apic_timer::get_current_ms().unwrap();

        if current_ms - start_ms >= ms {
            break;
        }
    }
}
