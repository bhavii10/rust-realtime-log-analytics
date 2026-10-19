import type {
  ServiceResponse,
  LogResponse,
  PaginatedLogs,
  LogFilter,
  FullAnalyticsResponse,
  AlertSummary,
} from '@/types'

const API_BASE = 'http://localhost:8080'

async function fetchJson<T>(url: string): Promise<T> {
  const res = await fetch(url)
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  return res.json()
}

export async function fetchHealth(): Promise<{ status: string }> {
  return fetchJson(`${API_BASE}/api/health`)
}

export async function fetchServices(): Promise<ServiceResponse[]> {
  return fetchJson(`${API_BASE}/api/services`)
}

export async function fetchLogs(): Promise<LogResponse[]> {
  return fetchJson(`${API_BASE}/api/logs`)
}

export async function searchLogs(filters: LogFilter): Promise<PaginatedLogs> {
  const params = new URLSearchParams()
  if (filters.level) params.set('level', filters.level)
  if (filters.service) params.set('service', filters.service)
  if (filters.method) params.set('method', filters.method)
  if (filters.endpoint) params.set('endpoint', filters.endpoint)
  if (filters.status) params.set('status', String(filters.status))
  if (filters.min_latency) params.set('min_latency', String(filters.min_latency))
  if (filters.max_latency) params.set('max_latency', String(filters.max_latency))
  if (filters.page) params.set('page', String(filters.page))
  if (filters.limit) params.set('limit', String(filters.limit))
  return fetchJson(`${API_BASE}/api/logs/search?${params}`)
}

export async function fetchAnalytics(
  params?: { service?: string; minutes?: number }
): Promise<FullAnalyticsResponse> {
  const qp = new URLSearchParams()
  if (params?.service) qp.set('service', params.service)
  if (params?.minutes) qp.set('minutes', String(params.minutes))
  const qs = qp.toString()
  return fetchJson(`${API_BASE}/api/analytics${qs ? `?${qs}` : ''}`)
}

export async function fetchAlerts(
  params?: { severity?: string; alert_type?: string; limit?: number }
): Promise<AlertSummary> {
  const qp = new URLSearchParams()
  if (params?.severity) qp.set('severity', params.severity)
  if (params?.alert_type) qp.set('alert_type', params.alert_type)
  if (params?.limit) qp.set('limit', String(params.limit))
  const qs = qp.toString()
  return fetchJson(`${API_BASE}/api/alerts${qs ? `?${qs}` : ''}`)
}
