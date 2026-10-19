import { useCallback } from 'react'
import {
  Chart as ChartJS,
  ArcElement,
  Tooltip,
  Legend,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Filler,
} from 'chart.js'
import { Doughnut, Line } from 'react-chartjs-2'
import { Activity, AlertTriangle, Clock, Zap, TrendingUp, Server } from 'lucide-react'
import { usePolling } from '@/hooks/usePolling'
import { fetchAnalytics, fetchLogs } from '@/lib/api'
import { formatNumber, levelColor, statusColor } from '@/lib/utils'

ChartJS.register(ArcElement, Tooltip, Legend, CategoryScale, LinearScale, PointElement, LineElement, Filler)

export default function Dashboard() {
  const fetchAnalyticsCb = useCallback(() => fetchAnalytics(), [])
  const fetchLogsCb = useCallback(() => fetchLogs(), [])
  const { data: analytics } = usePolling(fetchAnalyticsCb, 5000)
  const { data: logs } = usePolling(fetchLogsCb, 5000)

  const s = analytics?.summary

  const statCards = [
    { label: 'Total Requests', value: s ? formatNumber(s.total_requests) : '—', icon: Activity, color: 'text-blue-400', bg: 'bg-blue-500/10' },
    { label: 'Total Errors', value: s ? formatNumber(s.total_errors) : '—', icon: AlertTriangle, color: 'text-red-400', bg: 'bg-red-500/10' },
    { label: 'Error Rate', value: s ? `${s.error_rate}%` : '—', icon: TrendingUp, color: s && s.error_rate > 10 ? 'text-red-400' : 'text-green-400', bg: s && s.error_rate > 10 ? 'bg-red-500/10' : 'bg-green-500/10' },
    { label: 'Avg Latency', value: s ? `${s.average_latency_ms}ms` : '—', icon: Clock, color: s && s.average_latency_ms > 1000 ? 'text-yellow-400' : 'text-green-400', bg: s && s.average_latency_ms > 1000 ? 'bg-yellow-500/10' : 'bg-green-500/10' },
    { label: 'Req/sec', value: s ? s.requests_per_second.toFixed(1) : '—', icon: Zap, color: 'text-purple-400', bg: 'bg-purple-500/10' },
    { label: 'Slow Requests', value: s ? formatNumber(s.slow_requests) : '—', icon: Server, color: 'text-orange-400', bg: 'bg-orange-500/10' },
  ]

  const statusDist = analytics?.status_distribution || []
  const doughnutData = {
    labels: statusDist.map((d) => `${d.status_code}`),
    datasets: [{
      data: statusDist.map((d) => d.count),
      backgroundColor: statusDist.map((d) => {
        if (d.status_code >= 500) return 'rgba(239,68,68,0.8)'
        if (d.status_code >= 400) return 'rgba(245,158,11,0.8)'
        if (d.status_code === 201) return 'rgba(59,130,246,0.8)'
        return 'rgba(34,197,94,0.8)'
      }),
      borderWidth: 0,
    }],
  }

  const serviceData = analytics?.service_traffic || []
  const serviceChart = {
    labels: serviceData.map((s) => s.service_name.replace('-service', '')),
    datasets: [{
      label: 'Requests',
      data: serviceData.map((s) => s.request_count),
      borderColor: 'rgba(59,130,246,1)',
      backgroundColor: 'rgba(59,130,246,0.1)',
      fill: true,
      tension: 0.4,
    }],
  }

  const chartOptions = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: { display: false },
    },
    scales: {
      x: { ticks: { color: '#94a3b8' }, grid: { color: 'rgba(30,41,59,0.5)' } },
      y: { ticks: { color: '#94a3b8' }, grid: { color: 'rgba(30,41,59,0.5)' } },
    },
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-white">Dashboard</h2>
        <p className="text-sm text-[#94a3b8]">Real-time system overview</p>
      </div>

      {/* Stat Cards */}
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
        {statCards.map((card) => (
          <div key={card.label} className="bg-[#111827] border border-[#1e293b] rounded-xl p-4">
            <div className="flex items-center gap-2 mb-2">
              <div className={`p-1.5 rounded-lg ${card.bg}`}>
                <card.icon className={`w-3.5 h-3.5 ${card.color}`} />
              </div>
            </div>
            <p className="text-2xl font-bold text-white">{card.value}</p>
            <p className="text-xs text-[#94a3b8] mt-1">{card.label}</p>
          </div>
        ))}
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <div className="lg:col-span-2 bg-[#111827] border border-[#1e293b] rounded-xl p-5">
          <h3 className="text-sm font-medium text-[#94a3b8] mb-4">Service Traffic</h3>
          <div className="h-64">
            {serviceData.length > 0 && <Line data={serviceChart} options={chartOptions} />}
          </div>
        </div>
        <div className="bg-[#111827] border border-[#1e293b] rounded-xl p-5">
          <h3 className="text-sm font-medium text-[#94a3b8] mb-4">Status Codes</h3>
          <div className="h-64 flex items-center justify-center">
            {statusDist.length > 0 && (
              <Doughnut
                data={doughnutData}
                options={{
                  responsive: true,
                  maintainAspectRatio: false,
                  plugins: {
                    legend: { position: 'bottom', labels: { color: '#94a3b8', padding: 12, font: { size: 11 } } },
                  },
                  cutout: '65%',
                }}
              />
            )}
          </div>
        </div>
      </div>

      {/* Recent Logs */}
      <div className="bg-[#111827] border border-[#1e293b] rounded-xl overflow-hidden">
        <div className="px-5 py-3 border-b border-[#1e293b]">
          <h3 className="text-sm font-medium text-[#94a3b8]">Recent Logs</h3>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="text-xs text-[#94a3b8] border-b border-[#1e293b]">
                <th className="text-left px-4 py-2.5">Level</th>
                <th className="text-left px-4 py-2.5">Endpoint</th>
                <th className="text-left px-4 py-2.5">Status</th>
                <th className="text-left px-4 py-2.5">Latency</th>
                <th className="text-left px-4 py-2.5">Message</th>
              </tr>
            </thead>
            <tbody>
              {(logs || []).slice(0, 10).map((log) => (
                <tr key={log.id} className="border-b border-[#1e293b]/50 hover:bg-[#1e293b]/30 transition-colors">
                  <td className="px-4 py-2">
                    <span className={`px-2 py-0.5 rounded text-xs font-medium border ${levelColor(log.level)}`}>
                      {log.level}
                    </span>
                  </td>
                  <td className="px-4 py-2 font-mono text-xs text-[#e2e8f0]">
                    <span className="text-[#94a3b8]">{log.method}</span> {log.endpoint}
                  </td>
                  <td className={`px-4 py-2 font-mono text-xs ${statusColor(log.status_code)}`}>
                    {log.status_code}
                  </td>
                  <td className="px-4 py-2 text-xs text-[#94a3b8]">{log.response_time_ms}ms</td>
                  <td className="px-4 py-2 text-xs text-[#94a3b8] truncate max-w-xs">{log.message}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}
