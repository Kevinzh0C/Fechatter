use anyhow::Result;
use fechatter_server::services::ai::{AiServiceAdapter, OpenaiAdapter, AiMessage, AiRole};
use fechatter_core::contracts::infrastructure::{AIService, ChatMessage};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤖 AI Integration Example");
    println!("========================");
    
    // Example 1: Using ai_sdk directly for basic operations
    basic_ai_operations().await?;
    
    // Example 2: Using the adapter for fechatter-specific operations
    fechatter_ai_operations().await?;
    
    Ok(())
}

async fn basic_ai_operations() -> Result<()> {
    println!("\n📋 Basic AI Operations (using ai_sdk directly)");
    println!("-----------------------------------------------");
    
    // Create OpenAI adapter (requires OPENAI_API_KEY environment variable)
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        let openai = OpenaiAdapter::new(api_key, "gpt-4o-mini");
        
        // Basic chat completion
        let messages = vec![
            AiMessage::system("You are a helpful assistant."),
            AiMessage::user("What is the capital of Japan?"),
        ];
        
        match openai.complete(&messages).await {
            Ok(response) => println!("Chat completion: {}", response),
            Err(e) => println!("ERROR: Chat completion failed: {}", e),
        }
        
        // Generate embedding
        match openai.generate_embedding("Hello world").await {
            Ok(embedding) => println!("Generated embedding with {} dimensions", embedding.len()),
            Err(e) => println!("ERROR: Embedding generation failed: {}", e),
        }
        
        // Content moderation
        match openai.moderate_content("This is a normal message").await {
            Ok(is_safe) => println!("Content moderation: safe = {}", is_safe),
            Err(e) => println!("ERROR: Content moderation failed: {}", e),
        }
    } else {
        println!("WARNING: OPENAI_API_KEY not found, skipping OpenAI examples");
    }
    
    Ok(())
}

async fn fechatter_ai_operations() -> Result<()> {
    println!("\nFechatter AI Operations (using adapter)");
    println!("------------------------------------------");
    
    if let Ok(_) = std::env::var("OPENAI_API_KEY") {
        // Create fechatter AI service adapter
        let ai_service = match AiServiceAdapter::from_env() {
            Ok(service) => service,
            Err(e) => {
                println!("ERROR: Failed to create AI service: {}", e);
                return Ok(());
            }
        };
        
        // Chat completion using fechatter's interface
        let chat_messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful chat assistant for Fechatter.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "How do I create a new chat room?".to_string(),
            },
        ];
        
        match ai_service.chat_completion(chat_messages).await {
            Ok(response) => println!("Fechatter chat completion: {}", response),
            Err(e) => println!("ERROR: Fechatter chat completion failed: {}", e),
        }
        
        // Generate summary
        let long_text = "This is a very long conversation that happened in our chat room. Alice said hello, Bob responded with a greeting, Charlie joined and asked about the weather, and then they all discussed their weekend plans. It was a friendly conversation that lasted about 30 minutes.";
        
        match ai_service.generate_summary(long_text).await {
            Ok(summary) => println!("Summary: {}", summary),
            Err(e) => println!("ERROR: Summary generation failed: {}", e),
        }
        
        // Analyze sentiment
        match ai_service.analyze_sentiment("I love this new feature!").await {
            Ok(sentiment) => println!("Sentiment: {} (score: {})", sentiment.label, sentiment.score),
            Err(e) => println!("ERROR: Sentiment analysis failed: {}", e),
        }
        
        // Suggest replies
        let context = "User: Hey everyone! Just wanted to share that I got a promotion at work today!";
        match ai_service.suggest_replies(context).await {
            Ok(suggestions) => {
                println!("Reply suggestions:");
                for (i, suggestion) in suggestions.iter().enumerate() {
                    println!("   {}. {}", i + 1, suggestion);
                }
            },
            Err(e) => println!("ERROR: Reply suggestions failed: {}", e),
        }
        
        // Additional adapter methods (embedding and moderation)
        match ai_service.generate_embedding("test message").await {
            Ok(embedding) => println!("Adapter embedding with {} dimensions", embedding.len()),
            Err(e) => println!("ERROR: Adapter embedding failed: {}", e),
        }
        
        match ai_service.moderate_content("This is appropriate content").await {
            Ok(is_safe) => println!("Adapter moderation: safe = {}", is_safe),
            Err(e) => println!("ERROR: Adapter moderation failed: {}", e),
        }
    } else {
        println!("WARNING: OPENAI_API_KEY not found, skipping fechatter examples");
    }
    
    Ok(())
}

/// Example showing how to migrate from old OpenAI client to new adapter
#[allow(dead_code)]
async fn migration_example() -> Result<()> {
    println!("\n🔄 Migration Example");
    println!("--------------------");
    
    // OLD WAY (using fechatter_server::services::ai::openai::OpenAIClient directly)
    /*
    use fechatter_server::services::ai::openai::OpenAIClient;
    let old_client = OpenAIClient::from_env()?;
    let summary = old_client.generate_summary("some text").await?;
    */
    
    // NEW WAY (using ai_sdk through adapter)
    let ai_service = AiServiceAdapter::from_env()?;
    let summary = ai_service.generate_summary("some text").await?;
    
    println!("Migration complete: {}", summary);
    
    Ok(())
}