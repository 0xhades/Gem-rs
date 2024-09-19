use std::env;

pub mod api;
pub mod client;
pub mod errors;
pub mod types;

pub fn init_log() {
    pretty_env_logger::init();
    env::set_var("RUST_LOG", "info");
    log::info!("Logger initialized");
}

/*
- TODO: Log the responses in case of an error
*/
