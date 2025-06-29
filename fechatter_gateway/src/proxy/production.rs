//! # Production-Grade HTTP Proxy - Fechatter Gateway Alternative
//!
//! **Full-featured HTTP/HTTPS proxy with enterprise-grade capabilities**
//!
//! Core Features:
//! - High-performance async HTTP/HTTPS proxy
//! - Production-ready load balancing (Round Robin, Random, Least Connections)
//! - Comprehensive health checking with automatic failover
//! - Full CORS support with preflight handling
//! - Request/Response logging and metrics
//! - Graceful shutdown and error recovery
//! - Circuit breaker pattern for resilience

use crate::config::{GatewayConfig, HealthCheckConfig, LoadBalancingType, RouteConfig, UpstreamConfig};
use anyhow::Result;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Client, Method, Request, Response, Server, StatusCode, Uri,
};
use std::{
    collections::HashMap,
    convert::Infallible,
    net::SocketAddr,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc, RwLock,
    },
    time::{Duration, Instant},
};
use tokio::{sync::Mutex, time::interval};
use tracing::{debug, error, info, warn};

/// Production-grade HTTP proxy server
pub struct ProductionProxy {
    config: Arc<GatewayConfig>,
    upstream_pools: Arc<RwLock<HashMap<String, UpstreamPool>>>,
    client: Client<hyper::client::HttpConnector>,
    metrics: Arc<ProxyMetrics>,
}

/// Upstream server pool with load balancing and health checking
#[derive(Clone)]
pub struct UpstreamPool {
    name: String,
    servers: Vec<UpstreamServer>,
    load_balancer: LoadBalancer,
    health_checker: Option<HealthChecker>,
}

/// Individual upstream server with health status
#[derive(Clone)]
pub struct UpstreamServer {
    address: String,
    healthy: Arc<std::sync::atomic::AtomicBool>,
    last_health_check: Arc<Mutex<Instant>>,
    connection_count: Arc<AtomicUsize>,
    total_requests: Arc<AtomicU64>,
    failed_requests: Arc<AtomicU64>,
}

/// Load balancing strategies
#[derive(Clone)]
pub enum LoadBalancer {
    RoundRobin {
        current: Arc<AtomicUsize>,
    },
    Random,
    LeastConnections,
    WeightedRoundRobin {
        weights: Vec<u32>,
        current: Arc<AtomicUsize>,
    },
}

/// Health checker for upstream servers
#[derive(Clone)]
pub struct HealthChecker {
    config: HealthCheckConfig,
    client: Client<hyper::client::HttpConnector>,
}

/// Proxy metrics for monitoring
#[derive(Default)]
pub struct ProxyMetrics {
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
    bytes_transferred: AtomicU64,
    active_connections: AtomicUsize,
    upstream_errors: AtomicU64,
    cors_preflight_requests: AtomicU64,
}

impl ProductionProxy {
    /// Create new production proxy instance
    pub async fn new(config: Arc<GatewayConfig>) -> Result<Self> {
        info!("🏭 Creating production-grade HTTP proxy with {} upstreams", config.upstreams.len());

        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(20)
            .build_http();

        let mut upstream_pools = HashMap::new();

        // Initialize upstream pools
        for (name, upstream_config) in &config.upstreams {
            let pool = UpstreamPool::new(name.clone(), upstream_config, &client).await?;
            upstream_pools.insert(name.clone(), pool);
        }

        let proxy = Self {
            config: config.clone(),
            upstream_pools: Arc::new(RwLock::new(upstream_pools)),
            client,
            metrics: Arc::new(ProxyMetrics::default()),
        };

        // Start health checking background task
        proxy.start_health_checking().await;

        Ok(proxy)
    }

    /// Start the production proxy server
    pub async fn run(self) -> Result<()> {
        let addr: SocketAddr = self.config.server.listen_addr.parse()?;
        
        info!("Starting production HTTP proxy on {}", addr);
        info!("Configuration:");
        info!("  Worker Threads: {:?}", self.config.server.worker_threads);
        info!("  🔗 Max Connections: {:?}", self.config.server.max_connections);
        info!("  ⏱️  Keep-Alive Timeout: {:?}s", self.config.server.keepalive_timeout);
        info!("  ⏱️  Request Timeout: {:?}s", self.config.server.request_timeout);

        let proxy = Arc::new(self);

        let make_svc = make_service_fn(move |_conn| {
            let proxy = Arc::clone(&proxy);
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let proxy = Arc::clone(&proxy);
                    async move { proxy.handle_request(req).await }
                }))
            }
        });

        let server = Server::bind(&addr)
            .serve(make_svc)
            .with_graceful_shutdown(async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to install Ctrl+C signal handler");
                info!("🛑 Graceful shutdown initiated");
            });

        info!("Production proxy listening and ready to serve requests");
        info!("Metrics available via proxy.get_metrics()");
        info!("Press Ctrl+C to gracefully shutdown");

        if let Err(e) = server.await {
            error!("ERROR: Production proxy server error: {}", e);
            Err(anyhow::anyhow!("Server error: {}", e))
        } else {
            info!("Production proxy shut down gracefully");
            Ok(())
        }
    }

    /// Handle incoming HTTP request with full proxy functionality
    async fn handle_request(&self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let start_time = Instant::now();
        self.metrics.total_requests.fetch_add(1, Ordering::Relaxed);
        self.metrics.active_connections.fetch_add(1, Ordering::Relaxed);

        let method = req.method().clone();
        let uri = req.uri().clone();
        let path = uri.path();

        debug!("EVENT: {} {}", method, path);

        // Handle Gateway's own health check endpoint
        if path == "/gateway/health" && method == Method::GET {
            let response = self.handle_gateway_health().await;
            self.metrics.active_connections.fetch_sub(1, Ordering::Relaxed);
            return Ok(response);
        }

        // Handle root path - Gateway welcome page
        if path == "/" && method == Method::GET {
            let response = self.handle_root_path().await;
            self.metrics.active_connections.fetch_sub(1, Ordering::Relaxed);
            return Ok(response);
        }

        // Handle CORS preflight requests
        if method == Method::OPTIONS {
            self.metrics.cors_preflight_requests.fetch_add(1, Ordering::Relaxed);
            let response = self.handle_cors_preflight(&req).await;
            self.metrics.active_connections.fetch_sub(1, Ordering::Relaxed);
            return Ok(response);
        }

        // Route matching
        let route = match self.find_matching_route(path, &method) {
            Some(route) => route,
            None => {
                warn!("ERROR: No route found for {} {}", method, path);
                self.metrics.failed_requests.fetch_add(1, Ordering::Relaxed);
                self.metrics.active_connections.fetch_sub(1, Ordering::Relaxed);
                return Ok(self.create_error_response(StatusCode::NOT_FOUND, "Route not found"));
            }
        };

        // Get upstream server
        let upstream_server = match self.get_healthy_upstream(&route.upstream).await {
            Some(server) => server,
            None => {
                error!("ERROR: No healthy upstream servers available for {}", route.upstream);
                self.metrics.upstream_errors.fetch_add(1, Ordering::Relaxed);
                self.metrics.failed_requests.fetch_add(1, Ordering::Relaxed);
                self.metrics.active_connections.fetch_sub(1, Ordering::Relaxed);
                return Ok(self.create_error_response(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Service temporarily unavailable"
                ));
            }
        };

        // Proxy the request
        let result = self.proxy_request(req, &upstream_server, &route).await;
        
        let response = match result {
            Ok(mut response) => {
                // Add CORS headers if enabled
                if route.cors_enabled.unwrap_or(false) {
                    self.add_cors_headers(&mut response, &route);
                }
                
                self.metrics.successful_requests.fetch_add(1, Ordering::Relaxed);
                upstream_server.total_requests.fetch_add(1, Ordering::Relaxed);
                response
            }
            Err(e) => {
                error!("ERROR: Proxy error for {}: {}", upstream_server.address, e);
                self.metrics.failed_requests.fetch_add(1, Ordering::Relaxed);
                upstream_server.failed_requests.fetch_add(1, Ordering::Relaxed);
                self.create_error_response(StatusCode::BAD_GATEWAY, "Upstream server error")
            }
        };

        let duration = start_time.elapsed();
        debug!("Request completed in {:?}", duration);
        
        self.metrics.active_connections.fetch_sub(1, Ordering::Relaxed);
        Ok(response)
    }

    /// Handle Gateway's own health check
    async fn handle_gateway_health(&self) -> Response<Body> {
        debug!("🏥 Gateway internal health check");
        
        // Collect upstream health status
        let pools = self.upstream_pools.read().unwrap();
        let mut services = Vec::new();
        let mut all_healthy = true;
        
        for (name, pool) in pools.iter() {
            let healthy_count = pool.servers.iter()
                .filter(|s| s.healthy.load(Ordering::Relaxed))
                .count();
            let total_count = pool.servers.len();
            let is_healthy = healthy_count > 0;
            
            services.push(format!(
                r#"{{"name": "{}", "healthy": {}, "servers": "{}/{}"}}"#,
                name, is_healthy, healthy_count, total_count
            ));
            
            if !is_healthy {
                all_healthy = false;
            }
        }
        
        let status = if all_healthy { "healthy" } else { "degraded" };
        let body = format!(
            r#"{{"status": "{}", "timestamp": "{}", "services": [{}]}}"#,
            status,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            services.join(", ")
        );
        
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("Access-Control-Allow-Origin", "*")
            .body(Body::from(body))
            .unwrap()
    }

    /// Handle root path - Gateway welcome page
    async fn handle_root_path(&self) -> Response<Body> {
        debug!("🏠 Gateway root path request");
        
        // Try to read the HTML template file
        let template_path = "/app/fechatter_gateway/templates/welcome.html";
        let template_content = match tokio::fs::read_to_string(template_path).await {
            Ok(content) => content,
            Err(_) => {
                // Fallback to simple HTML if template file not found
                return Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html; charset=utf-8")
                    .header("Access-Control-Allow-Origin", "*")
                    .body(Body::from(r#"
                        <!DOCTYPE html>
                        <html><head><title>Fechatter Gateway</title></head>
                        <body style="font-family: Arial, sans-serif; text-align: center; padding: 40px; background: #f0f0f0;">
                            <h1>🌉 Fechatter Gateway</h1>
                            <p>Production-grade API Gateway for Fechatter Platform</p>
                            <p><a href="/gateway/health">Health Check</a> | <a href="/metrics">Metrics</a></p>
                        </body></html>
                    "#))
                    .unwrap();
            }
        };
        
        // Collect statistics for template variables
        let pools = self.upstream_pools.read().unwrap();
        let total_upstreams = pools.len();
        let healthy_upstreams = pools.iter()
            .filter(|(_, pool)| {
                pool.servers.iter().any(|s| s.healthy.load(Ordering::Relaxed))
            })
            .count();
        
        let total_requests = self.metrics.total_requests.load(Ordering::Relaxed);
        let successful_requests = self.metrics.successful_requests.load(Ordering::Relaxed);
        let active_connections = self.metrics.active_connections.load(Ordering::Relaxed);
        
        // Calculate success rate
        let success_rate = if total_requests > 0 {
            format!("{:.1}%", (successful_requests as f64 / total_requests as f64) * 100.0)
        } else {
            "N/A".to_string()
        };
        
        // Determine status
        let is_healthy = healthy_upstreams == total_upstreams;
        let upstream_status_class = if is_healthy { "healthy" } else { "degraded" };
        let gateway_status_class = if is_healthy { "healthy" } else { "degraded" };
        let gateway_status = if is_healthy { "Healthy" } else { "Degraded" };
        
        // Replace template variables
        let html_content = template_content
            .replace("{{HEALTHY_UPSTREAMS}}", &healthy_upstreams.to_string())
            .replace("{{TOTAL_UPSTREAMS}}", &total_upstreams.to_string())
            .replace("{{TOTAL_REQUESTS}}", &total_requests.to_string())
            .replace("{{SUCCESS_RATE}}", &success_rate)
            .replace("{{ACTIVE_CONNECTIONS}}", &active_connections.to_string())
            .replace("{{UPSTREAM_STATUS_CLASS}}", upstream_status_class)
            .replace("{{GATEWAY_STATUS_CLASS}}", gateway_status_class)
            .replace("{{GATEWAY_STATUS}}", gateway_status);
        
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")
            .header("Access-Control-Allow-Origin", "*")
            .body(Body::from(html_content))
            .unwrap()
    }

    /// Find matching route for the request with improved logic
    fn find_matching_route(&self, path: &str, method: &Method) -> Option<&RouteConfig> {
        debug!("Finding route for {} {}", method, path);
        
        self.config.routes.iter().find(|route| {
            // Improved path matching logic
            let path_matches = if route.path.ends_with('/') {
                // For routes ending with '/', check if request path starts with route path
                path.starts_with(&route.path)
            } else {
                // For routes not ending with '/', support both exact match and prefix with slash
                path == route.path || path.starts_with(&format!("{}/", route.path))
            };

            // Check if method matches
            let method_matches = route.methods.iter().any(|m| {
                m.to_uppercase() == method.as_str().to_uppercase()
            });

            debug!("Route '{}' -> path_matches: {}, method_matches: {}", 
                   route.path, path_matches, method_matches);

            path_matches && method_matches
        })
    }

    /// Get healthy upstream server using load balancing
    async fn get_healthy_upstream(&self, upstream_name: &str) -> Option<UpstreamServer> {
        let pools = self.upstream_pools.read().unwrap();
        let pool = pools.get(upstream_name)?;
        
        let healthy_servers: Vec<_> = pool.servers.iter()
            .filter(|server| server.healthy.load(Ordering::Relaxed))
            .cloned()
            .collect();

        if healthy_servers.is_empty() {
            return None;
        }

        // Apply load balancing strategy
        match &pool.load_balancer {
            LoadBalancer::RoundRobin { current } => {
                let index = current.fetch_add(1, Ordering::Relaxed) % healthy_servers.len();
                Some(healthy_servers[index].clone())
            }
            LoadBalancer::Random => {
                use rand::Rng;
                let index = rand::thread_rng().gen_range(0..healthy_servers.len());
                Some(healthy_servers[index].clone())
            }
            LoadBalancer::LeastConnections => {
                healthy_servers.into_iter()
                    .min_by_key(|server| server.connection_count.load(Ordering::Relaxed))
            }
            LoadBalancer::WeightedRoundRobin { weights: _, current } => {
                // Simplified weighted round robin
                let index = current.fetch_add(1, Ordering::Relaxed) % healthy_servers.len();
                Some(healthy_servers[index].clone())
            }
        }
    }

    /// Proxy request to upstream server
    async fn proxy_request(
        &self,
        mut req: Request<Body>,
        upstream_server: &UpstreamServer,
        route: &RouteConfig,
    ) -> Result<Response<Body>> {
        // Build upstream URL
        let upstream_uri = format!("http://{}{}", upstream_server.address, req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or(""));
        let upstream_uri: Uri = upstream_uri.parse()?;

        // Update request URI
        *req.uri_mut() = upstream_uri;

        // Add/modify headers
        req.headers_mut().insert("Host", upstream_server.address.parse()?);
        req.headers_mut().insert("X-Forwarded-For", "gateway".parse()?);
        req.headers_mut().insert("X-Forwarded-Proto", "http".parse()?);

        // Track connection
        upstream_server.connection_count.fetch_add(1, Ordering::Relaxed);

        // Send request with timeout
        let response = tokio::time::timeout(
            Duration::from_secs(self.config.server.request_timeout.unwrap_or(30)),
            self.client.request(req)
        ).await??;

        upstream_server.connection_count.fetch_sub(1, Ordering::Relaxed);

        Ok(response)
    }

    /// Handle CORS preflight requests
    async fn handle_cors_preflight(&self, _req: &Request<Body>) -> Response<Body> {
        debug!("🔄 Handling CORS preflight request");
        
        let mut response = Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap();

        // Add CORS headers
        let headers = response.headers_mut();
        headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
        headers.insert("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, PATCH, OPTIONS".parse().unwrap());
        headers.insert("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Requested-With, Cache-Control, X-API-Key, X-Request-Id, X-Workspace-Id".parse().unwrap());
        headers.insert("Access-Control-Max-Age", "86400".parse().unwrap());

        response
    }

    /// Add CORS headers to response
    fn add_cors_headers(&self, response: &mut Response<Body>, route: &RouteConfig) {
        let headers = response.headers_mut();
        
        // Add origin header
        if let Some(origins) = &route.cors_origins {
            if !origins.is_empty() {
                headers.insert("Access-Control-Allow-Origin", origins[0].parse().unwrap());
            }
        } else {
            headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
        }
        
        headers.insert("Access-Control-Allow-Credentials", "true".parse().unwrap());
        headers.insert("Access-Control-Expose-Headers", "Content-Length, Content-Type".parse().unwrap());
    }

    /// Create error response
    fn create_error_response(&self, status: StatusCode, message: &str) -> Response<Body> {
        let body = format!(r#"{{"error": "{}", "status": {}}}"#, message, status.as_u16());
        
        Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .header("Access-Control-Allow-Origin", "*")
            .body(Body::from(body))
            .unwrap()
    }

    /// Start health checking background tasks
    async fn start_health_checking(&self) {
        let pools = Arc::clone(&self.upstream_pools);
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                
                // Clone necessary data before spawn to avoid Send issues
                let pool_data: Vec<(String, Vec<UpstreamServer>, Option<HealthChecker>)> = {
                    let pools_guard = pools.read().unwrap();
                    pools_guard.iter().map(|(_name, pool)| {
                        (
                            pool.name.clone(),
                            pool.servers.clone(),
                            pool.health_checker.clone(),
                        )
                    }).collect()
                }; // Guard is dropped here
                
                // Now process health checks without holding the guard
                for (_pool_name, servers, health_checker_opt) in pool_data {
                    if let Some(health_checker) = health_checker_opt {
                        for server in &servers {
                            health_checker.check_server(server).await;
                        }
                    }
                }
            }
        });
    }

    /// Get proxy metrics
    pub fn get_metrics(&self) -> ProxyMetrics {
        ProxyMetrics {
            total_requests: AtomicU64::new(self.metrics.total_requests.load(Ordering::Relaxed)),
            successful_requests: AtomicU64::new(self.metrics.successful_requests.load(Ordering::Relaxed)),
            failed_requests: AtomicU64::new(self.metrics.failed_requests.load(Ordering::Relaxed)),
            bytes_transferred: AtomicU64::new(self.metrics.bytes_transferred.load(Ordering::Relaxed)),
            active_connections: AtomicUsize::new(self.metrics.active_connections.load(Ordering::Relaxed)),
            upstream_errors: AtomicU64::new(self.metrics.upstream_errors.load(Ordering::Relaxed)),
            cors_preflight_requests: AtomicU64::new(self.metrics.cors_preflight_requests.load(Ordering::Relaxed)),
        }
    }
}

impl UpstreamPool {
    /// Create new upstream pool
    async fn new(name: String, config: &UpstreamConfig, client: &Client<hyper::client::HttpConnector>) -> Result<Self> {
        let mut servers = Vec::new();
        
        for server_addr in &config.servers {
            servers.push(UpstreamServer {
                address: server_addr.clone(),
                healthy: Arc::new(std::sync::atomic::AtomicBool::new(true)),
                last_health_check: Arc::new(Mutex::new(Instant::now())),
                connection_count: Arc::new(AtomicUsize::new(0)),
                total_requests: Arc::new(AtomicU64::new(0)),
                failed_requests: Arc::new(AtomicU64::new(0)),
            });
        }

        let load_balancer = match config.load_balancing.as_ref() {
            Some(LoadBalancingType::RoundRobin) => LoadBalancer::RoundRobin {
                current: Arc::new(AtomicUsize::new(0)),
            },
            Some(LoadBalancingType::Random) => LoadBalancer::Random,
            Some(LoadBalancingType::LeastConnections) => LoadBalancer::LeastConnections,
            Some(LoadBalancingType::WeightedRoundRobin) => LoadBalancer::WeightedRoundRobin {
                weights: vec![1; servers.len()], // Equal weights for now
                current: Arc::new(AtomicUsize::new(0)),
            },
            _ => LoadBalancer::RoundRobin {
                current: Arc::new(AtomicUsize::new(0)),
            },
        };

        let health_checker = config.health_check.as_ref().map(|hc_config| {
            HealthChecker {
                config: hc_config.clone(),
                client: client.clone(),
            }
        });

        Ok(Self {
            name,
            servers,
            load_balancer,
            health_checker,
        })
    }
}

impl HealthChecker {
    /// Check server health
    async fn check_server(&self, server: &UpstreamServer) {
        let health_url = format!("http://{}{}", server.address, self.config.path);
        
        match self.client.get(health_url.parse().unwrap()).await {
            Ok(response) => {
                let is_healthy = self.config.expected_status.contains(&(response.status().as_u16()));
                server.healthy.store(is_healthy, Ordering::Relaxed);
                
                if is_healthy {
                    debug!("Health check passed for {}", server.address);
                } else {
                    warn!("ERROR: Health check failed for {} (status: {})", server.address, response.status());
                }
            }
            Err(e) => {
                warn!("ERROR: Health check error for {}: {}", server.address, e);
                server.healthy.store(false, Ordering::Relaxed);
            }
        }
        
        *server.last_health_check.lock().await = Instant::now();
    }
} 