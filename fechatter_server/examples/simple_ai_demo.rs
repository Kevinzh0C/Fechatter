use anyhow::Result;
use fechatter_server::services::ai::{SimpleWorkflow, SimpleTopicExtractor, openai::OpenAIClient};
use chrono::Utc;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Simple AI Demo - Everything delegated to LLM");
    println!("===============================================");
    
    // Mock chat data
    let messages = vec![
        (1, 101, "Hey everyone! Let's discuss the new project roadmap".to_string(), Utc::now()),
        (2, 102, "Great idea! I think we should focus on user experience first".to_string(), Utc::now()),
        (3, 103, "Agreed! Also, we need to consider security aspects".to_string(), Utc::now()),
        (4, 101, "Security is definitely important. What about performance?".to_string(), Utc::now()),
        (5, 104, "Performance optimization should be in phase 2".to_string(), Utc::now()),
        (6, 102, "Should we schedule a meeting to discuss this further?".to_string(), Utc::now()),
        (7, 103, "Yes, let's meet tomorrow at 2 PM".to_string(), Utc::now()),
        (8, 101, "Perfect! I'll send out calendar invites".to_string(), Utc::now()),
    ];
    
    if std::env::var("OPENAI_API_KEY").is_ok() {
        demo_with_real_ai(messages).await?;
    } else {
        demo_without_api(messages).await?;
    }
    
    Ok(())
}

async fn demo_with_real_ai(messages: Vec<(i64, i64, String, chrono::DateTime<chrono::Utc>)>) -> Result<()> {
    println!("🤖 Running with real OpenAI API...");
    
    // Create OpenAI client
    let openai_client = Arc::new(OpenAIClient::from_env()?);
    
    // Demo 1: Simple topic extraction
    println!("\n📋 Topic Extraction Demo");
    println!("-------------------------");
    
    let topic_extractor = SimpleTopicExtractor::new(openai_client.clone());
    let message_data: Vec<(i64, String)> = messages.iter()
        .map(|(id, _, content, _)| (*id, content.clone()))
        .collect();
    
    match topic_extractor.extract_topics(&message_data, 3).await {
        Ok(topics) => {
            for topic in topics {
                println!("Topic: {} (Keywords: {:?}, Messages: {})", 
                    topic.name, topic.keywords, topic.message_ids.len());
            }
        },
        Err(e) => println!("ERROR: Topic extraction failed: {}", e),
    }
    
    // Demo 2: Complete workflow
    println!("\n🔄 Complete Workflow Demo");
    println!("--------------------------");
    
    let workflow = SimpleWorkflow::new(openai_client);
    
    match workflow.analyze_chat_complete(123, messages).await {
        Ok(result) => {
            println!("Chat Analysis Complete:");
            println!("   Summary: {}", result.summary);
            println!("   🏷️  Topics: {:?}", result.topics);
            println!("   😊 Sentiment: {} ({})", result.sentiment.overall_label, result.sentiment.overall_score);
            println!("   📅 Timeline Events: {}", result.timeline.len());
            
            for event in result.timeline {
                println!("      - {}: {}", event.title, event.description);
            }
        },
        Err(e) => println!("ERROR: Workflow failed: {}", e),
    }
    
    Ok(())
}

async fn demo_without_api(messages: Vec<(i64, i64, String, chrono::DateTime<chrono::Utc>)>) -> Result<()> {
    println!("WARNING: OPENAI_API_KEY not found - showing structure only");
    println!();
    
    println!("📋 Would extract topics from {} messages", messages.len());
    println!("🔄 Would run complete workflow analysis");
    println!("Would generate summary, topics, sentiment, and timeline");
    println!();
    
    println!("Key Features:");
    println!("   Topic extraction (LLM-powered)");
    println!("   Sentiment analysis (delegated to OpenAI)");  
    println!("   Timeline generation (simple + LLM enhancement)");
    println!("   Complete workflow orchestration");
    println!("   Fallback implementations (when LLM fails)");
    println!();
    
    println!("Design Philosophy:");
    println!("   - Delegate complex logic to LLM");
    println!("   - Keep local processing minimal");
    println!("   - Simple data structures");
    println!("   - Fast implementation");
    
    Ok(())
}