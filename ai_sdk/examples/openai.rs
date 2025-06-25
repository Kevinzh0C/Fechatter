use ai_sdk::*;
use std::env;

#[tokio::main]
async fn main() {
  let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY environment variable must be set");
  let adapter = OpenaiAdapter::new(api_key, "gpt-4o");
  let messages = vec![Message {
    role: Role::User,
    content: "世界上最长的河流是什么？".to_string(),
  }];
  
  match adapter.complete(&messages).await {
    Ok(response) => {
      println!("OpenAI Response: {}", response);
    },
    Err(e) => {
      println!("Error calling OpenAI API: {}", e);
      println!("Please check your API key and quota limits at https://platform.openai.com/account/usage");
    }
  }
}
