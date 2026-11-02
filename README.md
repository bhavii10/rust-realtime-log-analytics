# Rust Real-Time Log Analytics System

Rust-based data-intensive application that processes and analyzes high-volume real-time log data. The system focuses on efficient data ingestion, processing, aggregation, real-time analytics, and anomaly detection using Rust.

**Research Paper Theme:** Rust-Based Applications for Data-Intensive Systems

**Research Question:** How effectively can Rust handle high-volume real-time log processing in terms of throughput, latency, memory consumption, and scalability?

## Quick Start

**Prerequisites:** Rust toolchain, Docker, Node.js

**Terminal 1 — PostgreSQL**
```
docker run -d --name log_analytics_postgres \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=log_analytics \
  -p 5432:5432 postgres:16
```

**Terminal 2 — Backend**
```
cd backend
echo 'DATABASE_URL=postgres://postgres:postgres@localhost:5432/log_analytics' > .env
sqlx migrate run
cargo run
```

**Terminal 3 — Log Generator**
```
cd log-generator
cargo run -- --rate 10
```

**Terminal 4 — Frontend**
```
cd frontend
npm install
npm run dev
```

**Terminal 5 — Load Testing (Phase 10)**
```
cd load-tester
cargo run -- --rate 1000 --duration 60 --concurrency 200
cargo run -- --rate 5000 --duration 60 --batch-size 50   # batch mode
```

Open **http://localhost:5173** for the dashboard.

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Backend | Rust (Axum, Tokio, SQLx) |
| Database | PostgreSQL |
| Log Generator | Rust (Reqwest, Clap, Rand) |
| Load Tester | Rust (Tokio, Reqwest, Semaphore-based concurrency) |
| Frontend | React + TypeScript + Tailwind CSS + Chart.js |
| Containerization | Docker |

## API Endpoints

| Method | Endpoint | Phase | Description |
|--------|----------|-------|-------------|
| GET | `/api/health` | 1 | Health check |
| GET | `/api/services` | 4 | List all registered services |
| GET | `/api/logs` | 4 | Get last 100 logs |
| POST | `/api/logs` | 3 | Ingest a new log entry |
| POST | `/api/logs/batch` | 11 | Batch insert up to 500 logs in one request |
| GET | `/api/logs/search` | 5 | Advanced log search with filters & pagination |
| GET | `/api/analytics` | 6 | Full analytics (summary, status codes, traffic, top endpoints) |
| GET | `/api/alerts` | 8 | Anomaly detection alerts with severity summary |

### Query Parameter Examples

```
GET /api/logs/search?level=ERROR&service=payment-service&status=500&min_latency=2000&page=1&limit=50
GET /api/analytics?service=payment-service&minutes=60
GET /api/alerts?severity=HIGH&alert_type=HIGH_ERROR_RATE&limit=20
```

## Development Phases — ALL COMPLETE ✅

| Phase | Description | Status |
|-------|-------------|--------|
| 1 | Project & DB Setup (Rust backend, PostgreSQL, schema, migrations) | ✅ Done |
| 2 | Log Generator (multi-service, configurable rate, realistic data) | ✅ Done |
| 3 | Log Ingestion (Generator → POST /api/logs → PostgreSQL) | ✅ Done |
| 4 | Log Retrieval (GET /api/services, GET /api/logs) | ✅ Done |
| 5 | Advanced Search & Filtering (query params, pagination, QueryBuilder) | ✅ Done |
| 6 | Analytics Engine (aggregation queries, status distribution, service traffic) | ✅ Done |
| 7 | Real-Time Analytics (on-demand computation, polling from frontend) | ✅ Done |
| 8 | Anomaly Detection (background task every 30s, error rate/latency/traffic spike alerts) | ✅ Done |
| 9 | React Dashboard (dark theme, 4 pages, Chart.js, live polling) | ✅ Done |
| 10 | Performance & Load Testing (concurrent load tester, CSV results, latency percentiles) | ✅ Done |
| 11 | Optimization & Benchmarking (batch inserts, pool tuning, composite indexes) | ✅ Done |

### Phase 10 — Load Testing Tool

The `load-tester/` binary sends concurrent HTTP requests at configurable rates:

```bash
cd load-tester

# Baseline test
cargo run -- --rate 100 --duration 30

# Medium load
cargo run -- --rate 1000 --duration 60 --concurrency 200

# Heavy load with batch optimization
cargo run -- --rate 5000 --duration 60 --batch-size 50 --concurrency 200

# Maximum stress test
cargo run -- --rate 50000 --duration 60 --batch-size 100 --concurrency 500
```

Results are saved to CSV files (e.g., `results_1000rps_60s.csv`) for the research paper.

### Phase 11 — Optimizations Applied

| Optimization | Before | After | Impact |
|-------------|--------|-------|--------|
| Log insertion | Single INSERT per request | Multi-row batch INSERT (up to 500) | 3–10× throughput |
| Connection pool | 20 max / 2 min | 50 max / 5 min + idle/lifetime timeouts | Higher concurrency |
| DB indexes | 6 basic indexes | 11 indexes including 5 composite indexes | Faster queries |

**Benchmark comparison:**
```bash
# Before: single inserts
cargo run -- --rate 1000 --duration 60 --batch-size 1

# After: batch inserts
cargo run -- --rate 1000 --duration 60 --batch-size 50
```

## Recommended Test Matrix for Research Paper

| Rate | Duration | Batch | Concurrency | Purpose |
|------|----------|-------|-------------|---------|
| 100/sec | 30s | 1 | 50 | Baseline |
| 1,000/sec | 60s | 1 | 100 | Single-insert medium |
| 1,000/sec | 60s | 50 | 100 | Batch medium (comparison) |
| 5,000/sec | 60s | 1 | 200 | Single-insert heavy |
| 5,000/sec | 60s | 50 | 200 | Batch heavy (comparison) |
| 10,000/sec | 60s | 100 | 300 | Stress test |
| 50,000/sec | 60s | 100 | 500 | Maximum throughput |

## Frontend Pages

| Page | Route | Description |
|------|-------|-------------|
| Dashboard | `/` | Stat cards, service traffic chart, status code doughnut, recent logs |
| Logs | `/logs` | Searchable log table with filters, pagination, color-coded levels |
| Analytics | `/analytics` | Status distribution, service traffic bars, top endpoints, breakdown table |
| Alerts | `/alerts` | Severity summary cards, alert history table with color-coded badges |

## Project Structure

```
rust-realtime-log-analytics/
├── backend/
│   ├── Cargo.toml
│   ├── migrations/
│   │   ├── 20260809073040_create_initial_schema.sql
│   │   └── 20261102120000_phase11_optimization_indexes.sql
│   └── src/
│       ├── main.rs
│       ├── config/
│       ├── database/        (optimized pool — Phase 11)
│       ├── errors/
│       ├── handlers/        (+ batch insert handler)
│       ├── models/
│       ├── routes/          (+ /api/logs/batch route)
│       └── services/        (+ batch insert service)
├── log-generator/
│   ├── Cargo.toml
│   └── src/main.rs
├── load-tester/             (Phase 10 — NEW)
│   ├── Cargo.toml
│   └── src/main.rs
├── frontend/
│   ├── package.json
│   └── src/
│       ├── App.tsx
│       ├── components/
│       ├── hooks/
│       ├── lib/
│       ├── pages/
│       └── types/
├── docs/
│   ├── phase6-analytics-spec.md
│   ├── phase7-realtime-spec.md
│   ├── phase8-anomaly-spec.md
│   ├── phase10-load-testing-spec.md
│   └── phase11-optimization-spec.md
└── README.md
```

## Research Paper Connection

This project directly supports the research paper by providing:

1. **Experimental setup** — A real system processing real workloads
2. **Measurable metrics** — Throughput, latency, memory, CPU at various loads
3. **Comparative analysis** — Before/after optimization data (single vs batch inserts)
4. **Scalability evidence** — Performance from 100 to 50,000+ logs/sec
5. **Rust-specific advantages** — Zero-cost abstractions, memory safety without GC, fearless concurrency
6. **Load testing tool** — Reproducible benchmarks with CSV output for charts/tables
