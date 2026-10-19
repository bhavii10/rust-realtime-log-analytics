import { useState, useCallback, useEffect } from 'react'
import { ChevronLeft, ChevronRight, Search } from 'lucide-react'
import { searchLogs, fetchServices } from '@/lib/api'
import { levelColor, statusColor } from '@/lib/utils'
import type { PaginatedLogs, ServiceResponse, LogFilter } from '@/types'

export default function Logs() {
  const [services, setServices] = useState<ServiceResponse[]>([])
  const [data, setData] = useState<PaginatedLogs | null>(null)
  const [loading, setLoading] = useState(true)
  const [filters, setFilters] = useState<LogFilter>({ page: 1, limit: 25 })

  useEffect(() => {
    fetchServices().then(setServices).catch(() => {})
  }, [])

  const load = useCallback(async () => {
    setLoading(true)
    try {
      const result = await searchLogs(filters)
      setData(result)
    } catch (e) {
      console.error(e)
    } finally {
      setLoading(false)
    }
  }, [filters])

  useEffect(() => { load() }, [load])

  const updateFilter = (key: keyof LogFilter, value: string | number | undefined) => {
    setFilters((prev) => ({ ...prev, [key]: value || undefined, page: 1 }))
  }

  const selectClass = "bg-[#1e293b] border border-[#334155] text-[#e2e8f0] rounded-lg px-3 py-2 text-xs focus:outline-none focus:ring-1 focus:ring-blue-500"
  const inputClass = "bg-[#1e293b] border border-[#334155] text-[#e2e8f0] rounded-lg px-3 py-2 text-xs w-28 focus:outline-none focus:ring-1 focus:ring-blue-500"

  return (
    <div className="space-y-5">
      <div>
        <h2 className="text-2xl font-bold text-white">Log Explorer</h2>
        <p className="text-sm text-[#94a3b8]">Search and filter log entries</p>
      </div>

      {/* Filters */}
      <div className="bg-[#111827] border border-[#1e293b] rounded-xl p-4">
        <div className="flex flex-wrap gap-3 items-center">
          <Search className="w-4 h-4 text-[#94a3b8]" />

          <select className={selectClass} value={filters.level || ''} onChange={(e) => updateFilter('level', e.target.value)}>
            <option value="">All Levels</option>
            {['ERROR', 'WARN', 'INFO', 'DEBUG'].map((l) => <option key={l} value={l}>{l}</option>)}
          </select>

          <select className={selectClass} value={filters.service || ''} onChange={(e) => updateFilter('service', e.target.value)}>
            <option value="">All Services</option>
            {services.map((s) => <option key={s.id} value={s.name}>{s.name}</option>)}
          </select>

          <select className={selectClass} value={filters.method || ''} onChange={(e) => updateFilter('method', e.target.value)}>
            <option value="">All Methods</option>
            {['GET', 'POST', 'PUT', 'DELETE'].map((m) => <option key={m} value={m}>{m}</option>)}
          </select>

          <select className={selectClass} value={filters.status ? String(filters.status) : ''} onChange={(e) => updateFilter('status', e.target.value ? Number(e.target.value) : undefined)}>
            <option value="">All Status</option>
            {[200, 201, 400, 404, 500, 503].map((c) => <option key={c} value={c}>{c}</option>)}
          </select>

          <input className={inputClass} type="number" placeholder="Min latency" value={filters.min_latency || ''} onChange={(e) => updateFilter('min_latency', e.target.value ? Number(e.target.value) : undefined)} />
          <input className={inputClass} type="number" placeholder="Max latency" value={filters.max_latency || ''} onChange={(e) => updateFilter('max_latency', e.target.value ? Number(e.target.value) : undefined)} />

          <button onClick={() => setFilters({ page: 1, limit: 25 })} className="text-xs text-[#94a3b8] hover:text-white transition-colors px-2 py-1">
            Clear
          </button>
        </div>
      </div>

      {/* Table */}
      <div className="bg-[#111827] border border-[#1e293b] rounded-xl overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="text-xs text-[#94a3b8] border-b border-[#1e293b] bg-[#0d1117]">
                <th className="text-left px-4 py-3">Timestamp</th>
                <th className="text-left px-4 py-3">Level</th>
                <th className="text-left px-4 py-3">Method</th>
                <th className="text-left px-4 py-3">Endpoint</th>
                <th className="text-left px-4 py-3">Status</th>
                <th className="text-left px-4 py-3">Latency</th>
                <th className="text-left px-4 py-3">IP</th>
                <th className="text-left px-4 py-3">Message</th>
              </tr>
            </thead>
            <tbody>
              {loading && !data ? (
                <tr><td colSpan={8} className="px-4 py-8 text-center text-[#94a3b8]">Loading...</td></tr>
              ) : (data?.logs || []).length === 0 ? (
                <tr><td colSpan={8} className="px-4 py-8 text-center text-[#94a3b8]">No logs found</td></tr>
              ) : (
                (data?.logs || []).map((log) => (
                  <tr key={log.id} className="border-b border-[#1e293b]/50 hover:bg-[#1e293b]/30 transition-colors">
                    <td className="px-4 py-2.5 text-xs text-[#94a3b8] font-mono whitespace-nowrap">
                      {new Date(log.timestamp).toLocaleTimeString()}
                    </td>
                    <td className="px-4 py-2.5">
                      <span className={`px-2 py-0.5 rounded text-xs font-medium border ${levelColor(log.level)}`}>{log.level}</span>
                    </td>
                    <td className="px-4 py-2.5 text-xs font-mono text-[#94a3b8]">{log.method}</td>
                    <td className="px-4 py-2.5 text-xs font-mono text-[#e2e8f0]">{log.endpoint}</td>
                    <td className={`px-4 py-2.5 text-xs font-mono font-medium ${statusColor(log.status_code)}`}>{log.status_code}</td>
                    <td className="px-4 py-2.5 text-xs text-[#94a3b8]">{log.response_time_ms}ms</td>
                    <td className="px-4 py-2.5 text-xs font-mono text-[#94a3b8]">{log.ip_address}</td>
                    <td className="px-4 py-2.5 text-xs text-[#94a3b8] truncate max-w-[200px]">{log.message}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>

        {/* Pagination */}
        {data && data.total_pages > 0 && (
          <div className="flex items-center justify-between px-4 py-3 border-t border-[#1e293b] bg-[#0d1117]">
            <span className="text-xs text-[#94a3b8]">{data.total} total results</span>
            <div className="flex items-center gap-2">
              <button
                disabled={data.page <= 1}
                onClick={() => setFilters((prev) => ({ ...prev, page: (prev.page || 1) - 1 }))}
                className="p-1.5 rounded-lg border border-[#334155] text-[#94a3b8] hover:text-white hover:bg-[#1e293b] disabled:opacity-30 transition-colors"
              >
                <ChevronLeft className="w-4 h-4" />
              </button>
              <span className="text-xs text-[#e2e8f0]">
                Page {data.page} of {data.total_pages}
              </span>
              <button
                disabled={data.page >= data.total_pages}
                onClick={() => setFilters((prev) => ({ ...prev, page: (prev.page || 1) + 1 }))}
                className="p-1.5 rounded-lg border border-[#334155] text-[#94a3b8] hover:text-white hover:bg-[#1e293b] disabled:opacity-30 transition-colors"
              >
                <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
