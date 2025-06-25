use ai_sdk::{AiService, Message, OllamaAdapter, OpenaiAdapter, Role};
use std::env;

#[tokio::main]
async fn main() {
    println!("🤖 AI SDK Bot Test");
    println!("==================");

    // Test OpenAI if API key is available
    if let Ok(api_key) = env::var("OPENAI_API_KEY") {
        if !api_key.is_empty() {
            println!("\n🧠 Testing OpenAI...");
            test_openai(api_key).await;
        } else {
            println!("\n⚠️  OPENAI_API_KEY is empty, skipping OpenAI test");
        }
    } else {
        println!("\n⚠️  OPENAI_API_KEY not set, skipping OpenAI test");
    }

    // Test Ollama (local)
    println!("\n🦙 Testing Ollama...");
    test_ollama().await;

    println!("\n✅ AI SDK Bot Test Complete");
}

async fn test_openai(api_key: String) {
    let adapter = OpenaiAdapter::new(api_key, "gpt-4o-mini");
    let messages = vec![
        Message::system("You are a helpful assistant. Give very short responses."),
        Message::user("Say hello in one word."),
    ];

    match adapter.complete(&messages).await {
        Ok(response) => {
            println!("✅ OpenAI Response: {}", response.trim());
        }
        Err(e) => {
            println!("❌ OpenAI Error: {}", e);
        }
    }
}

async fn test_ollama() {
    let adapter = OllamaAdapter::default();
    let messages = vec![
        Message::system("You are a helpful assistant. Give very short responses."),
        Message::user("Say hello in one word."),
    ];

    match adapter.complete(&messages).await {
        Ok(response) => {
            println!("✅ Ollama Response: {}", response.trim());
        }
        Err(e) => {
            println!("❌ Ollama Error: {}", e);
            println!("💡 Make sure Ollama is running: ollama serve");
            println!("💡 And model is available: ollama pull llama3.2");
        }
    }
} 