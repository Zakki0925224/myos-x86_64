use crate::println;

pub const OS_NAME: &str = "myos";
pub const ENV_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const ENV_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const ENV_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn print_info() {
    println!("{}", OS_NAME);
    println!("Version: {}", ENV_VERSION);
    println!("Authors: {}", ENV_AUTHORS);
    println!("Description: {}", ENV_DESCRIPTION);
}
