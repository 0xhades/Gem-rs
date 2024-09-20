use std::path::Path;

use futures::stream::StreamExt;
use gem_rs::api::Models;
use gem_rs::client::{GemSession, GemSessionBuilder};
use gem_rs::init_log;
use gem_rs::types::{Blob, Context, FileManager, HarmBlockThreshold, Settings};

const API_KEY: &str = "X";

#[tokio::main]
async fn main() {
    init_log();
    test_clear_files().await;
}

//TODO: Something with the API cause the cached files in cloud to change uri every time they are deleted
async fn test_clear_files() {
    let mut file_manager = FileManager::new(API_KEY);
    file_manager.fetch_list().await.unwrap();
    file_manager.clear_files().await;
}

async fn test_blob() {
    let mut session = GemSession::Builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(30))
        .model(Models::Gemini15Flash)
        .context(Context::new())
        .build(API_KEY.to_string());

    let mut settings = Settings::new();
    settings.set_all_safety_settings(HarmBlockThreshold::BlockNone);
    settings.set_system_instruction("Explain any file that I send to you, in a (UwU) style");
    settings.set_max_output_tokens(8192);
    settings.set_temperature(1.5);

    let blob = Blob::new(
        "image/png",
        include_bytes!("C:/Users/0xhades/Downloads/x.png"),
    );

    let response = session.send_blob(blob, &settings).await;

    match response {
        Ok(response) => {
            println!("Response: {:#?}", response.get_results());
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

async fn test_blob_stream() {
    let mut session = GemSession::Builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(30))
        .model(Models::Gemini15Flash)
        .context(Context::new())
        .build(API_KEY.to_string());

    let mut settings = Settings::new();
    settings.set_all_safety_settings(HarmBlockThreshold::BlockNone);
    settings.set_system_instruction("Explain any file that I send to you, in a (UwU) style");
    settings.set_max_output_tokens(8192);
    settings.set_temperature(1.5);

    let blob = Blob::new(
        "image/png",
        include_bytes!("C:/Users/0xhades/Downloads/x.png"),
    );

    let stream_result = session.send_blob_stream(blob, &settings).await;

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

async fn test_file_stream() {
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

    let stream_result = session.send_file_stream(data, &settings).await;

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
