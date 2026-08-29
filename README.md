# Rust Real-Time Log Analytics System

Rust-based data-intensive application that processes and analyzes high-volume real-time log data. The system focuses on efficient data ingestion, processing, aggregation, real-time analytics, and anomaly detection using Rust.

## Quick Start

**Terminal 1 — Backend**
```
cd rust-realtime-log-analytics/backend
sqlx migrate run
cargo run
```

**Terminal 2 — Log Generator**
```
cd rust-realtime-log-analytics/log-generator
cargo run
```

## API Endpoints

| Method | Endpoint | Phase | Description |
|--------|----------|-------|-------------|
| GET | `/api/services` | 4 | List all registered services |
| GET | `/api/logs` | 4 | Get last 100 logs |
| POST | `/api/logs` | 3 | Ingest a new log entry |
| GET | `/api/logs/search` | 5 | Advanced log search with filters & pagination |
| GET | `/api/analytics` | 6 | Full analytics (summary, status codes, traffic, top endpoints) |
| GET | `/api/analytics/realtime` | 7 | Real-time metrics snapshots (last 30 by default) |
| GET | `/api/alerts` | 8 | Anomaly detection alerts |
| GET | `/api/alerts/summary` | 8 | Alert counts by severity + recent alerts |

### Query Parameter Examples

```
GET /api/logs/search?level=ERROR&service=payment-service&status=500&min_latency=2000&page=1&limit=50
GET /api/analytics?service=payment-service&minutes=60
GET /api/analytics/realtime?limit=60
GET /api/alerts?severity=HIGH&alert_type=HIGH_ERROR_RATE&limit=20
```

## Phases

- **Phase 1** — Project & DB Setup ✅
- **Phase 2** — Log Generator ✅
- **Phase 3** — Log Ingestion ✅
- **Phase 4** — Log Retrieval ✅
- **Phase 5** — Advanced Log Search & Filtering ✅
- **Phase 6** — Analytics Engine ✅
- **Phase 7** — Real-Time Analytics ✅
- **Phase 8** — Anomaly Detection ✅
