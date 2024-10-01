//! Gem-rs: A Rust library for interacting with the Gemini API
//!
//! This library serves as a wrapper around the Gemini API, providing support for
//! streaming, file uploads, and various other functionalities. It is designed to
//! facilitate easy integration of Gemini API capabilities into Rust projects.
//!
//! # Features
//!
//! - Streaming support for real-time interactions
//! - File and image upload capabilities
//! - Caching mechanism for efficient file handling
//! - Comprehensive error handling and logging
//! - Support for multiple Gemini API models
//!
//! # Modules
//!
//! - `api`: Contains API-related constants and model definitions
//! - `client`: Provides the main client interface for interacting with the Gemini API
//! - `errors`: Defines custom error types for the library
//! - `types`: Contains various type definitions used throughout the library
//! - `utils`: Utility functions for internal use

use std::env;

pub mod api;
pub mod client;
pub mod errors;
pub mod types;
mod utils;

/// Initializes the logger for the Gem-rs library.
///
/// This function sets up the logging system using `pretty_env_logger` and sets
/// the default log level to "info". It's recommended to call this function at
/// the start of your application to enable logging for the Gem-rs library.
///
/// # Example
///
/// ```
/// use gem_rs::init_log;
///
/// fn main() {
///     init_log();
///     // Your application code here
/// }
/// ```
pub fn init_log() {
    pretty_env_logger::init();
    env::set_var("RUST_LOG", "info");
    log::info!("Logger initialized");
}