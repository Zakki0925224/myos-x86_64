use crate::println;

pub const ENV_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const ENV_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const ENV_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");
pub const ENV_NAME: &str = env!("CARGO_PKG_NAME");
pub const ENV_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ENV_VERSION_MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
pub const ENV_VERSION_MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");
pub const ENV_VERSION_PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");
pub const ENV_VERSION_PRE: &str = env!("CARGO_PKG_VERSION_PRE");

pub fn print_info()
{
    println!("myos");
    println!("Version: {}", ENV_VERSION);
    println!("Authors: {}", ENV_AUTHORS);
    println!("Description: {}", ENV_DESCRIPTION);
}
