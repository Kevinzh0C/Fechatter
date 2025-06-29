use ai_sdk::{OpenaiAdapter, AiService, Message, Role};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🧪 AI SDK Basic Test");
    println!("===================");
    
    // Test basic functionality without OpenAI API calls
    test_message_creation();
    
    // Only test API calls if API key is available
    if std::env::var("OPENAI_API_KEY").is_ok() {
        test_openai_integration().await?;
    } else {
        println!("WARNING: OPENAI_API_KEY not set, skipping API tests");
    }
    
    Ok(())
}

fn test_message_creation() {
    println!("\nTesting message creation...");
    
    let system_msg = Message::system("You are helpful");
    let user_msg = Message::user("Hello");
    let assistant_msg = Message::assistant("Hi there!");
    
    println!("Created system message: {:?}", system_msg.content);
    println!("Created user message: {:?}", user_msg.content);
    println!("Created assistant message: {:?}", assistant_msg.content);
}

async fn test_openai_integration() -> anyhow::Result<()> {
    println!("\n🤖 Testing OpenAI integration...");
    
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let adapter = OpenaiAdapter::new(api_key, "gpt-4o-mini");
    
    // Test chat completion
    let messages = vec![
        Message::system("You are a helpful assistant. Respond with exactly one word."),
        Message::user("Say 'Hello'"),
    ];
    
    println!("🔄 Testing chat completion...");
    match adapter.complete(&messages).await {
        Ok(response) => {
            println!("Chat completion: {}", response.trim());
        },
        Err(e) => {
            println!("ERROR: Chat completion failed: {}", e);
        }
    }
    
    // Test embedding generation
    println!("🔄 Testing embedding generation...");
    match adapter.generate_embedding("test").await {
        Ok(embedding) => {
            println!("Embedding generated: {} dimensions", embedding.len());
        },
        Err(e) => {
            println!("ERROR: Embedding generation failed: {}", e);
        }
    }
    
    // Test content moderation
    println!("🔄 Testing content moderation...");
    match adapter.moderate_content("This is a normal message").await {
        Ok(is_safe) => {
            println!("Content moderation: safe = {}", is_safe);
        },
        Err(e) => {
            println!("ERROR: Content moderation failed: {}", e);
        }
    }
    
    // Test summary generation
    println!("🔄 Testing summary generation...");
    match adapter.generate_summary("This is a long text that needs to be summarized for testing purposes.").await {
        Ok(summary) => {
            println!("Summary generated: {}", summary.trim());
        },
        Err(e) => {
            println!("ERROR: Summary generation failed: {}", e);
        }
    }
    
    Ok(())
}