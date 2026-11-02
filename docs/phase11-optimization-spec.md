# Phase 11 — Optimization & Benchmarking

## Overview

Phase 11 optimizes the backend for high-throughput log ingestion based on bottlenecks identified during Phase 10 load testing. The goal is to produce before/after benchmark comparisons for the research paper.

## Optimizations Applied

### 1. Batch Insert Endpoint (`POST /api/logs/batch`)

**Before:** Each log requires a separate HTTP request → separate DB INSERT.
**After:** A single HTTP request can insert up to 500 logs via multi-row INSERT.

```
# Before (N logs = N HTTP requests = N DB round-trips)
POST /api/logs → INSERT INTO logs VALUES (...)  × N

# After (N logs = 1 HTTP request = 1 DB round-trip)
POST /api/logs/batch → INSERT INTO logs VALUES (...), (...), (...) ... × 1
```

**Impact:** Reduces HTTP overhead and DB round-trips by up to 500×.

**File:** `backend/src/services/log_service.rs` — `create_logs_batch()`

### 2. Connection Pool Tuning

| Setting | Before | After | Reason |
|---------|--------|-------|--------|
| `max_connections` | 20 | 50 | Handle more concurrent queries |
| `min_connections` | 2 | 5 | Reduce cold-start latency |
| `acquire_timeout` | 10s | 30s | Prevent timeouts under burst load |
| `idle_timeout` | none | 600s | Reclaim unused connections |
| `max_lifetime` | none | 1800s | Prevent stale connections |

**File:** `backend/src/database/postgres.rs`

### 3. Additional Database Indexes

New composite indexes for common query patterns:

| Index | Columns | Helps With |
|-------|---------|------------|
| `idx_logs_service_status` | `(service_id, status_code)` | Per-service error rate queries |
| `idx_logs_level_timestamp` | `(level, timestamp)` | Log filtering by level + time range |
| `idx_logs_status_timestamp` | `(status_code, timestamp)` | Status distribution over time |
| `idx_logs_response_time` | `(response_time_ms)` | Latency percentile queries |
| `idx_logs_created_at` | `(created_at)` | Ingestion-order queries |

**File:** `backend/migrations/20261102120000_phase11_optimization_indexes.sql`

## Benchmarking: Before vs After

Run the load tester in both single and batch modes to compare:

```bash
# BEFORE optimization (single inserts)
cargo run -- --rate 1000 --duration 60 --batch-size 1

# AFTER optimization (batch inserts)
cargo run -- --rate 1000 --duration 60 --batch-size 50
```

Expected improvements:
- **Throughput**: 3–10× higher with batch inserts
- **Latency**: Lower p95/p99 due to fewer DB round-trips
- **Failure rate**: Lower under heavy load due to pool tuning
- **DB performance**: Faster queries due to composite indexes

## Research Paper Data Points

The before/after comparison directly supports the research question:

> How effectively can Rust handle high-volume real-time log processing in terms of throughput, latency, memory consumption, and scalability?

Key findings to report:
1. Single-insert throughput ceiling
2. Batch-insert throughput improvement factor
3. Connection pool saturation point (before vs after)
4. Latency distribution under increasing load
5. Failure rate curves at different rates
