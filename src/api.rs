use serde::{Deserialize, Serialize};

pub const GENERATE_CONTENT: &str = "https://generativelanguage.googleapis.com/v1beta/models/";
pub const STREAM_GENERATE_CONTENT: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Models {
    #[serde(rename = "gemini-1.5-pro-exp-0827")]
    Gemini15ProExp0827,
    #[serde(rename = "gemini-1.5-flash-exp-0827")]
    Gemini15FlashExp0827,
    #[serde(rename = "gemini-1.5-flash-8b-exp-0827")]
    Gemini15Flash8bExp0827,
    #[default]
    #[serde(rename = "gemini-1.5-pro")]
    Gemini15Pro,
    #[serde(rename = "gemini-1.5-flash")]
    Gemini15Flash,
    #[serde(rename = "gemini-1.0-pro")]
    Gemini10Pro,
    #[serde(rename = "gemma-2-2b-it")]
    Gemma2_2bIt,
    #[serde(rename = "gemma-2-9b-it")]
    Gemma2_9bIt,
    #[serde(rename = "gemma-2-27b-it")]
    Gemma2_27bIt,
}

impl ToString for Models {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap().replace("\"", "")
    }
}
