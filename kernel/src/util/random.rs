use crate::device;

pub fn xorshift32(seed: u32) -> u32 {
    let mut x = seed;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}

pub fn xorshift64(seed: u64) -> u64 {
    let mut x = seed;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

pub fn xorshift64_seed_is_apic_timer() -> u64 {
    let seed = device::local_apic_timer::get_current_tick();
    xorshift64(seed as u64)
}
