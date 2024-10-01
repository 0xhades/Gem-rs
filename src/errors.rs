//! Error types for the Gem-rs library.
//!
//! This module defines custom error types used throughout the Gem-rs library,
//! providing detailed information about various error conditions that may occur
//! during interactions with the Gemini API.

use std::error::Error;
use std::fmt;

use crate::types;

/// Represents errors that can occur in the Gem-rs library.
#[derive(Debug)]
pub enum GemError {
    /// Indicates that an empty response was received from the API.
    EmptyApiResponse,

    /// Represents a connection error when communicating with the API.
    ConnectionError(reqwest::Error),

    /// Represents an error in the API response, including the HTTP status code.
    ResponseError((reqwest::Error, reqwest::StatusCode)),

    /// Indicates that the prompt feedback was blocked by the API.
    PromptFeedbackBlocked,

    /// Indicates that all candidate responses were blocked by the API.
    AllCandidatesBlocked,

    /// Represents an error returned by the Gemini API.
    GeminiAPIError(types::Error),

    /// Represents an error that occurred while parsing the API response.
    ParsingError(serde_json::Error),

    /// Represents an error that occurred during the feedback process.
    FeedbackError(String),

    /// Represents an error that occurred while streaming data.
    StreamError(String),

    /// Represents an error related to file operations.
    FileError(String),
}

impl fmt::Display for GemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GemError::EmptyApiResponse => write!(f, "Received an empty response from the API"),
            GemError::PromptFeedbackBlocked => write!(f, "Prompt feedback state is blocked"),
            GemError::AllCandidatesBlocked => write!(f, "All candidates have a block error"),
            GemError::ConnectionError(e) => write!(f, "Connection error: {}", e),
            GemError::ParsingError(e) => write!(f, "Parsing error: {}", e),
            GemError::GeminiAPIError(e) => write!(f, "Gemini API error: {}", e),
            GemError::ResponseError((e, status)) => {
                write!(f, "Response error: {} (status code: {})", e, status)
            }
            GemError::FeedbackError(e) => write!(f, "Feedback error: {}", e),
            GemError::StreamError(e) => write!(f, "Stream error: {}", e),
            GemError::FileError(e) => write!(f, "File error: {}", e),
        }
    }
}

impl Error for GemError {}

/// Represents the reason why the Gemini API finished generating content.
#[derive(Debug)]
pub enum FinishReason {
    /// Indicates that the generation was stopped due to safety concerns.
    Safety,

    /// Indicates that the generation was stopped due to content recitation.
    Recitation,

    /// Represents any other reason for finishing content generation.
    Other,
}

impl FinishReason {
    /// Determines if the finish reason is considered a block error.
    ///
    /// # Returns
    ///
    /// `true` if the finish reason is either `Safety` or `Recitation`, `false` otherwise.
    pub fn is_block_error(&self) -> bool {
        matches!(self, FinishReason::Safety | FinishReason::Recitation)
    }
}
