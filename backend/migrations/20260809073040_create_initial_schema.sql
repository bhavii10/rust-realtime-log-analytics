CREATE TABLE services (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO services (id, name, description)
VALUES
(
    gen_random_uuid(),
    'auth-service',
    'Handles authentication and user login operations'
),
(
    gen_random_uuid(),
    'user-service',
    'Handles user profile and account operations'
),
(
    gen_random_uuid(),
    'payment-service',
    'Handles payment processing operations'
),
(
    gen_random_uuid(),
    'order-service',
    'Handles order creation and management'
);

CREATE TABLE logs (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    level VARCHAR(20) NOT NULL,
    service_id UUID NOT NULL,
    method VARCHAR(10) NOT NULL,
    endpoint VARCHAR(255) NOT NULL,
    status_code INTEGER NOT NULL,
    response_time_ms BIGINT NOT NULL,
    ip_address INET,
    message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_logs_service
        FOREIGN KEY (service_id)
        REFERENCES services(id),

    CONSTRAINT chk_log_level
        CHECK (
            level IN (
                'TRACE',
                'DEBUG',
                'INFO',
                'WARN',
                'ERROR',
                'FATAL'
            )
        ),

    CONSTRAINT chk_status_code
        CHECK (
            status_code BETWEEN 100 AND 599
        ),

    CONSTRAINT chk_response_time
        CHECK (
            response_time_ms >= 0
        )
);

CREATE INDEX idx_logs_timestamp
ON logs(timestamp);

CREATE INDEX idx_logs_level
ON logs(level);

CREATE INDEX idx_logs_service
ON logs(service_id);

CREATE INDEX idx_logs_status_code
ON logs(status_code);

CREATE INDEX idx_logs_endpoint
ON logs(endpoint);

CREATE INDEX idx_logs_timestamp_service
ON logs(timestamp, service_id);


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

    CONSTRAINT fk_alerts_service
        FOREIGN KEY (service_id)
        REFERENCES services(id),

    CONSTRAINT chk_alert_severity
        CHECK (
            severity IN (
                'LOW',
                'MEDIUM',
                'HIGH',
                'CRITICAL'
            )
        )
);

CREATE INDEX idx_alerts_created_at
ON alerts(created_at);

CREATE INDEX idx_alerts_service
ON alerts(service_id);


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

CREATE INDEX idx_analytics_recorded_at
ON analytics(recorded_at);