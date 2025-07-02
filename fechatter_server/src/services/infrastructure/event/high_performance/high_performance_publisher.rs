// High-Performance NATS Event Publisher with Zero-Cost Abstractions
//
// This module implements a production-ready, high-performance event publishing system
// using idiomatic Rust patterns, tokio mspc channels, and zero-cost abstractions.

use async_trait::async_trait;
use bytes::Bytes;
use futures::{stream::FuturesUnordered, StreamExt};
use std::{
    collections::HashMap,
    fmt,
    pin::Pin,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::{mpsc, oneshot, RwLock, Semaphore},
    time::{sleep, timeout},
};
use tracing::{debug, error, info, instrument, warn};

use crate::error::{AppError, EventTransportError};
use serde::{Deserialize, Serialize};
use uuid;

// =====================================================================================
// CORE TRAITS AND ZERO-COST ABSTRACTIONS
// =====================================================================================

/// Zero-cost abstraction for serializable events
pub trait EventData: Send + Sync + 'static {
    /// Get the subject for this event
    fn subject(&self) -> &str;

    /// Serialize to bytes with zero-copy optimization where possible
    fn serialize(&self) -> Result<Bytes, AppError>;

    /// Get event metadata for tracing and monitoring
    fn metadata(&self) -> EventMetadata;

    /// Get priority for backpressure handling
    fn priority(&self) -> EventPriority {
        EventPriority::Normal
    }
}

/// Event metadata for monitoring and debugging
#[derive(Debug, Clone)]
pub struct EventMetadata {
    pub event_type: String,
    pub event_id: String,
    pub trace_id: Option<String>,
    pub created_at: Instant,
}

/// Event priority for backpressure management
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Publishing result with detailed metrics
#[derive(Debug)]
pub struct PublishResult {
    pub success: bool,
    pub latency: Duration,
    pub retry_count: u32,
    pub error: Option<EventTransportError>,
}

// =====================================================================================
// HIGH-PERFORMANCE PUBLISHER CONFIGURATION
// =====================================================================================

/// Publisher configuration optimized for different workload patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherConfig {
    /// Buffer size for the internal channel (affects memory usage)
    pub buffer_size: usize,

    /// Maximum number of concurrent publish operations
    pub max_concurrent: usize,

    /// Batch size for bulk operations
    pub batch_size: usize,

    /// Maximum time to wait before sending a partial batch
    #[serde(with = "duration_serde")]
    pub batch_timeout: Duration,

    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,

    /// Retry configuration
    pub retry: RetryConfig,

    /// Backpressure configuration
    pub backpressure: BackpressureConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    #[serde(with = "duration_serde")]
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    #[serde(with = "duration_serde")]
    pub initial_backoff: Duration,
    #[serde(with = "duration_serde")]
    pub max_backoff: Duration,
    pub jitter: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureConfig {
    pub enabled: bool,
    pub high_water_mark: usize,
    pub low_water_mark: usize,
    pub shed_probability: f64,
}

// Helper module for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

impl Default for PublisherConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10_000,
            max_concurrent: 100,
            batch_size: 50,
            batch_timeout: Duration::from_millis(10),
            circuit_breaker: CircuitBreakerConfig {
                failure_threshold: 5,
                success_threshold: 3,
                timeout: Duration::from_secs(30),
            },
            retry: RetryConfig {
                max_retries: 3,
                initial_backoff: Duration::from_millis(50),
                max_backoff: Duration::from_secs(5),
                jitter: true,
            },
            backpressure: BackpressureConfig {
                enabled: true,
                high_water_mark: 8_000,
                low_water_mark: 2_000,
                shed_probability: 0.1,
            },
        }
    }
}

// =====================================================================================
// CIRCUIT BREAKER IMPLEMENTATION
// =====================================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure_time: Arc::new(RwLock::new(None)),
            config,
        }
    }

    async fn is_request_allowed(&self) -> bool {
        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() > self.config.timeout {
                        // Try to transition to half-open
                        let mut state_guard = self.state.write().await;
                        if matches!(*state_guard, CircuitState::Open) {
                            *state_guard = CircuitState::HalfOpen;
                            self.success_count.store(0, Ordering::SeqCst);
                            return true;
                        }
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    async fn record_success(&self) {
        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::HalfOpen => {
                let success_count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if success_count >= self.config.success_threshold {
                    let mut state_guard = self.state.write().await;
                    *state_guard = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                }
            }
            CircuitState::Open => {}
        }
    }

    async fn record_failure(&self) {
        let failure_count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;

        if failure_count >= self.config.failure_threshold {
            let mut state_guard = self.state.write().await;
            *state_guard = CircuitState::Open;
            *self.last_failure_time.write().await = Some(Instant::now());
        }
    }
}

// =====================================================================================
// PUBLISH REQUEST INTERNAL TYPES
// =====================================================================================

struct PublishRequest {
    event_data: Box<dyn EventData>,
    response_tx: oneshot::Sender<PublishResult>,
    enqueued_at: Instant,
}

struct BatchPublishRequest {
    events: Vec<Box<dyn EventData>>,
    response_tx: oneshot::Sender<Vec<PublishResult>>,
    enqueued_at: Instant,
}

enum InternalRequest {
    Single(PublishRequest),
    Batch(BatchPublishRequest),
    Shutdown,
}

// =====================================================================================
// METRICS AND MONITORING
// =====================================================================================

#[derive(Debug, Default)]
pub struct PublisherMetrics {
    pub total_published: AtomicU64,
    pub total_failed: AtomicU64,
    pub total_retries: AtomicU64,
    pub total_circuit_breaks: AtomicU64,
    pub total_backpressure_drops: AtomicU64,
    pub queue_size: AtomicUsize,
    pub average_latency_us: AtomicU64,
    pub batch_count: AtomicU64,
    pub avg_batch_size: AtomicU64,
}

impl PublisherMetrics {
    pub fn record_publish_success(&self, latency: Duration) {
        self.total_published.fetch_add(1, Ordering::Relaxed);

        // Update rolling average latency
        let latency_us = latency.as_micros() as u64;
        let current_avg = self.average_latency_us.load(Ordering::Relaxed);
        let total_published = self.total_published.load(Ordering::Relaxed);

        if total_published > 1 {
            let new_avg = (current_avg * (total_published - 1) + latency_us) / total_published;
            self.average_latency_us.store(new_avg, Ordering::Relaxed);
        } else {
            self.average_latency_us.store(latency_us, Ordering::Relaxed);
        }
    }

    pub fn record_publish_failure(&self) {
        self.total_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_retry(&self) {
        self.total_retries.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_circuit_break(&self) {
        self.total_circuit_breaks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_backpressure_drop(&self) {
        self.total_backpressure_drops
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn update_queue_size(&self, size: usize) {
        self.queue_size.store(size, Ordering::Relaxed);
    }

    pub fn record_batch(&self, batch_size: usize) {
        self.batch_count.fetch_add(1, Ordering::Relaxed);

        let current_avg = self.avg_batch_size.load(Ordering::Relaxed);
        let total_batches = self.batch_count.load(Ordering::Relaxed);

        if total_batches > 1 {
            let new_avg = (current_avg * (total_batches - 1) + batch_size as u64) / total_batches;
            self.avg_batch_size.store(new_avg, Ordering::Relaxed);
        } else {
            self.avg_batch_size
                .store(batch_size as u64, Ordering::Relaxed);
        }
    }
}

// =====================================================================================
// HIGH-PERFORMANCE PUBLISHER IMPLEMENTATION
// =====================================================================================

/// High-performance, production-ready NATS event publisher
pub struct HighPerformancePublisher {
    request_tx: mpsc::Sender<InternalRequest>,
    metrics: Arc<PublisherMetrics>,
    config: PublisherConfig,
}

impl HighPerformancePublisher {
    /// Create a new high-performance publisher
    pub fn new(nats_client: async_nats::Client, config: PublisherConfig) -> Result<Self, AppError> {
        let (request_tx, request_rx) = mpsc::channel(config.buffer_size);
        let metrics = Arc::new(PublisherMetrics::default());

        // Start the background processing task
        let processor =
            PublisherProcessor::new(nats_client, config.clone(), metrics.clone(), request_rx);

        tokio::spawn(async move {
            if let Err(e) = processor.run().await {
                error!("Publisher processor failed: {}", e);
            }
        });

        Ok(Self {
            request_tx,
            metrics,
            config,
        })
    }

    /// Publish a single event with zero-copy optimization
    #[instrument(skip(self, event), fields(subject = %event.subject(), priority = ?event.priority()))]
    pub async fn publish<E>(&self, event: E) -> Result<PublishResult, AppError>
    where
        E: EventData + 'static,
    {
        // Check backpressure
        if self.should_shed_load().await {
            self.metrics.record_backpressure_drop();
            return Ok(PublishResult {
                success: false,
                latency: Duration::ZERO,
                retry_count: 0,
                error: Some(EventTransportError::Publish(
                    "Load shedding activated".to_string(),
                )),
            });
        }

        let (response_tx, response_rx) = oneshot::channel();
        let request = PublishRequest {
            event_data: Box::new(event),
            response_tx,
            enqueued_at: Instant::now(),
        };

        // Send with timeout to prevent blocking on full queue
        match timeout(
            Duration::from_millis(100),
            self.request_tx.send(InternalRequest::Single(request)),
        )
        .await
        {
            Ok(Ok(())) => response_rx.await.map_err(|_| {
                AppError::EventPublishError("Publisher processor disconnected".to_string())
            }),
            Ok(Err(_)) => Err(AppError::EventPublishError(
                "Request queue full".to_string(),
            )),
            Err(_) => Err(AppError::EventPublishError("Request timeout".to_string())),
        }
    }

    /// Publish multiple events in a batch with automatic batching optimization
    #[instrument(skip(self, events), fields(batch_size = events.len()))]
    pub async fn publish_batch<E>(&self, events: Vec<E>) -> Result<Vec<PublishResult>, AppError>
    where
        E: EventData + 'static,
    {
        if events.is_empty() {
            return Ok(vec![]);
        }

        // Check backpressure
        if self.should_shed_load().await {
            self.metrics.record_backpressure_drop();
            return Ok(events
                .into_iter()
                .map(|_| PublishResult {
                    success: false,
                    latency: Duration::ZERO,
                    retry_count: 0,
                    error: Some(EventTransportError::Publish(
                        "Load shedding activated".to_string(),
                    )),
                })
                .collect());
        }

        let (response_tx, response_rx) = oneshot::channel();
        let request = BatchPublishRequest {
            events: events
                .into_iter()
                .map(|e| Box::new(e) as Box<dyn EventData>)
                .collect(),
            response_tx,
            enqueued_at: Instant::now(),
        };

        match timeout(
            Duration::from_millis(500), // Longer timeout for batch
            self.request_tx.send(InternalRequest::Batch(request)),
        )
        .await
        {
            Ok(Ok(())) => response_rx.await.map_err(|_| {
                AppError::EventPublishError("Publisher processor disconnected".to_string())
            }),
            Ok(Err(_)) => Err(AppError::EventPublishError(
                "Request queue full".to_string(),
            )),
            Err(_) => Err(AppError::EventPublishError("Request timeout".to_string())),
        }
    }

    /// Get current metrics snapshot
    pub fn metrics(&self) -> &PublisherMetrics {
        &self.metrics
    }

    /// Check if we should shed load based on backpressure configuration
    async fn should_shed_load(&self) -> bool {
        if !self.config.backpressure.enabled {
            return false;
        }

        let queue_size = self.metrics.queue_size.load(Ordering::Relaxed);

        if queue_size >= self.config.backpressure.high_water_mark {
            // Use probabilistic load shedding
            use rand::Rng;
            let mut rng = rand::thread_rng();
            rng.gen::<f64>() < self.config.backpressure.shed_probability
        } else {
            false
        }
    }

    /// Graceful shutdown
    pub async fn shutdown(&self) -> Result<(), AppError> {
        self.request_tx
            .send(InternalRequest::Shutdown)
            .await
            .map_err(|_| AppError::EventPublishError("Failed to send shutdown signal".to_string()))
    }
}

// =====================================================================================
// BACKGROUND PROCESSOR IMPLEMENTATION
// =====================================================================================

struct PublisherProcessor {
    nats_client: async_nats::Client,
    config: PublisherConfig,
    metrics: Arc<PublisherMetrics>,
    request_rx: mpsc::Receiver<InternalRequest>,
    circuit_breaker: CircuitBreaker,
    semaphore: Arc<Semaphore>,
}

impl PublisherProcessor {
    fn new(
        nats_client: async_nats::Client,
        config: PublisherConfig,
        metrics: Arc<PublisherMetrics>,
        request_rx: mpsc::Receiver<InternalRequest>,
    ) -> Self {
        let circuit_breaker = CircuitBreaker::new(config.circuit_breaker.clone());
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent));

        Self {
            nats_client,
            config,
            metrics,
            request_rx,
            circuit_breaker,
            semaphore,
        }
    }

    async fn run(mut self) -> Result<(), AppError> {
        let mut batch_buffer = Vec::with_capacity(self.config.batch_size);
        let mut batch_deadline = None::<Pin<Box<tokio::time::Sleep>>>;

        loop {
            self.metrics.update_queue_size(self.request_rx.len());

            tokio::select! {
                // Handle incoming requests
                request = self.request_rx.recv() => {
                    match request {
                        Some(InternalRequest::Single(req)) => {
                            if batch_buffer.len() >= self.config.batch_size {
                                self.flush_batch(&mut batch_buffer, &mut batch_deadline).await;
                            }

                            batch_buffer.push(req);

                            if batch_deadline.is_none() {
                                batch_deadline = Some(Box::pin(sleep(self.config.batch_timeout)));
                            }
                        }
                        Some(InternalRequest::Batch(req)) => {
                            // Flush any pending batch first
                            if !batch_buffer.is_empty() {
                                self.flush_batch(&mut batch_buffer, &mut batch_deadline).await;
                            }

                            // Process batch request immediately
                            self.handle_batch_request(req).await;
                        }
                        Some(InternalRequest::Shutdown) => {
                            // Flush any remaining events
                            if !batch_buffer.is_empty() {
                                self.flush_batch(&mut batch_buffer, &mut batch_deadline).await;
                            }
                            break;
                        }
                        None => break, // Channel closed
                    }
                }

                // Handle batch timeout
                _ = async {
                    if let Some(deadline) = &mut batch_deadline {
                        deadline.as_mut().await;
                    } else {
                        futures::future::pending::<()>().await;
                    }
                } => {
                    if !batch_buffer.is_empty() {
                        self.flush_batch(&mut batch_buffer, &mut batch_deadline).await;
                    }
                }
            }
        }

        Ok(())
    }

    async fn flush_batch(
        &self,
        batch_buffer: &mut Vec<PublishRequest>,
        batch_deadline: &mut Option<Pin<Box<tokio::time::Sleep>>>,
    ) {
        if batch_buffer.is_empty() {
            return;
        }

        let batch_size = batch_buffer.len();
        let batch = batch_buffer.drain(..).collect::<Vec<_>>();
        *batch_deadline = None;

        self.metrics.record_batch(batch_size);

        // Process batch concurrently with semaphore for backpressure
        let mut futures = FuturesUnordered::new();

        for request in batch {
            let permit = self.semaphore.clone().acquire_owned().await;
            if let Ok(permit) = permit {
                let circuit_breaker = &self.circuit_breaker;
                let nats_client = &self.nats_client;
                let config = &self.config;
                let metrics = &self.metrics;

                futures.push(async move {
                    let result = Self::publish_single_with_retry(
                        nats_client,
                        config,
                        circuit_breaker,
                        request.event_data,
                        request.enqueued_at,
                    )
                    .await;

                    if result.success {
                        metrics.record_publish_success(result.latency);
                    } else {
                        metrics.record_publish_failure();
                    }

                    let _ = request.response_tx.send(result);
                    drop(permit); // Release semaphore
                });
            } else {
                // Semaphore acquisition failed, return error
                let result = PublishResult {
                    success: false,
                    latency: Duration::ZERO,
                    retry_count: 0,
                    error: Some(EventTransportError::Publish(
                        "Semaphore acquisition failed".to_string(),
                    )),
                };
                let _ = request.response_tx.send(result);
            }
        }

        // Wait for all publishes to complete
        while futures.next().await.is_some() {}
    }

    async fn handle_batch_request(&self, request: BatchPublishRequest) {
        let batch_size = request.events.len();
        let mut results = Vec::with_capacity(batch_size);

        self.metrics.record_batch(batch_size);

        // Process all events concurrently with semaphore
        let mut futures = FuturesUnordered::new();

        for event_data in request.events {
            let permit = self.semaphore.clone().acquire_owned().await;
            if let Ok(permit) = permit {
                let circuit_breaker = &self.circuit_breaker;
                let nats_client = &self.nats_client;
                let config = &self.config;
                let metrics = &self.metrics;
                let enqueued_at = request.enqueued_at;

                futures.push(async move {
                    let result = Self::publish_single_with_retry(
                        nats_client,
                        config,
                        circuit_breaker,
                        event_data,
                        enqueued_at,
                    )
                    .await;

                    if result.success {
                        metrics.record_publish_success(result.latency);
                    } else {
                        metrics.record_publish_failure();
                    }

                    drop(permit);
                    result
                });
            } else {
                results.push(PublishResult {
                    success: false,
                    latency: Duration::ZERO,
                    retry_count: 0,
                    error: Some(EventTransportError::Publish(
                        "Semaphore acquisition failed".to_string(),
                    )),
                });
            }
        }

        // Collect all results
        while let Some(result) = futures.next().await {
            results.push(result);
        }

        let _ = request.response_tx.send(results);
    }

    async fn publish_single_with_retry(
        nats_client: &async_nats::Client,
        config: &PublisherConfig,
        circuit_breaker: &CircuitBreaker,
        event_data: Box<dyn EventData>,
        enqueued_at: Instant,
    ) -> PublishResult {
        let start_time = Instant::now();
        let total_latency = start_time.duration_since(enqueued_at);

        // Check circuit breaker
        if !circuit_breaker.is_request_allowed().await {
            return PublishResult {
                success: false,
                latency: total_latency,
                retry_count: 0,
                error: Some(EventTransportError::Publish(
                    "Circuit breaker open".to_string(),
                )),
            };
        }

        // Serialize event data once
        let payload = match event_data.serialize() {
            Ok(data) => data,
            Err(e) => {
                return PublishResult {
                    success: false,
                    latency: total_latency,
                    retry_count: 0,
                    error: Some(EventTransportError::Publish(format!(
                        "Serialization failed: {}",
                        e
                    ))),
                };
            }
        };

        let subject = event_data.subject();
        let mut retry_count = 0;
        let mut last_error = None;

        // Retry loop with exponential backoff
        while retry_count <= config.retry.max_retries {
            match Self::try_publish_once(nats_client, subject, &payload).await {
                Ok(()) => {
                    circuit_breaker.record_success().await;
                    return PublishResult {
                        success: true,
                        latency: start_time.elapsed(),
                        retry_count,
                        error: None,
                    };
                }
                Err(e) => {
                    last_error = Some(e);
                    retry_count += 1;

                    if retry_count <= config.retry.max_retries {
                        let backoff = Self::calculate_backoff(&config.retry, retry_count);
                        sleep(backoff).await;
                    }
                }
            }
        }

        // All retries failed
        circuit_breaker.record_failure().await;
        PublishResult {
            success: false,
            latency: start_time.elapsed(),
            retry_count,
            error: last_error,
        }
    }

    async fn try_publish_once(
        nats_client: &async_nats::Client,
        subject: &str,
        payload: &Bytes,
    ) -> Result<(), EventTransportError> {
        nats_client
            .publish(subject.to_string(), payload.clone())
            .await
            .map_err(|e| EventTransportError::Publish(e.to_string()))
    }

    fn calculate_backoff(config: &RetryConfig, attempt: u32) -> Duration {
        let base = config.initial_backoff.as_millis() as u64;
        let exponential = base * 2u64.pow(attempt.saturating_sub(1));
        let capped = exponential.min(config.max_backoff.as_millis() as u64);

        let final_backoff = if config.jitter {
            use rand::Rng;
            let jitter = rand::thread_rng().gen_range(0.8..=1.2);
            ((capped as f64) * jitter) as u64
        } else {
            capped
        };

        Duration::from_millis(final_backoff)
    }
}

impl fmt::Debug for PublisherProcessor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PublisherProcessor")
            .field("config", &self.config)
            .finish()
    }
}

// =====================================================================================
// CONCRETE IMPLEMENTATIONS FOR COMMON EVENTS
// =====================================================================================

/// High-performance message event implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastMessageEvent {
    pub message_id: u64,
    pub chat_id: u64,
    pub user_id: u64,
    pub content: String,
    pub event_type: String,
    pub timestamp: u64,
    pub workspace_id: Option<u64>,
}

impl EventData for FastMessageEvent {
    fn subject(&self) -> &str {
        match self.event_type.as_str() {
            "created" => "fechatter.messages.created.v2",
            "updated" => "fechatter.messages.updated.v2",
            "deleted" => "fechatter.messages.deleted.v2",
            _ => "fechatter.messages.unknown.v2",
        }
    }

    fn serialize(&self) -> Result<Bytes, AppError> {
        // Use efficient binary serialization for performance
        let data =
            serde_json::to_vec(self).map_err(|e| AppError::SerializationError(e.to_string()))?;
        Ok(Bytes::from(data))
    }

    fn metadata(&self) -> EventMetadata {
        EventMetadata {
            event_type: format!("message.{}", self.event_type),
            event_id: format!("msg_{}_{}", self.message_id, self.timestamp),
            trace_id: None,
            created_at: Instant::now(),
        }
    }

    fn priority(&self) -> EventPriority {
        match self.event_type.as_str() {
            "deleted" => EventPriority::High,
            _ => EventPriority::Normal,
        }
    }
}

/// High-performance chat member event implementation  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastChatMemberEvent {
    pub chat_id: u64,
    pub user_id: u64,
    pub event_type: String, // "joined" or "left"
    pub timestamp: u64,
    pub workspace_id: Option<u64>,
}

impl EventData for FastChatMemberEvent {
    fn subject(&self) -> &str {
        match self.event_type.as_str() {
            "joined" => "fechatter.chat.member.joined.v2",
            "left" => "fechatter.chat.member.left.v2",
            _ => "fechatter.chat.member.unknown.v2",
        }
    }

    fn serialize(&self) -> Result<Bytes, AppError> {
        let data =
            serde_json::to_vec(self).map_err(|e| AppError::SerializationError(e.to_string()))?;
        Ok(Bytes::from(data))
    }

    fn metadata(&self) -> EventMetadata {
        EventMetadata {
            event_type: format!("chat.member.{}", self.event_type),
            event_id: format!(
                "chat_{}_{}_{}_{}",
                self.chat_id, self.user_id, self.event_type, self.timestamp
            ),
            trace_id: None,
            created_at: Instant::now(),
        }
    }

    fn priority(&self) -> EventPriority {
        EventPriority::Normal
    }
}

use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_publisher_config_defaults() {
        let config = PublisherConfig::default();
        assert_eq!(config.buffer_size, 10_000);
        assert_eq!(config.max_concurrent, 100);
        assert_eq!(config.batch_size, 50);
    }

    #[tokio::test]
    async fn test_circuit_breaker_state_transitions() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_millis(100),
        };
        let cb = CircuitBreaker::new(config);

        // Initially closed
        assert!(cb.is_request_allowed().await);

        // Record failures
        cb.record_failure().await;
        assert!(cb.is_request_allowed().await);

        cb.record_failure().await;
        assert!(!cb.is_request_allowed().await); // Now open

        // Wait for timeout and verify half-open
        sleep(Duration::from_millis(150)).await;
        assert!(cb.is_request_allowed().await); // Should be half-open

        // Record success to close
        cb.record_success().await;
        assert!(cb.is_request_allowed().await); // Should be closed
    }

    #[test]
    fn test_fast_message_event_serialization() {
        let event = FastMessageEvent {
            message_id: 123,
            chat_id: 456,
            user_id: 789,
            content: "Hello".to_string(),
            event_type: "created".to_string(),
            timestamp: 1234567890,
            workspace_id: Some(42),
        };

        // Use fully qualified method to resolve ambiguity between EventData::serialize and serde::Serialize
        let bytes = <FastMessageEvent as EventData>::serialize(&event).unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(event.subject(), "fechatter.messages.created.v2");
    }

    #[test]
    fn test_event_priority_ordering() {
        assert!(EventPriority::Critical > EventPriority::High);
        assert!(EventPriority::High > EventPriority::Normal);
        assert!(EventPriority::Normal > EventPriority::Low);
    }
}
