export interface ServiceResponse {
  id: string
  name: string
  description: string | null
}

export interface LogResponse {
  id: string
  timestamp: string
  level: string
  service_id: string
  method: string
  endpoint: string
  status_code: number
  response_time_ms: number
  ip_address: string | null
  message: string | null
}

export interface LogFilter {
  level?: string
  service?: string
  method?: string
  endpoint?: string
  status?: number
  min_latency?: number
  max_latency?: number
  page?: number
  limit?: number
}

export interface PaginatedLogs {
  logs: LogResponse[]
  total: number
  page: number
  limit: number
  total_pages: number
}

export interface AnalyticsResponse {
  total_requests: number
  total_errors: number
  error_rate: number
  average_latency_ms: number
  max_latency_ms: number
  min_latency_ms: number
  requests_per_second: number
  slow_requests: number
}

export interface StatusCodeDistribution {
  status_code: number
  count: number
  percentage: number
}

export interface ServiceTraffic {
  service_name: string
  request_count: number
  error_count: number
  error_rate: number
  avg_latency_ms: number
}

export interface TopEndpoint {
  endpoint: string
  method: string
  request_count: number
  avg_latency_ms: number
}

export interface FullAnalyticsResponse {
  summary: AnalyticsResponse
  status_distribution: StatusCodeDistribution[]
  service_traffic: ServiceTraffic[]
  top_endpoints: TopEndpoint[]
}

export interface Alert {
  id: string
  alert_type: string
  service_id: string | null
  message: string
  severity: string
  threshold_value: number | null
  actual_value: number | null
  created_at: string
  resolved_at: string | null
}

export interface AlertSummary {
  total_alerts: number
  critical: number
  high: number
  medium: number
  low: number
  recent_alerts: Alert[]
}
