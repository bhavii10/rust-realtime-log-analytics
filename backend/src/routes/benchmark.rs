use axum::{routing::get, Json, Router};
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use std::fs;

#[derive(Serialize)]
pub struct BenchmarkResult {
    target_rate: f64,
    actual_throughput: f64,
    duration_s: f64,
    total_logs: u64,
    successful: u64,
    failed: u64,
    failure_rate_pct: f64,
    latency_avg_ms: f64,
    latency_p95_ms: f64,
    latency_p99_ms: f64,
    latency_max_ms: f64,
    batch_size: u32,
    concurrency: u32,
}

pub fn routes() -> Router<PgPool> {
    Router::new().route("/api/benchmark", get(get_benchmark))
}

async fn get_benchmark() -> Json<Value> {
    let mut results: Vec<BenchmarkResult> = Vec::new();

    if let Ok(entries) = fs::read_dir("../load-tester") {
        let mut paths: Vec<_> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("results_") && n.ends_with(".csv"))
                    .unwrap_or(false)
            })
            .collect();

        paths.sort();

        for path in paths {
            if let Ok(content) = fs::read_to_string(&path) {
                let mut lines = content.lines();
                lines.next();
                if let Some(row) = lines.next() {
                    let cols: Vec<&str> = row.split(',').collect();
                    if cols.len() >= 15 {
                        results.push(BenchmarkResult {
                            target_rate: cols[0].parse().unwrap_or(0.0),
                            actual_throughput: cols[1].parse().unwrap_or(0.0),
                            duration_s: cols[2].parse().unwrap_or(0.0),
                            total_logs: cols[3].parse().unwrap_or(0),
                            successful: cols[5].parse().unwrap_or(0),
                            failed: cols[6].parse().unwrap_or(0),
                            failure_rate_pct: cols[7].parse().unwrap_or(0.0),
                            latency_avg_ms: cols[9].parse().unwrap_or(0.0),
                            latency_p95_ms: cols[10].parse().unwrap_or(0.0),
                            latency_p99_ms: cols[11].parse().unwrap_or(0.0),
                            latency_max_ms: cols[12].parse().unwrap_or(0.0),
                            batch_size: cols[13].parse().unwrap_or(0),
                            concurrency: cols[14].parse().unwrap_or(0),
                        });
                    }
                }
            }
        }
    }

    results.sort_by(|a, b| a.target_rate.partial_cmp(&b.target_rate).unwrap());
    Json(serde_json::json!({ "results": results }))
}
