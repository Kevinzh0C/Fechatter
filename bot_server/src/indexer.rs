use anyhow::Result;
use bot_server::AppConfig;
use sqlx::postgres::PgPoolOptions;
use swiftide::{
  indexing::{
    self,
    loaders::FileLoader,
    transformers::{ChunkCode, Embed, MetadataQACode},
  },
  integrations,
};
use swiftide_pgvector::PgVector;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
  let layer = Layer::new().with_filter(LevelFilter::INFO);
  tracing_subscriber::registry().with(layer).init();

  let config = AppConfig::load()?;
  info!("✅ {}", config.get_summary());

  let db_url = &config.server.db_url;
  let vector_size = config.bot.vector.size;

  let pool = PgPoolOptions::new().connect(db_url).await?;

  let client = integrations::openai::OpenAI::builder()
    .default_embed_model(&config.bot.openai.embed_model)
    .default_prompt_model(&config.bot.openai.model)
    .build()?;

  let store = PgVector::try_new(pool, vector_size as _).await?;

  info!(
    "🚀 Starting code indexing with vector size: {}",
    vector_size
  );

  indexing::Pipeline::from_loader(FileLoader::new(".").with_extensions(&["rs"]))
    .then(MetadataQACode::new(client.clone()))
    .then_chunk(ChunkCode::try_for_language_and_chunk_size(
      "rust",
      10..2048,
    )?)
    .then_in_batch(Embed::new(client).with_batch_size(10))
    .then_store_with(store)
    .run()
    .await?;

  info!("✅ Code indexing completed successfully");
  Ok(())
}
