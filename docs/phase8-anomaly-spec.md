# Phase 8 — Anomaly Detection Engine

## Overview

Phase 8 adds **automatic anomaly detection** to the real-time processing pipeline. After each metrics snapshot (Phase 7), the system checks for three types of anomalies and stores alerts in the `alerts` table. This makes the system proactively detect issues rather than waiting for manual inspection.

---

## Architecture

```
Phase 7 Snapshot (every 10s)
        │
        ▼
┌─────────────────────────────────┐
│     Anomaly Detection Engine     │
│                                  │
│  Check 1: Error Rate > 10%?     │
│  Check 2: Avg Latency > 2000ms? │
│  Check 3: Traffic > 3x average? │
│                                  │
│  Deduplication: suppress same    │
│  alert type within 60 seconds    │
└─────────────────────────────────┘
        │
        ▼ (if anomaly detected)
   alerts table
        │
        ▼
GET /api/alerts
GET /api/alerts/summary
```

---

## Files Introduced / Modified

| File | Role |
|------|------|
| `src/services/anomaly_service.rs` | **NEW** — Anomaly checks, alert creation, alert retrieval |
| `src/models/analytics.rs` | **MODIFIED** — Added `Alert`, `AlertQuery`, `AlertSummary` structs |
| `src/handlers/analytics_handler.rs` | **MODIFIED** — Added `get_alerts`, `get_alert_summary` handlers |
| `src/routes/analytics.rs` | **MODIFIED** — Added `/api/alerts` and `/api/alerts/summary` routes |
| `src/main.rs` | **MODIFIED** — Calls `check_anomalies()` after each snapshot |

---

## Anomaly Thresholds

| Alert Type | Condition | Severity Mapping |
|------------|-----------|------------------|
| `HIGH_ERROR_RATE` | `error_rate > 10%` | ≥50% → CRITICAL, ≥25% → HIGH, ≥10% → MEDIUM |
| `HIGH_LATENCY` | `avg_latency > 2000ms` | ≥5000ms → CRITICAL, ≥3000ms → HIGH, ≥2000ms → MEDIUM |
| `TRAFFIC_SPIKE` | `rps > 3× historical average` | Always HIGH |

### Traffic Spike Detection

The historical average is computed from the **6 snapshots before the current one**. This means the system needs at least 7 snapshots (70 seconds of data) before traffic spike detection activates.

### Deduplication

To prevent alert spam, the same alert type cannot fire more than once within 60 seconds. This is checked via a SQL query before inserting.

---

## New Endpoints

### `GET /api/alerts`

Returns recent alerts with optional filtering.

| Parameter    | Type   | Required | Default | Description |
|-------------|--------|----------|---------|-------------|
| `severity`  | string | No       | —       | Filter: LOW, MEDIUM, HIGH, CRITICAL |
| `alert_type`| string | No       | —       | Filter: HIGH_ERROR_RATE, HIGH_LATENCY, TRAFFIC_SPIKE |
| `limit`     | integer| No       | 50      | Max alerts to return (max 200) |

```json
[
  {
    "id": "uuid",
    "alert_type": "HIGH_ERROR_RATE",
    "service_id": null,
    "message": "🚨 Error rate spiked to 15.00% (threshold: 10%)",
    "severity": "MEDIUM",
    "threshold_value": 10.0,
    "actual_value": 15.0,
    "created_at": "2026-08-29T10:05:20+00:00",
    "resolved_at": null
  }
]
```

### `GET /api/alerts/summary`

Returns aggregate alert counts by severity plus the 10 most recent alerts.

```json
{
  "total_alerts": 42,
  "critical": 2,
  "high": 8,
  "medium": 25,
  "low": 7,
  "recent_alerts": [ ... ]
}
```

---

## Database Table (already exists in migration)

```sql
CREATE TABLE alerts (
    id UUID PRIMARY KEY,
    alert_type VARCHAR(50) NOT NULL,
    service_id UUID,
    message TEXT NOT NULL,
    severity VARCHAR(20) NOT NULL,
    threshold_value DOUBLE PRECISION,
    actual_value DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,

    CONSTRAINT fk_alerts_service FOREIGN KEY (service_id) REFERENCES services(id),
    CONSTRAINT chk_alert_severity CHECK (severity IN ('LOW', 'MEDIUM', 'HIGH', 'CRITICAL'))
);
```

---

## Console Output

When the server is running, anomaly alerts are printed to the terminal:

```
⚡ Real-time processing engine started (10s interval)
🔍 Anomaly detection active — thresholds:
   • Error rate  > 10%
   • Avg latency > 2000ms
   • Traffic     > 3x historical average

🚨 ALERT [HIGH_ERROR_RATE] 🚨 Error rate spiked to 15.00% (threshold: 10%) — severity: MEDIUM (actual: 15.00, threshold: 10.00)
🚨 ALERT [HIGH_LATENCY] 🚨 Average latency reached 2417ms (threshold: 2000ms) — severity: MEDIUM (actual: 2417.00, threshold: 2000.00)
```
