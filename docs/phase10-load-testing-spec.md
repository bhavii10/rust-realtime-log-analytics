# Phase 10 — Performance & Load Testing

## Overview

Phase 10 introduces a dedicated **load testing tool** that measures system performance under increasing traffic. This is the most critical phase for the research paper — it produces the experimental data.

## Tool: `load-tester`

A standalone Rust binary in `load-tester/` that:
- Sends concurrent HTTP requests to the backend
- Generates realistic log data (same distribution as `log-generator`)
- Measures throughput, latency percentiles, success/failure rates
- Outputs results to CSV for the research paper

## Usage

```bash
cd load-tester

# Basic: 100 logs/sec for 30 seconds
cargo run -- --rate 100 --duration 30

# Medium load: 1000 logs/sec, 200 concurrent connections
cargo run -- --rate 1000 --duration 60 --concurrency 200

# Heavy load with batch mode (Phase 11 optimization)
cargo run -- --rate 5000 --duration 60 --batch-size 50 --concurrency 200

# Maximum stress test
cargo run -- --rate 50000 --duration 60 --batch-size 100 --concurrency 500
```

## CLI Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `--rate` / `-r` | 100 | Target logs per second |
| `--duration` / `-d` | 30 | Test duration in seconds |
| `--concurrency` / `-c` | 100 | Maximum parallel HTTP requests |
| `--batch-size` / `-b` | 1 | Logs per request (1=single, >1=batch mode) |

## Metrics Collected

| Metric | Description |
|--------|-------------|
| Actual throughput | Real logs/sec achieved |
| Latency min/avg/p95/p99/max | HTTP request latency distribution |
| Success count | HTTP 2xx responses |
| Failure count | HTTP errors + timeouts |
| Failure rate | Percentage of failed requests |

## Output

Results are printed to console and saved to CSV:
```
results_1000rps_60s.csv
```

## Recommended Test Matrix

| Rate | Duration | Batch Size | Purpose |
|------|----------|------------|---------|
| 100/sec | 30s | 1 | Baseline |
| 1,000/sec | 60s | 1 | Medium single-insert |
| 1,000/sec | 60s | 50 | Medium batch comparison |
| 5,000/sec | 60s | 1 | Heavy single-insert |
| 5,000/sec | 60s | 50 | Heavy batch comparison |
| 10,000/sec | 60s | 100 | Stress test |
| 50,000/sec | 60s | 100 | Maximum throughput |
