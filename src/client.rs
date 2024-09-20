use super::types::Context;
use error::StreamBodyError;
use futures::stream::TryStreamExt;
use futures::Stream;
use reqwest::{Client as webClient, StatusCode};
use reqwest_streams::*;

use crate::api::{Models, GENERATE_CONTENT, STREAM_GENERATE_CONTENT};
use crate::errors::GemError;
use crate::types::{Blob, Error, FileData, FileManager, GenerateContentResponse, Role, Settings};

// Builder
pub struct GemSession {
    client: Client,
    context: Context,
}
pub struct GemSessionBuilder(Config);
struct Config {
    timeout: std::time::Duration,
    connect_timeout: std::time::Duration,
    model: Models,
    context: Context,
}

impl GemSessionBuilder {
    pub fn new() -> GemSessionBuilder {
        GemSessionBuilder(Config {
            timeout: std::time::Duration::from_secs(30),
            connect_timeout: std::time::Duration::from_secs(30),
            model: Models::default(),
            context: Context::new(),
        })
    }

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

    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.0.timeout = timeout;
        self
    }

    pub fn model(mut self, model: Models) -> Self {
        self.0.model = model;
        self
    }

    pub fn connect_timeout(mut self, connect_timeout: std::time::Duration) -> Self {
        self.0.connect_timeout = connect_timeout;
        self
    }

    pub fn context(mut self, context: Context) -> Self {
        self.0.context = context;
        self
    }

    pub fn build(self, api_key: String) -> GemSession {
        GemSession::build(api_key, self.0)
    }
}

pub struct Client {
    client: webClient,
    api_key: String,
    model: Models,
}

impl Client {
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

    pub(crate) async fn send_context(
        &self,
        context: &Context,
        settings: &Settings,
    ) -> Result<GenerateContentResponse, GemError> {
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

    pub(crate) async fn send_context_stream(
        &self,
        context: &Context,
        settings: &Settings,
    ) -> Result<impl Stream<Item = Result<GenerateContentResponse, StreamBodyError>>, GemError>
    {
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
                        let json_stream =
                            response.json_array_stream::<GenerateContentResponse>(2048);
                        Ok(json_stream)
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

    pub fn new(api_key: String) -> Self {
        GemSessionBuilder::default(api_key)
    }

    pub fn Builder() -> GemSessionBuilder {
        GemSessionBuilder::new()
    }

    //WARNING: You shouldn't call send methods with files, without consulting the FileCache first
    //TODO: Implement more restrictive file caching

    pub async fn send_message(
        &mut self,
        message: &str,
        settings: &Settings,
    ) -> Result<GenerateContentResponse, GemError> {
        self.context.push_message(None, message.to_string());
        let response = self.send_context(settings).await?;
        if let Some(candidate) = response.get_candidates().first() {
            if let Some(content) = candidate.get_content() {
                self.context.push_message(
                    Some(Role::Model),
                    match content.get_text() {
                        Some(text) => text.clone(),
                        None => return Err(GemError::EmptyApiResponse),
                    },
                );
            }
        }
        Ok(response)
    }

    pub async fn send_file(
        &mut self,
        file_data: FileData,
        settings: &Settings,
    ) -> Result<GenerateContentResponse, GemError> {
        self.context.push_file(None, file_data);

        let response = self.send_context(settings).await?;
        if let Some(candidate) = response.get_candidates().first() {
            if let Some(content) = candidate.get_content() {
                self.context.push_message(
                    Some(Role::Model),
                    match content.get_text() {
                        Some(text) => text.clone(),
                        None => return Err(GemError::EmptyApiResponse),
                    },
                );
            }
        }
        Ok(response)
    }

    // inline data with the request without caching and storing within the context (prompt with one time)
    pub async fn send_blob(
        &mut self,
        blob: Blob,
        settings: &Settings,
    ) -> Result<GenerateContentResponse, GemError> {
        self.context.push_blob(None, blob);
        let response = self.send_context(settings).await?;
        if let Some(candidate) = response.get_candidates().first() {
            if let Some(content) = candidate.get_content() {
                self.context.push_message(
                    Some(Role::Model),
                    match content.get_text() {
                        Some(text) => text.clone(),
                        None => return Err(GemError::EmptyApiResponse),
                    },
                );
            }
        }
        Ok(response)
    }

    pub async fn send_message_with_file(
        &mut self,
        message: &str,
        file_data: FileData,
        settings: &Settings,
    ) -> Result<GenerateContentResponse, GemError> {
        self.context
            .push_message_with_file(None, message, file_data);
        let response = self.send_context(settings).await?;
        if let Some(candidate) = response.get_candidates().first() {
            if let Some(content) = candidate.get_content() {
                self.context.push_message(
                    Some(Role::Model),
                    match content.get_text() {
                        Some(text) => text.clone(),
                        None => return Err(GemError::EmptyApiResponse),
                    },
                );
            }
        }
        Ok(response)
    }

    pub async fn send_message_with_blob(
        &mut self,
        message: &str,
        blob: Blob,
        settings: &Settings,
    ) -> Result<GenerateContentResponse, GemError> {
        self.context.push_message_with_blob(None, message, blob);
        let response = self.send_context(settings).await?;
        if let Some(candidate) = response.get_candidates().first() {
            if let Some(content) = candidate.get_content() {
                self.context.push_message(
                    Some(Role::Model),
                    match content.get_text() {
                        Some(text) => text.clone(),
                        None => return Err(GemError::EmptyApiResponse),
                    },
                );
            }
        }
        Ok(response)
    }

    pub async fn send_message_stream(
        &mut self,
        message: &str,
        settings: &Settings,
    ) -> Result<impl Stream<Item = Result<GenerateContentResponse, StreamBodyError>>, GemError>
    {
        self.context.push_message(None, message.to_string());
        self.send_context_stream(settings).await
    }

    pub async fn send_file_stream(
        &mut self,
        file_data: FileData,
        settings: &Settings,
    ) -> Result<impl Stream<Item = Result<GenerateContentResponse, StreamBodyError>>, GemError>
    {
        self.context.push_file(None, file_data);
        self.send_context_stream(settings).await
    }

    pub async fn send_blob_stream(
        &mut self,
        blob: Blob,
        settings: &Settings,
    ) -> Result<impl Stream<Item = Result<GenerateContentResponse, StreamBodyError>>, GemError>
    {
        self.context.push_blob(None, blob);
        self.send_context_stream(settings).await
    }

    pub async fn send_message_with_file_stream(
        &mut self,
        message: &str,
        file_data: FileData,
        settings: &Settings,
    ) -> Result<impl Stream<Item = Result<GenerateContentResponse, StreamBodyError>>, GemError>
    {
        self.context
            .push_message_with_file(None, message, file_data);
        self.send_context_stream(settings).await
    }

    pub async fn send_message_with_blob_stream(
        &mut self,
        message: &str,
        blob: Blob,
        settings: &Settings,
    ) -> Result<impl Stream<Item = Result<GenerateContentResponse, StreamBodyError>>, GemError>
    {
        self.context.push_message_with_blob(None, message, blob);
        self.send_context_stream(settings).await
    }

    async fn send_context(
        &mut self,
        settings: &Settings,
    ) -> Result<GenerateContentResponse, GemError> {
        self.client.send_context(&self.context, settings).await
    }

    async fn send_context_stream(
        &mut self,
        settings: &Settings,
    ) -> Result<impl Stream<Item = Result<GenerateContentResponse, StreamBodyError>>, GemError>
    {
        self.client
            .send_context_stream(&self.context, settings)
            .await
    }
}

mod tests {

    use crate::types::HarmBlockThreshold;

    use super::*;

    const API_KEY: &str = "X";

    #[tokio::test]
    async fn test_gem_session_send_context() {
        let mut session = GemSession::Builder()
            .connect_timeout(std::time::Duration::from_secs(30))
            .timeout(std::time::Duration::from_secs(30))
            .model(Models::Gemini15FlashExp0827)
            .context(Context::new())
            .build(API_KEY.to_string());

        let mut settings = Settings::new();
        settings.set_all_safety_settings(HarmBlockThreshold::BlockNone);

        let response = session
            .send_message("Hello! What is your name?", &settings)
            .await;
    }

    #[test]
    fn test_models_display() {
        let model = Models::Gemini15ProExp0827;
        assert_eq!(model.to_string(), "\"gemini-1.5-pro-exp-0827\"");

        let model = Models::Gemini15FlashExp0827;
        assert_eq!(model.to_string(), "\"gemini-1.5-flash-exp-0827\"");

        let model = Models::Gemini15Flash8bExp0827;
        assert_eq!(model.to_string(), "\"gemini-1.5-flash-8b-exp-0827\"");

        let model = Models::Gemini15Pro;
        assert_eq!(model.to_string(), "\"gemini-1.5-pro\"");

        let model = Models::Gemini15Flash;
        assert_eq!(model.to_string(), "\"gemini-1.5-flash\"");

        let model = Models::Gemini10Pro;
        assert_eq!(model.to_string(), "\"gemini-1.0-pro\"");

        let model = Models::Gemma2_2bIt;
        assert_eq!(model.to_string(), "\"gemma-2-2b-it\"");

        let model = Models::Gemma2_9bIt;
        assert_eq!(model.to_string(), "\"gemma-2-9b-it\"");

        let model = Models::Gemma2_27bIt;
        assert_eq!(model.to_string(), "\"gemma-2-27b-it\"");
    }
}
