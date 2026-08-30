# Phase 7 — Real-Time Analytics Processing Engine

## Overview

Phase 7 transforms the system from on-demand analytics (Phase 6) to **continuous real-time processing**. A background task runs every 10 seconds, computes metrics from incoming logs, and stores snapshots in the `analytics` table. This is where the **data-intensive + Rust** aspect becomes strongest — the system processes log streams continuously without blocking the HTTP server.

---

## Architecture

```
Log Generator → POST /api/logs → logs table
                                      ↓
                          ┌───────────────────────┐
                          │  Background Task       │
                          │  (every 10 seconds)    │
                          │                        │
                          │  1. Query last 10s     │
                          │     of logs            │
                          │  2. Compute metrics    │
                          │  3. Store snapshot     │
                          │     in analytics table │
                          └───────────────────────┘
                                      ↓
                              analytics table
                                      ↓
                          GET /api/analytics/realtime
```

---

## Files Introduced / Modified

| File | Role |
|------|------|
| `src/services/realtime_service.rs` | **NEW** — Snapshot computation + retrieval |
| `src/main.rs` | **MODIFIED** — Spawns background `tokio::spawn` task |
| `src/models/analytics.rs` | **MODIFIED** — Added `RealtimeQuery` struct |
| `src/handlers/analytics_handler.rs` | **MODIFIED** — Added `get_realtime_metrics` handler |
| `src/routes/analytics.rs` | **MODIFIED** — Added `/api/analytics/realtime` route |
| `src/services/mod.rs` | **MODIFIED** — Exports `realtime_service` |

---

## New Endpoint

```
GET /api/analytics/realtime
GET /api/analytics/realtime?limit=60
```

### Query Parameters

| Parameter | Type    | Required | Default | Description |
|-----------|---------|----------|---------|-------------|
| `limit`   | integer | No       | 30      | Number of recent snapshots to return (max 200) |

### Response (HTTP 200)

```json
[
  {
    "id": "uuid",
    "recorded_at": "2026-08-29T10:00:00+00:00",
    "request_count": 95,
    "error_count": 8,
    "error_rate": 8.42,
    "average_latency_ms": 312.5,
    "requests_per_second": 9.5,
    "slow_request_count": 3
  },
  ...
]
```

---

## Background Task Details

- **Interval**: 10 seconds
- **Window**: Queries logs from the last 10 seconds
- **Computation**: Single SQL aggregation query (no in-memory processing)
- **Storage**: Each snapshot is a row in the `analytics` table
- Runs as a `tokio::spawn` task — non-blocking, shares the same connection pool

### Metrics Computed Per Snapshot

| Metric | Description |
|--------|-------------|
| `request_count` | Total logs in the 10s window |
| `error_count` | Logs with `status_code >= 500` |
| `error_rate` | `(error_count / request_count) * 100` |
| `average_latency_ms` | Mean `response_time_ms` |
| `requests_per_second` | `request_count / 10` |
| `slow_request_count` | Logs with `response_time_ms > 2000` |

---

## Database Table (already exists in migration)

```sql
CREATE TABLE analytics (
    id UUID PRIMARY KEY,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    request_count BIGINT NOT NULL DEFAULT 0,
    error_count BIGINT NOT NULL DEFAULT 0,
    error_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    average_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    requests_per_second DOUBLE PRECISION NOT NULL DEFAULT 0,
    slow_request_count BIGINT NOT NULL DEFAULT 0
);
```
