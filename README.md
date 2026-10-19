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

Open **http://localhost:5173** for the dashboard.

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Backend | Rust (Axum, Tokio, SQLx) |
| Database | PostgreSQL |
| Log Generator | Rust (Reqwest, Clap, Rand) |
| Frontend | React + TypeScript + Tailwind CSS + Chart.js |
| Containerization | Docker |

## API Endpoints

| Method | Endpoint | Phase | Description |
|--------|----------|-------|-------------|
| GET | `/api/health` | 1 | Health check |
| GET | `/api/services` | 4 | List all registered services |
| GET | `/api/logs` | 4 | Get last 100 logs |
| POST | `/api/logs` | 3 | Ingest a new log entry |
| GET | `/api/logs/search` | 5 | Advanced log search with filters & pagination |
| GET | `/api/analytics` | 6 | Full analytics (summary, status codes, traffic, top endpoints) |
| GET | `/api/alerts` | 8 | Anomaly detection alerts with severity summary |

### Query Parameter Examples

```
GET /api/logs/search?level=ERROR&service=payment-service&status=500&min_latency=2000&page=1&limit=50
GET /api/analytics?service=payment-service&minutes=60
GET /api/alerts?severity=HIGH&alert_type=HIGH_ERROR_RATE&limit=20
```

## Development Phases

### Completed

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

### Remaining

| Phase | Description | Status |
|-------|-------------|--------|
| 10 | Performance & Load Testing | ❌ Next |
| 11 | Optimization & Benchmarking | ❌ Planned |

### Phase 10 — Performance & Load Testing

Generate logs at increasing rates and measure system behavior:

| Rate | Throughput | Processing Latency | CPU | Memory | DB Performance | Failure Rate |
|------|-----------|-------------------|-----|--------|----------------|-------------|
| 100/sec | — | — | — | — | — | — |
| 1,000/sec | — | — | — | — | — | — |
| 5,000/sec | — | — | — | — | — | — |
| 10,000/sec | — | — | — | — | — | — |
| 50,000/sec | — | — | — | — | — | — |

This produces the experimental data for the research paper.

### Phase 11 — Optimization & Benchmarking

Once baseline benchmarks exist, investigate and apply:
- Batch processing (bulk inserts instead of one-by-one)
- Connection pooling tuning
- Async processing improvements
- Buffered writes
- Database indexing optimization
- Concurrent processing

Compare before/after benchmarks to demonstrate optimization impact.

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
│   └── src/
│       ├── main.rs
│       ├── config/
│       ├── database/
│       ├── errors/
│       ├── handlers/
│       ├── models/
│       ├── routes/
│       └── services/
├── log-generator/
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
└── README.md
```

## Research Paper Connection

This project directly supports the research paper by providing:

1. **Experimental setup** — A real system processing real workloads
2. **Measurable metrics** — Throughput, latency, memory, CPU at various loads
3. **Comparative analysis** — Before/after optimization data
4. **Scalability evidence** — Performance from 100 to 50,000+ logs/sec
5. **Rust-specific advantages** — Zero-cost abstractions, memory safety without GC, fearless concurrency
