import { useCallback } from 'react'
import { ShieldCheck, AlertTriangle, AlertOctagon, Info } from 'lucide-react'
import { usePolling } from '@/hooks/usePolling'
import { fetchAlerts } from '@/lib/api'
import { severityColor } from '@/lib/utils'

export default function Alerts() {
  const fetchAlertsCb = useCallback(() => fetchAlerts({ limit: 50 }), [])
  const { data: alertData, loading } = usePolling(fetchAlertsCb, 10000)

  const summary = alertData
  const alerts = alertData?.recent_alerts || []

  const severityCounts = [
    { label: 'Critical', count: summary?.critical || 0, color: 'text-red-400', bg: 'bg-red-500/10', icon: AlertOctagon },
    { label: 'High', count: summary?.high || 0, color: 'text-orange-400', bg: 'bg-orange-500/10', icon: AlertTriangle },
    { label: 'Medium', count: summary?.medium || 0, color: 'text-yellow-400', bg: 'bg-yellow-500/10', icon: Info },
    { label: 'Low', count: summary?.low || 0, color: 'text-blue-400', bg: 'bg-blue-500/10', icon: ShieldCheck },
  ]

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-white">Alerts</h2>
        <p className="text-sm text-[#94a3b8]">Anomaly detection and system alerts</p>
      </div>

      {/* Severity Summary */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {severityCounts.map((s) => (
          <div key={s.label} className="bg-[#111827] border border-[#1e293b] rounded-xl p-4">
            <div className="flex items-center justify-between">
              <div>
                <p className={`text-2xl font-bold ${s.color}`}>{s.count}</p>
                <p className="text-xs text-[#94a3b8] mt-1">{s.label}</p>
              </div>
              <div className={`p-2 rounded-lg ${s.bg}`}>
                <s.icon className={`w-5 h-5 ${s.color}`} />
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Alerts Table */}
      <div className="bg-[#111827] border border-[#1e293b] rounded-xl overflow-hidden">
        <div className="px-5 py-3 border-b border-[#1e293b] flex items-center justify-between">
          <h3 className="text-sm font-medium text-[#94a3b8]">
            Recent Alerts ({summary?.total_alerts || 0} total)
          </h3>
        </div>

        {loading && alerts.length === 0 ? (
          <div className="px-4 py-12 text-center text-[#94a3b8]">Loading...</div>
        ) : alerts.length === 0 ? (
          <div className="px-4 py-12 text-center">
            <ShieldCheck className="w-12 h-12 text-green-500 mx-auto mb-3" />
            <p className="text-[#e2e8f0] font-medium">No anomalies detected</p>
            <p className="text-xs text-[#94a3b8] mt-1">All systems operating normally</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="text-xs text-[#94a3b8] border-b border-[#1e293b] bg-[#0d1117]">
                  <th className="text-left px-4 py-3">Time</th>
                  <th className="text-left px-4 py-3">Severity</th>
                  <th className="text-left px-4 py-3">Type</th>
                  <th className="text-left px-4 py-3">Message</th>
                  <th className="text-left px-4 py-3">Threshold</th>
                  <th className="text-left px-4 py-3">Actual</th>
                </tr>
              </thead>
              <tbody>
                {alerts.map((alert) => (
                  <tr key={alert.id} className="border-b border-[#1e293b]/50 hover:bg-[#1e293b]/30 transition-colors">
                    <td className="px-4 py-2.5 text-xs font-mono text-[#94a3b8] whitespace-nowrap">
                      {new Date(alert.created_at).toLocaleString()}
                    </td>
                    <td className="px-4 py-2.5">
                      <span className={`px-2 py-0.5 rounded text-xs font-medium border ${severityColor(alert.severity)}`}>
                        {alert.severity}
                      </span>
                    </td>
                    <td className="px-4 py-2.5 text-xs font-mono text-[#e2e8f0]">{alert.alert_type}</td>
                    <td className="px-4 py-2.5 text-xs text-[#94a3b8] max-w-xs truncate">{alert.message}</td>
                    <td className="px-4 py-2.5 text-xs text-[#94a3b8]">
                      {alert.threshold_value != null ? alert.threshold_value.toFixed(1) : '—'}
                    </td>
                    <td className="px-4 py-2.5 text-xs font-medium text-red-400">
                      {alert.actual_value != null ? alert.actual_value.toFixed(1) : '—'}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  )
}
