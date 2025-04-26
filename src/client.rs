//! Client module for interacting with the Gemini API.
//!
//! This module provides the main structures and implementations for creating and managing
//! sessions with the Gemini API, including support for sending messages, files, and blobs,
//! as well as streaming responses.

use super::types::Context;
use dotenv::dotenv;
use error::StreamBodyError;
use futures::Stream;
use reqwest::{Client as webClient, StatusCode};
use reqwest_streams::*;

use crate::api::{Models, GENERATE_CONTENT, STREAM_GENERATE_CONTENT};
use crate::errors::GemError;
use crate::types::{Blob, Error, FileData, GenerateContentResponse, Role, Settings};

pub type StreamResponseResult = Result<
    Box<dyn Stream<Item = Result<GenerateContentResponse, StreamBodyError>> + Unpin>,
    GemError,
>;
pub type ResponseResult = Result<GenerateContentResponse, GemError>;

pub type StreamResponse = Box<
    dyn futures::Stream<
            Item = Result<GenerateContentResponse, reqwest_streams::error::StreamBodyError>,
        > + Unpin,
>;

pub type Response = GenerateContentResponse;

/// Represents a session with the Gemini API.
pub struct GemSession {
    client: Client,
    context: Context,
}

/// Builder for creating a `GemSession` with custom configurations.
pub struct GemSessionBuilder(Config);

/// Internal configuration structure for `GemSessionBuilder`.
pub struct Config {
    timeout: std::time::Duration,
    connect_timeout: std::time::Duration,
    model: Models,
    context: Context,
}

impl GemSessionBuilder {
    /// Creates a new `GemSessionBuilder` with default settings.
    pub fn new() -> GemSessionBuilder {
        GemSessionBuilder(Config {
            timeout: std::time::Duration::from_secs(30),
            connect_timeout: std::time::Duration::from_secs(30),
            model: Models::default(),
            context: Context::new(),
        })
    }

    /// Creates a default `GemSession` with the provided API key.
    pub fn default(api_key: String) -> GemSession {
        GemSession {
            client: Client::new(
                api_key,
                Models::default(),
                std::time::Duration::from_secs(30),
                std::time::Duration::from_secs(30),
            ),
            context: Context::new(),
        }
    }

    /// Sets the timeout for API requests.
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.0.timeout = timeout;
        self
    }

    /// Sets the Gemini model to use for the session.
    pub fn model(mut self, model: Models) -> Self {
        self.0.model = model;
        self
    }

    /// Sets a custom model to use for the session.
    pub fn custom_model(mut self, model: String) -> Self {
        self.0.model = Models::Custom(model);
        self
    }

    /// Sets the connection timeout for API requests.
    pub fn connect_timeout(mut self, connect_timeout: std::time::Duration) -> Self {
        self.0.connect_timeout = connect_timeout;
        self
    }

    /// Sets the initial context for the session.
    pub fn context(mut self, context: Context) -> Self {
        self.0.context = context;
        self
    }

    /// Builds a `GemSession` with the configured settings and provided API key.
    pub fn build(self) -> GemSession {
        dotenv().expect("Failed to load Gemini API key");
        let api_key = std::env::var("GEMINI_API_KEY").unwrap();
        GemSession::build(api_key, self.0)
    }
}

/// Internal client for making API requests to Gemini.
pub struct Client {
    client: webClient,
    api_key: String,
    model: Models,
}

impl Client {
    /// Creates a new `Client` instance.
    pub fn new(
        api_key: String,
        model: Models,
        timeout: std::time::Duration,
        connect_timeout: std::time::Duration,
    ) -> Self {
        Client {
            client: webClient::builder()
                .timeout(timeout)
                .connect_timeout(connect_timeout)
                .build()
                .unwrap_or(webClient::new()),
            api_key,
            model,
        }
    }

    /// Sends a context to the Gemini API and returns the response.
    pub(crate) async fn send_context(
        &self,
        context: &Context,
        settings: &Settings,
    ) -> ResponseResult {
        let url = format!(
            "{}{}:generateContent",
            GENERATE_CONTENT,
            self.model.to_string()
        );

        log::info!("URL: {}", url);

        let context = context.build(settings);
        log::info!("Request: {:#?}", serde_json::to_string(&context).unwrap());

        let response = match self
            .client
            .post(url)
            .query(&[("key", &self.api_key)])
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&context)
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => return Err(GemError::ConnectionError(e)),
        };

        let status_code = response.status();
        let response_text = match response.text().await {
            Ok(text) => text,
            Err(e) => return Err(GemError::ResponseError((e, status_code))),
        };

        log::info!("Response: {}", response_text);

        let response = match status_code {
            StatusCode::OK => match serde_json::from_str::<GenerateContentResponse>(&response_text)
            {
                Ok(response) => response,
                Err(e) => {
                    return Err(GemError::ParsingError(e));
                }
            },
            _ => match serde_json::from_str::<Error>(&response_text) {
                Ok(error) => {
                    return Err(GemError::GeminiAPIError(error));
                }
                Err(e) => return Err(GemError::ParsingError(e)),
            },
        };

        if response.get_candidates().len() == 0 {
            return Err(GemError::EmptyApiResponse);
        }

        let mut blocked = true;
        for candidate in response.get_candidates() {
            if candidate.get_content().is_some()
            /*&& !candidate.is_blocked()*/
            {
                blocked = false;
                break;
            }
        }

        if blocked {
            if let Some(reason) = response.feedback() {
                return Err(GemError::FeedbackError(reason.to_string()));
            }
            return Err(GemError::AllCandidatesBlocked);
        }

        Ok(response)
    }

    /// Sends a context to the Gemini API and returns a stream of responses.
    pub(crate) async fn send_context_stream(
        &self,
        context: &Context,
        settings: &Settings,
    ) -> StreamResponseResult {
        let url = format!(
            "{}{}:streamGenerateContent",
            STREAM_GENERATE_CONTENT,
            self.model.to_string()
        );

        let response = self
            .client
            .post(url)
            .query(&[("key", &self.api_key)])
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&context.build(settings))
            .send()
            .await;

        match response {
            Ok(response) => {
                let status_code = response.status();
                match status_code {
                    StatusCode::OK => {
                        let json_stream = response.json_array_stream::<GenerateContentResponse>(
                            settings.get_stream_max_json_size() as usize,
                        );
                        Ok(Box::new(json_stream))
                    }
                    _ => {
                        return Err(GemError::StreamError(format!(
                            "Response error: {} (status code: {})",
                            response.text().await.unwrap(),
                            status_code
                        )));
                    }
                }
            }

            Err(e) => {
                return Err(GemError::ConnectionError(e));
            }
        }
    }
}

impl GemSession {
    /// Builds a new `GemSession` with the provided API key and configuration.
    pub(crate) fn build(api_key: String, config: Config) -> Self {
        GemSession {
            client: Client::new(
                api_key,
                config.model,
                config.timeout,
                config.connect_timeout,
            ),
            context: config.context,
        }
    }

    /// Creates a new `GemSession` with default settings and the provided API key.
    pub fn new(api_key: String) -> Self {
        GemSessionBuilder::default(api_key)
    }

    /// Returns a new `GemSessionBuilder` for creating a customized `GemSession`.
    pub fn Builder() -> GemSessionBuilder {
        GemSessionBuilder::new()
    }

    /// Sends a message to the Gemini API and returns the response.
    pub async fn send_message(
        &mut self,
        message: &str,
        role: Role,
        settings: &Settings,
    ) -> ResponseResult {
        self.context.push_message(role, message.to_string());
        let response = self.send_context(settings).await?;
        if let Some(candidate) = response.get_candidates().first() {
            if let Some(content) = candidate.get_content() {
                self.context.push_message(
                    Role::Model,
                    match content.get_text() {
                        Some(text) => text.clone(),
                        None => return Err(GemError::EmptyApiResponse),
                    },
                );
            }
        }
        Ok(response)
    }

    /// Sends a file to the Gemini API and returns the response.
    pub async fn send_file(
        &mut self,
        file_data: FileData,
        role: Role,
        settings: &Settings,
    ) -> ResponseResult {
        self.context.push_file(role, file_data);

        let response = self.send_context(settings).await?;
        if let Some(candidate) = response.get_candidates().first() {
            if let Some(content) = candidate.get_content() {
                self.context.push_message(
                    Role::Model,
                    match content.get_text() {
                        Some(text) => text.clone(),
                        None => return Err(GemError::EmptyApiResponse),
                    },
                );
            }
        }
        Ok(response)
    }

    /// Sends a blob to the Gemini API and returns the response.
    pub async fn send_blob(
        &mut self,
        blob: Blob,
        role: Role,
        settings: &Settings,
    ) -> ResponseResult {
        self.context.push_blob(role, blob);
        let response = self.send_context(settings).await?;
        if let Some(candidate) = response.get_candidates().first() {
            if let Some(content) = candidate.get_content() {
                self.context.push_message(
                    Role::Model,
                    match content.get_text() {
                        Some(text) => text.clone(),
                        None => return Err(GemError::EmptyApiResponse),
                    },
                );
            }
        }
        Ok(response)
    }

    /// Sends a message with an attached file to the Gemini API and returns the response.
    pub async fn send_message_with_file(
        &mut self,
        message: &str,
        file_data: FileData,
        role: Role,
        settings: &Settings,
    ) -> ResponseResult {
        self.context
            .push_message_with_file(role, message, file_data);
        let response = self.send_context(settings).await?;
        if let Some(candidate) = response.get_candidates().first() {
            if let Some(content) = candidate.get_content() {
                self.context.push_message(
                    Role::Model,
                    match content.get_text() {
                        Some(text) => text.clone(),
                        None => return Err(GemError::EmptyApiResponse),
                    },
                );
            }
        }
        Ok(response)
    }

    /// Sends a message with an attached blob to the Gemini API and returns the response.
    pub async fn send_message_with_blob(
        &mut self,
        message: &str,
        blob: Blob,
        role: Role,
        settings: &Settings,
    ) -> ResponseResult {
        self.context.push_message_with_blob(role, message, blob);
        let response = self.send_context(settings).await?;
        if let Some(candidate) = response.get_candidates().first() {
            if let Some(content) = candidate.get_content() {
                self.context.push_message(
                    Role::Model,
                    match content.get_text() {
                        Some(text) => text.clone(),
                        None => return Err(GemError::EmptyApiResponse),
                    },
                );
            }
        }
        Ok(response)
    }

    /// Sends a message to the Gemini API and returns a stream of responses.
    pub async fn send_message_stream(
        &mut self,
        message: &str,
        role: Role,
        settings: &Settings,
    ) -> StreamResponseResult {
        self.context.push_message(role, message.to_string());
        Ok(Box::new(self.send_context_stream(settings).await?))
    }

    /// Sends a file to the Gemini API and returns a stream of responses.
    pub async fn send_file_stream(
        &mut self,
        file_data: FileData,
        role: Role,
        settings: &Settings,
    ) -> StreamResponseResult {
        self.context.push_file(role, file_data);
        Ok(Box::new(self.send_context_stream(settings).await?))
    }

    /// Sends a blob to the Gemini API and returns a stream of responses.
    pub async fn send_blob_stream(
        &mut self,
        blob: Blob,
        role: Role,
        settings: &Settings,
    ) -> StreamResponseResult {
        self.context.push_blob(role, blob);
        Ok(Box::new(self.send_context_stream(settings).await?))
    }

    /// Sends a message with an attached file to the Gemini API and returns a stream of responses.
    pub async fn send_message_with_file_stream(
        &mut self,
        message: &str,
        file_data: FileData,
        role: Role,
        settings: &Settings,
    ) -> StreamResponseResult {
        self.context
            .push_message_with_file(role, message, file_data);
        Ok(Box::new(self.send_context_stream(settings).await?))
    }

    /// Sends a message with an attached blob to the Gemini API and returns a stream of responses.
    pub async fn send_message_with_blob_stream(
        &mut self,
        message: &str,
        blob: Blob,
        role: Role,
        settings: &Settings,
    ) -> StreamResponseResult {
        self.context.push_message_with_blob(role, message, blob);
        Ok(Box::new(self.send_context_stream(settings).await?))
    }

    /// Internal method to send a context to the Gemini API.
    async fn send_context(&mut self, settings: &Settings) -> ResponseResult {
        self.client.send_context(&self.context, settings).await
    }

    /// Internal method to send a context to the Gemini API and return a stream of responses.
    async fn send_context_stream(&mut self, settings: &Settings) -> StreamResponseResult {
        self.client
            .send_context_stream(&self.context, settings)
            .await
    }
}

mod tests {

    #[tokio::test]
    async fn test_gem_session_send_context() {
        dotenv().expect("Failed to load Gemini API key");
        let api_key = std::env::var("GEMINI_API_KEY").unwrap();

        let mut session = GemSession::Builder()
            .connect_timeout(std::time::Duration::from_secs(30))
            .timeout(std::time::Duration::from_secs(30))
            .model(Models::Gemini15FlashExp0827)
            .context(Context::new())
            .build();

        let mut settings = Settings::new();
        settings.set_all_safety_settings(HarmBlockThreshold::BlockNone);

        let response = session
            .send_message("Hello! What is your name?", Role::User, &settings)
            .await;
    }

    #[test]
    fn test_models_display() {
        let model = Models::Gemini15ProExp0827;
        assert_eq!(model.to_string(), "gemini-1.5-pro-exp-0827");

        let model = Models::Gemini15FlashExp0827;
        assert_eq!(model.to_string(), "gemini-1.5-flash-exp-0827");

        let model = Models::Gemini15Flash8bExp0827;
        assert_eq!(model.to_string(), "gemini-1.5-flash-8b-exp-0827");

        let model = Models::Gemini15Pro;
        assert_eq!(model.to_string(), "gemini-1.5-pro");

        let model = Models::Gemini15Flash;
        assert_eq!(model.to_string(), "gemini-1.5-flash");

        let model = Models::Gemini10Pro;
        assert_eq!(model.to_string(), "gemini-1.0-pro");

        let model = Models::Gemma2_2bIt;
        assert_eq!(model.to_string(), "gemma-2-2b-it");

        let model = Models::Gemma2_9bIt;
        assert_eq!(model.to_string(), "gemma-2-9b-it");

        let model = Models::Gemma2_27bIt;
        assert_eq!(model.to_string(), "gemma-2-27b-it");

        let model = Models::Custom("gemini-3-flash-001".to_string());
        assert_eq!(model.to_string(), "gemini-3-flash-001");
    }
}
