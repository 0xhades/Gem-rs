# Gem-rs

Gem-rs is a Rust library that serves as a wrapper around the Gemini API, providing support for streaming. This library is designed to facilitate interaction with the Gemini API, making it easier to integrate its functionalities into Rust projects. Future updates will include support for uploading files and images, as well as caching files to Gemini.

## TODO List

- **Improve Documentation**: Enhance the documentation with more examples and detailed explanations.
- **Error Logging**: Implement a comprehensive error logging system.
- **Concurrency Support [✅]**: Add support for concurrent data processing.
- **Optimization**: Few functions and data structres need to be optimized.
- **Upload Files and Images [✅]**: Add support for uploading files and images to Gemini.
- **Caching Files [✅]**: Implement file caching to Gemini.
- **More File Types**: Add support to more file types eg. gif, doc, docx, code files, etc.
- **APIs abnormalites**: DELETE "files/x" dosen't delete the cloud cache related to the API key, it only change the URI. 
- **API Key Env**: I'm a busy medical student, implement it yourself (JK, I have finals).

## Dependencies

To use the Gem-rs library, add the following dependencies to your `Cargo.toml` file:

```toml
[dependencies]
futures = "*"
tokio = { version = "*", features = ["full"] }
gem-rs = { path = "..." }
```

## Example Usage

Here's a basic example of how to use the Gem-rs library (Further examples in examples folder):

```rust
use futures::stream::StreamExt;
use gem_rs::api::Models;
use gem_rs::client::{GemSession, GemSessionBuilder};
use gem_rs::init_log;
use gem_rs::types::{Context, HarmBlockThreshold, Settings};

const API_KEY: &str = "مفتاحك هنا ياحلو";

#[tokio::main]
async fn main() {
    init_log();
    test().await;
    test_stream().await;
    test_file().await;
}

async fn test_file() {
    let mut session = GemSession::Builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(30))
        .model(Models::Gemini15Flash)
        .context(Context::new())
        .build(API_KEY.to_string());

    let mut settings = Settings::new();
    settings.set_all_safety_settings(HarmBlockThreshold::BlockNone);
    settings.set_system_instruction("Summraize the PDFs that I send to you, in a (UwU) style");
    settings.set_max_output_tokens(8192);
    settings.set_temperature(1.5);

    let mut file_manager = FileManager::new(API_KEY);
    file_manager.fetch_list().await.unwrap();
    let data = file_manager
        .add_file(Path::new("C:/Users/0xhades/Downloads/9.pdf"))
        .await
        .unwrap();

    let response = session.send_file(data, &settings).await;

    match response {
        Ok(response) => {
            println!("Response: {:#?}", response.get_results());
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

async fn test_stream() {
    let mut session = GemSession::Builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(30))
        .model(Models::Gemini15Flash)
        .context(Context::new())
        .build(API_KEY.to_string());

    let mut settings = Settings::new();
    settings.set_all_safety_settings(HarmBlockThreshold::BlockNone);

    let stream_result = session
        .send_message_stream("Hello! What is your name?", &settings)
        .await;

    match stream_result {
        Ok(mut stream) => {
            while let Some(response) = stream.next().await {
                match response {
                    Ok(response) => {
                        println!(
                            "{}",
                            response.get_results().get(0).unwrap_or(&"".to_string())
                        );
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

async fn test() {
    let mut session = GemSession::Builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(30))
        .model(Models::Gemini15Flash)
        .context(Context::new())
        .build(API_KEY.to_string());

    let mut settings = Settings::new();
    settings.set_all_safety_settings(HarmBlockThreshold::BlockNone);

    let response = session
        .send_message("Hello! What is your name?", &settings)
        .await;

    match response {
        Ok(response) => {
            println!("Response: {:#?}", response.get_results());
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
```