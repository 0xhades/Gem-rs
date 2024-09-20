use std::env;

pub mod api;
pub mod client;
pub mod errors;
pub mod types;
mod utils;

pub fn init_log() {
    pretty_env_logger::init();
    env::set_var("RUST_LOG", "info");
    log::info!("Logger initialized");
}

/*
- TODO: Log the responses in case of an error

- file caching:
    - local cache within settings [shared with all contexts] per API Key (never upload same file twice)
    - gemini cache [upload then check if file proccessed in loop]
*/
