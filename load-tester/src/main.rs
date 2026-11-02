use chrono::Utc;
use clap::Parser;
use rand::{RngExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use uuid::Uuid;

const BACKEND_URL: &str = "http://127.0.0.1:8080";

// ======================================================
// Phase 10 — Performance & Load Testing Tool
//
// Generates logs at configurable rates with concurrent
// HTTP requests, measuring:
//   - Throughput (actual requests/sec)
//   - Latency (min, avg, max, p95, p99)
//   - Success / failure counts
//   - Duration
//
// Supports two modes:
//   - single: POST /api/logs (one log per request)
//   - batch:  POST /api/logs/batch (N logs per request)
//
// Usage:
//   cargo run -- --rate 1000 --duration 30
//   cargo run -- --rate 5000 --duration 60 --batch-size 50
// ======================================================

#[derive(Parser, Debug)]
#[command(
    name = "load-tester",
    about = "Performance load tester for the Rust Log Analytics System"
)]
struct Args {
    /// Target logs per second
    #[arg(short, long, default_value_t = 100)]
    rate: u64,

    /// Test duration in seconds
    #[arg(short, long, default_value_t = 30)]
    duration: u64,

    /// Max concurrent requests
    #[arg(short, long, default_value_t = 100)]
    concurrency: usize,

    /// Batch size (1 = single mode, >1 = batch mode)
    #[arg(short, long, default_value_t = 1)]
    batch_size: usize,
}

#[derive(Debug, Deserialize, Clone)]
struct Service {
    id: Uuid,
    name: String,
    #[allow(dead_code)]
    description: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
struct LogRequest {
    timestamp: String,
    level: String,
    service_id: Uuid,
    method: String,
    endpoint: String,
    status_code: i32,
    response_time_ms: i64,
    ip_address: Option<String>,
    message: Option<String>,
}

// ======================================================
// Shared counters for real-time stats
// ======================================================

struct Stats {
    total_requests: AtomicU64,
    successful: AtomicU64,
    failed: AtomicU64,
    total_logs_sent: AtomicU64,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("╔══════════════════════════════════════════════╗");
    println!("║    🔥 Rust Log Analytics — Load Tester       ║");
    println!("╚══════════════════════════════════════════════╝");
    println!();
    println!("  Target rate:   {} logs/sec", args.rate);
    println!("  Duration:      {}s", args.duration);
    println!("  Concurrency:   {} max parallel requests", args.concurrency);
    println!("  Batch size:    {}", if args.batch_size > 1 {
        format!("{} logs/request (batch mode)", args.batch_size)
    } else {
        "1 (single mode)".to_string()
    });
    println!();

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .pool_max_idle_per_host(args.concurrency)
        .build()
        .expect("Failed to create HTTP client");

    // Fetch services
    let services = match fetch_services(&client).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ Failed to fetch services: {}", e);
            eprintln!("   Make sure the backend is running on port 8080.");
            return;
        }
    };

    if services.is_empty() {
        eprintln!("❌ No services found.");
        return;
    }

    println!("✅ Loaded {} services", services.len());

    let stats = Arc::new(Stats {
        total_requests: AtomicU64::new(0),
        successful: AtomicU64::new(0),
        failed: AtomicU64::new(0),
        total_logs_sent: AtomicU64::new(0),
    });

    let semaphore = Arc::new(Semaphore::new(args.concurrency));

    // Collect latencies for percentile calculation
    let latencies: Arc<tokio::sync::Mutex<Vec<f64>>> =
        Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // Calculate how many requests per second we need
    let requests_per_sec = if args.batch_size > 1 {
        (args.rate as f64 / args.batch_size as f64).ceil() as u64
    } else {
        args.rate
    };

    let total_requests = requests_per_sec * args.duration;

    println!(
        "📊 Sending {} HTTP requests ({} logs total) over {}s...\n",
        total_requests,
        total_requests * args.batch_size as u64,
        args.duration
    );

    let start = Instant::now();

    // Spawn a progress reporter
    let stats_clone = Arc::clone(&stats);
    let progress_handle = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(5));
        ticker.tick().await; // skip first instant tick
        loop {
            ticker.tick().await;
            let sent = stats_clone.total_logs_sent.load(Ordering::Relaxed);
            let ok = stats_clone.successful.load(Ordering::Relaxed);
            let fail = stats_clone.failed.load(Ordering::Relaxed);
            println!(
                "  ⏱  logs_sent={:<8} requests_ok={:<8} requests_fail={:<6}",
                sent, ok, fail
            );
        }
    });

    // Generate and send requests
    let interval_ns = if requests_per_sec > 0 {
        1_000_000_000u64 / requests_per_sec
    } else {
        1_000_000_000
    };

    let mut handles = Vec::new();

    for i in 0..total_requests {
        // Rate limiting: sleep between requests
        let expected_time = Duration::from_nanos(i * interval_ns);
        let elapsed = start.elapsed();
        if expected_time > elapsed {
            tokio::time::sleep(expected_time - elapsed).await;
        }

        let permit = Arc::clone(&semaphore);
        let client = client.clone();
        let services = services.clone();
        let stats = Arc::clone(&stats);
        let latencies = Arc::clone(&latencies);
        let batch_size = args.batch_size;

        let handle = tokio::spawn(async move {
            let _permit = permit.acquire().await.unwrap();

            let req_start = Instant::now();

            let result = if batch_size > 1 {
                let logs: Vec<LogRequest> = (0..batch_size)
                    .map(|_| generate_log(&services))
                    .collect();

                client
                    .post(format!("{}/api/logs/batch", BACKEND_URL))
                    .json(&logs)
                    .send()
                    .await
            } else {
                let log = generate_log(&services);
                client
                    .post(format!("{}/api/logs", BACKEND_URL))
                    .json(&log)
                    .send()
                    .await
            };

            let latency_ms = req_start.elapsed().as_secs_f64() * 1000.0;

            stats.total_requests.fetch_add(1, Ordering::Relaxed);
            stats
                .total_logs_sent
                .fetch_add(batch_size as u64, Ordering::Relaxed);

            match result {
                Ok(resp) if resp.status().is_success() => {
                    stats.successful.fetch_add(1, Ordering::Relaxed);
                    latencies.lock().await.push(latency_ms);
                }
                Ok(resp) => {
                    stats.failed.fetch_add(1, Ordering::Relaxed);
                    eprintln!("  ❌ HTTP {}", resp.status());
                }
                Err(e) => {
                    stats.failed.fetch_add(1, Ordering::Relaxed);
                    eprintln!("  ❌ {}", e);
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let _ = handle.await;
    }

    progress_handle.abort();

    let total_duration = start.elapsed();

    // ======================================================
    // Calculate and print results
    // ======================================================

    let total_reqs = stats.total_requests.load(Ordering::Relaxed);
    let ok = stats.successful.load(Ordering::Relaxed);
    let fail = stats.failed.load(Ordering::Relaxed);
    let logs_sent = stats.total_logs_sent.load(Ordering::Relaxed);

    let mut lats = latencies.lock().await;
    lats.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let avg_latency = if !lats.is_empty() {
        lats.iter().sum::<f64>() / lats.len() as f64
    } else {
        0.0
    };

    let min_latency = lats.first().copied().unwrap_or(0.0);
    let max_latency = lats.last().copied().unwrap_or(0.0);
    let p95 = percentile(&lats, 95.0);
    let p99 = percentile(&lats, 99.0);

    let actual_rps = if total_duration.as_secs_f64() > 0.0 {
        logs_sent as f64 / total_duration.as_secs_f64()
    } else {
        0.0
    };

    let failure_rate = if total_reqs > 0 {
        (fail as f64 / total_reqs as f64) * 100.0
    } else {
        0.0
    };

    println!();
    println!("╔══════════════════════════════════════════════╗");
    println!("║              📊 LOAD TEST RESULTS            ║");
    println!("╠══════════════════════════════════════════════╣");
    println!("║  Target rate:        {:>8} logs/sec        ║", args.rate);
    println!("║  Actual throughput:  {:>8.1} logs/sec        ║", actual_rps);
    println!("║  Duration:           {:>8.2}s               ║", total_duration.as_secs_f64());
    println!("║  Total logs sent:    {:>8}                 ║", logs_sent);
    println!("║  HTTP requests:      {:>8}                 ║", total_reqs);
    println!("║  Successful:         {:>8}                 ║", ok);
    println!("║  Failed:             {:>8}                 ║", fail);
    println!("║  Failure rate:       {:>7.2}%                ║", failure_rate);
    println!("╠══════════════════════════════════════════════╣");
    println!("║  LATENCY (ms)                                ║");
    println!("║  Min:                {:>8.2}                 ║", min_latency);
    println!("║  Avg:                {:>8.2}                 ║", avg_latency);
    println!("║  P95:                {:>8.2}                 ║", p95);
    println!("║  P99:                {:>8.2}                 ║", p99);
    println!("║  Max:                {:>8.2}                 ║", max_latency);
    println!("╚══════════════════════════════════════════════╝");

    // Write results to CSV for the research paper
    let csv_file = format!("results_{}rps_{}s.csv", args.rate, args.duration);
    let csv_content = format!(
        "target_rate,actual_throughput,duration_s,total_logs,http_requests,successful,failed,failure_rate_pct,latency_min_ms,latency_avg_ms,latency_p95_ms,latency_p99_ms,latency_max_ms,batch_size,concurrency\n\
         {},{:.1},{:.2},{},{},{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{},{}\n",
        args.rate,
        actual_rps,
        total_duration.as_secs_f64(),
        logs_sent,
        total_reqs,
        ok,
        fail,
        failure_rate,
        min_latency,
        avg_latency,
        p95,
        p99,
        max_latency,
        args.batch_size,
        args.concurrency,
    );

    if let Err(e) = std::fs::write(&csv_file, &csv_content) {
        eprintln!("⚠️  Could not write {}: {}", csv_file, e);
    } else {
        println!("\n📁 Results saved to {}", csv_file);
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((p / 100.0) * sorted.len() as f64).ceil() as usize;
    sorted[idx.min(sorted.len()) - 1]
}

// ======================================================
// Log generation (same logic as log-generator)
// ======================================================

fn generate_log(services: &[Service]) -> LogRequest {
    let mut rng = rand::rng();

    let service = &services[rng.random_range(0..services.len())];

    let endpoints = get_endpoints(&service.name);
    let ep = &endpoints[rng.random_range(0..endpoints.len())];

    let status_code = match rng.random_range(0..100u32) {
        0..=59 => 200,
        60..=74 => 201,
        75..=84 => 400,
        85..=89 => 404,
        90..=97 => 500,
        _ => 503,
    };

    let level = match status_code {
        500 | 503 => "ERROR",
        400 | 404 => "WARN",
        _ => "INFO",
    };

    let response_time_ms = match rng.random_range(0..100u32) {
        0..=64 => rng.random_range(20..=300i64),
        65..=89 => rng.random_range(301..=1500),
        _ => rng.random_range(1501..=5000),
    };

    LogRequest {
        timestamp: Utc::now().to_rfc3339(),
        level: level.to_string(),
        service_id: service.id,
        method: ep.0.to_string(),
        endpoint: ep.1.to_string(),
        status_code,
        response_time_ms,
        ip_address: Some(format!("192.168.1.{}", rng.random_range(1..=254u8))),
        message: Some(format!("{} {} {}", service.name, ep.1, status_code)),
    }
}

fn get_endpoints(service_name: &str) -> Vec<(&'static str, &'static str)> {
    match service_name {
        "auth-service" => vec![
            ("POST", "/login"), ("POST", "/logout"),
            ("POST", "/refresh"), ("POST", "/verify"),
        ],
        "payment-service" => vec![
            ("POST", "/payment"), ("POST", "/refund"),
            ("GET", "/invoice"), ("POST", "/checkout"),
        ],
        "order-service" => vec![
            ("GET", "/orders"), ("POST", "/orders"),
            ("PUT", "/orders/12345"), ("GET", "/orders/status"),
        ],
        "user-service" => vec![
            ("GET", "/users"), ("GET", "/profile"),
            ("PUT", "/profile"), ("GET", "/users/12345"),
        ],
        _ => vec![("GET", "/health"), ("GET", "/api/data")],
    }
}

async fn fetch_services(
    client: &Client,
) -> Result<Vec<Service>, reqwest::Error> {
    client
        .get(format!("{}/api/services", BACKEND_URL))
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<Service>>()
        .await
}
