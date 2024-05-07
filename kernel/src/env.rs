use crate::println;

pub const OS_NAME: &str = "myos";
pub const ENV_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const ENV_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const ENV_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const ASCII_ART: &str =
    "                                               ___     __     __  _  _   \n                                              / _ \\   / /    / / | || |  \n  _ __ ___   _   _   ___   ___  ______ __  __| (_) | / /_   / /_ | || |_ \n | '_ ` _ \\ | | | | / _ \\ / __||______|\\ \\/ / > _ < | '_ \\ | '_ \\|__   _|\n | | | | | || |_| || (_) |\\__ \\         >  < | (_) || (_) || (_) |  | |  \n |_| |_| |_| \\__, | \\___/ |___/        /_/\\_\\ \\___/  \\___/  \\___/   |_|  \n              __/ |                                    ______            \n             |___/                                    |______|           ";

pub fn print_info() {
    println!("{}", ASCII_ART);
    println!("{}", OS_NAME);
    println!("Version: {}", ENV_VERSION);
    println!("Authors: {}", ENV_AUTHORS);
    println!("Description: {}", ENV_DESCRIPTION);
}
