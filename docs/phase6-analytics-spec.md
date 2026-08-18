# Phase 6 — Analytics Engine: API Specification

## Overview

Phase 6 introduces the Analytics Engine — the core data-processing layer of the system.
It exposes a single endpoint that runs four SQL aggregation queries against the `logs` table
and returns a unified JSON response covering summary metrics, status code distribution,
per-service traffic, and top endpoints.

All computation happens inside PostgreSQL via Rust's `sqlx` query builder.
No in-memory aggregation is performed in Rust — the database does the heavy lifting.

---

## Files Introduced

| File | Role |
|------|------|
| `src/services/analytics_service.rs` | All SQL aggregation logic |
| `src/handlers/analytics_handler.rs` | HTTP handler — extracts query params, calls service |
| `src/routes/analytics.rs` | Registers `GET /api/analytics` on the Axum router |
| `src/models/analytics.rs` | All request/response structs |

---

## Endpoint

```
GET /api/analytics
```

### Query Parameters

| Parameter | Type    | Required | Description |
|-----------|---------|----------|-------------|
| `service` | string  | No       | Filter all metrics to a single service by name (e.g. `payment-service`) |
| `minutes` | integer | No       | Restrict data to the last N minutes (e.g. `60` = last 1 hour). Omit for all-time data. |

### Examples

```
GET /api/analytics
GET /api/analytics?minutes=60
GET /api/analytics?service=payment-service
GET /api/analytics?service=auth-service&minutes=30
```

---

## Response Structure

**HTTP 200 OK**
**Content-Type:** `application/json`

```json
{
  "summary": { ... },
  "status_distribution": [ ... ],
  "service_traffic": [ ... ],
  "top_endpoints": [ ... ]
}
```

**HTTP 500 Internal Server Error** — returned if any database query fails.

---

## Response Fields — Detailed

### 1. `summary` — Overall Metrics

Single object. Covers all logs matching the query parameters.

```json
"summary": {
  "total_requests": 125420,
  "total_errors": 3821,
  "error_rate": 3.04,
  "average_latency_ms": 284.5,
  "max_latency_ms": 4998,
  "min_latency_ms": 20,
  "requests_per_second": 42.5,
  "slow_requests": 1240
}
```

| Field                 | Type    | Description |
|-----------------------|---------|-------------|
| `total_requests`      | integer | Total number of log entries matching the filter |
| `total_errors`        | integer | Count of logs where `status_code >= 500` |
| `error_rate`          | float   | `(total_errors / total_requests) * 100`, rounded to 2 decimal places |
| `average_latency_ms`  | float   | Mean `response_time_ms` across all matched logs |
| `max_latency_ms`      | integer | Highest `response_time_ms` in the result set |
| `min_latency_ms`      | integer | Lowest `response_time_ms` in the result set |
| `requests_per_second` | float   | `total_requests / (timestamp_max - timestamp_min)` in seconds |
| `slow_requests`       | integer | Count of logs where `response_time_ms > 2000` |

> **Note on `requests_per_second`:** Calculated as total requests divided by the elapsed time
> between the earliest and latest log timestamp in the filtered window. Returns `0` if only
> one log exists (no time delta).

---

### 2. `status_distribution` — Per Status Code Breakdown

Array of objects, one per distinct HTTP status code, ordered by count descending.

```json
"status_distribution": [
  { "status_code": 200, "count": 75252, "percentage": 60.0 },
  { "status_code": 201, "count": 18813, "percentage": 15.0 },
  { "status_code": 400, "count": 12542, "percentage": 10.0 },
  { "status_code": 500, "count": 10034, "percentage": 8.0  },
  { "status_code": 404, "count": 6271,  "percentage": 5.0  },
  { "status_code": 503, "count": 2508,  "percentage": 2.0  }
]
```

| Field         | Type    | Description |
|---------------|---------|-------------|
| `status_code` | integer | HTTP status code (e.g. 200, 404, 500) |
| `count`       | integer | Number of logs with this status code |
| `percentage`  | float   | `(count / total_requests) * 100`, rounded to 2 decimals |

---

### 3. `service_traffic` — Per Service Breakdown

Array of objects, one per service, ordered by `request_count` descending.
The `service` query param does **not** filter this section — it always shows all services.

```json
"service_traffic": [
  {
    "service_name": "auth-service",
    "request_count": 34210,
    "error_count": 980,
    "error_rate": 2.86,
    "avg_latency_ms": 210.45
  },
  {
    "service_name": "payment-service",
    "request_count": 31050,
    "error_count": 1540,
    "error_rate": 4.96,
    "avg_latency_ms": 412.30
  }
]
```

| Field            | Type    | Description |
|------------------|---------|-------------|
| `service_name`   | string  | Name of the service (joined from `services` table) |
| `request_count`  | integer | Total log entries for this service |
| `error_count`    | integer | Logs with `status_code >= 500` for this service |
| `error_rate`     | float   | `(error_count / request_count) * 100`, rounded to 2 decimals |
| `avg_latency_ms` | float   | Mean `response_time_ms` for this service, rounded to 2 decimals |

---

### 4. `top_endpoints` — Most Accessed Endpoints

Array of up to **10** objects, ordered by `request_count` descending.
Grouped by `(endpoint, method)` pair.

```json
"top_endpoints": [
  {
    "endpoint": "/login",
    "method": "POST",
    "request_count": 18420,
    "avg_latency_ms": 195.30
  },
  {
    "endpoint": "/orders",
    "method": "GET",
    "request_count": 15310,
    "avg_latency_ms": 88.70
  }
]
```

| Field            | Type    | Description |
|------------------|---------|-------------|
| `endpoint`       | string  | The API path (e.g. `/login`, `/orders/12345`) |
| `method`         | string  | HTTP method (e.g. `GET`, `POST`, `PUT`, `DELETE`) |
| `request_count`  | integer | Total hits for this endpoint + method combination |
| `avg_latency_ms` | float   | Mean `response_time_ms` for this endpoint |

---

## Internal Architecture

```
GET /api/analytics?service=X&minutes=Y
        │
        ▼
analytics_handler::get_analytics()
  - Extracts Query<AnalyticsQuery> from request
  - Passes pool + query to service layer
        │
        ▼
analytics_service::get_full_analytics()
  - Runs 4 queries in sequence (all async/await)
  - Each query builds a dynamic SQL string with optional WHERE clauses
        │
        ├── get_summary()             → AnalyticsResponse
        ├── get_status_distribution() → Vec<StatusCodeDistribution>
        ├── get_service_traffic()     → Vec<ServiceTraffic>
        └── get_top_endpoints()       → Vec<TopEndpoint>
        │
        ▼
FullAnalyticsResponse { summary, status_distribution, service_traffic, top_endpoints }
        │
        ▼
JSON serialized via serde → HTTP 200
```

---

## SQL Filter Logic

Both query parameters are converted to SQL fragment strings and injected into each query:

| Parameter               | SQL Fragment Generated |
|-------------------------|------------------------|
| `minutes=60`            | `AND l.timestamp >= NOW() - INTERVAL '60 minutes'` |
| `service=payment-service` | `AND s.name = 'payment-service'` |
| Neither provided        | Empty string — no extra WHERE clause beyond `WHERE 1=1` |

Every query starts with `WHERE 1=1` so filter fragments can be appended
cleanly with `AND` regardless of whether any filter is active.

---

## Data Sources

| Table      | Used For |
|------------|----------|
| `logs`     | All metrics — latency, status codes, counts |
| `services` | Joining service name for `service_traffic` and the `service` filter |

---

## Rust Structs (`src/models/analytics.rs`)

```rust
// Query parameters
pub struct AnalyticsQuery {
    pub service: Option<String>,  // ?service=payment-service
    pub minutes: Option<i64>,     // ?minutes=60
}

// Top-level response
pub struct FullAnalyticsResponse {
    pub summary:             AnalyticsResponse,
    pub status_distribution: Vec<StatusCodeDistribution>,
    pub service_traffic:     Vec<ServiceTraffic>,
    pub top_endpoints:       Vec<TopEndpoint>,
}

pub struct AnalyticsResponse {
    pub total_requests:      i64,
    pub total_errors:        i64,
    pub error_rate:          f64,
    pub average_latency_ms:  f64,
    pub max_latency_ms:      i64,
    pub min_latency_ms:      i64,
    pub requests_per_second: f64,
    pub slow_requests:       i64,
}

pub struct StatusCodeDistribution {
    pub status_code: i32,
    pub count:       i64,
    pub percentage:  f64,
}

pub struct ServiceTraffic {
    pub service_name:   String,
    pub request_count:  i64,
    pub error_count:    i64,
    pub error_rate:     f64,
    pub avg_latency_ms: f64,
}

pub struct TopEndpoint {
    pub endpoint:       String,
    pub method:         String,
    pub request_count:  i64,
    pub avg_latency_ms: f64,
}
```

---

## Full Sample Response

```json
{
  "summary": {
    "total_requests": 125420,
    "total_errors": 3821,
    "error_rate": 3.04,
    "average_latency_ms": 284.5,
    "max_latency_ms": 4998,
    "min_latency_ms": 20,
    "requests_per_second": 42.5,
    "slow_requests": 1240
  },
  "status_distribution": [
    { "status_code": 200, "count": 75252, "percentage": 60.0 },
    { "status_code": 201, "count": 18813, "percentage": 15.0 },
    { "status_code": 400, "count": 12542, "percentage": 10.0 },
    { "status_code": 500, "count": 10034, "percentage": 8.0  },
    { "status_code": 404, "count": 6271,  "percentage": 5.0  },
    { "status_code": 503, "count": 2508,  "percentage": 2.0  }
  ],
  "service_traffic": [
    {
      "service_name": "auth-service",
      "request_count": 34210,
      "error_count": 980,
      "error_rate": 2.86,
      "avg_latency_ms": 210.45
    },
    {
      "service_name": "payment-service",
      "request_count": 31050,
      "error_count": 1540,
      "error_rate": 4.96,
      "avg_latency_ms": 412.30
    },
    {
      "service_name": "order-service",
      "request_count": 30820,
      "error_count": 820,
      "error_rate": 2.66,
      "avg_latency_ms": 305.10
    },
    {
      "service_name": "user-service",
      "request_count": 29340,
      "error_count": 481,
      "error_rate": 1.64,
      "avg_latency_ms": 198.75
    }
  ],
  "top_endpoints": [
    { "endpoint": "/login",         "method": "POST", "request_count": 18420, "avg_latency_ms": 195.30 },
    { "endpoint": "/orders",        "method": "GET",  "request_count": 15310, "avg_latency_ms": 88.70  },
    { "endpoint": "/payment",       "method": "POST", "request_count": 14200, "avg_latency_ms": 430.50 },
    { "endpoint": "/users/12345",   "method": "GET",  "request_count": 13100, "avg_latency_ms": 102.40 },
    { "endpoint": "/profile",       "method": "GET",  "request_count": 11800, "avg_latency_ms": 75.20  },
    { "endpoint": "/orders/12345",  "method": "PUT",  "request_count": 10500, "avg_latency_ms": 310.80 },
    { "endpoint": "/checkout",      "method": "POST", "request_count": 9800,  "avg_latency_ms": 520.60 },
    { "endpoint": "/refresh",       "method": "POST", "request_count": 8700,  "avg_latency_ms": 145.90 },
    { "endpoint": "/invoice",       "method": "GET",  "request_count": 7600,  "avg_latency_ms": 88.30  },
    { "endpoint": "/orders/status", "method": "GET",  "request_count": 6990,  "avg_latency_ms": 65.10  }
  ]
}
```
