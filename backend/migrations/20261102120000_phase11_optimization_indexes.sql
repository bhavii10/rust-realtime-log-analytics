-- ======================================================
-- Phase 11 — Optimization: Additional Database Indexes
--
-- These indexes improve query performance under heavy load:
--   1. Composite (service_id, status_code) — speeds up per-service error rate queries
--   2. Composite (level, timestamp) — speeds up log filtering by level + time range
--   3. Composite (status_code, timestamp) — speeds up status distribution queries
--   4. response_time_ms — speeds up latency percentile / slow request queries
--   5. created_at on logs — speeds up ingestion-order queries
-- ======================================================

CREATE INDEX IF NOT EXISTS idx_logs_service_status
ON logs(service_id, status_code);

CREATE INDEX IF NOT EXISTS idx_logs_level_timestamp
ON logs(level, timestamp);

CREATE INDEX IF NOT EXISTS idx_logs_status_timestamp
ON logs(status_code, timestamp);

CREATE INDEX IF NOT EXISTS idx_logs_response_time
ON logs(response_time_ms);

CREATE INDEX IF NOT EXISTS idx_logs_created_at
ON logs(created_at);
