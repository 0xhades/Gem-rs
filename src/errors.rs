use std::error::Error;
use std::fmt;

use crate::types;

#[derive(Debug)]
pub enum GemError {
    EmptyApiResponse,
    ConnectionError(reqwest::Error),
    ResponseError((reqwest::Error, reqwest::StatusCode)),
    PromptFeedbackBlocked,
    AllCandidatesBlocked,
    GeminiAPIError(types::Error),
    ParsingError(serde_json::Error),
    FeedbackError(String),
    StreamError(String),
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

// Example usage of FinishReason enum
#[derive(Debug)]
pub enum FinishReason {
    Safety,
    Recitation,
    Other,
}

impl FinishReason {
    pub fn is_block_error(&self) -> bool {
        matches!(self, FinishReason::Safety | FinishReason::Recitation)
    }
}
