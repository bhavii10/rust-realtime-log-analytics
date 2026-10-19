import { useCallback } from 'react'
import {
  Chart as ChartJS,
  ArcElement,
  Tooltip,
  Legend,
  CategoryScale,
  LinearScale,
  BarElement,
} from 'chart.js'
import { Doughnut, Bar } from 'react-chartjs-2'
import { usePolling } from '@/hooks/usePolling'
import { fetchAnalytics } from '@/lib/api'

ChartJS.register(ArcElement, Tooltip, Legend, CategoryScale, LinearScale, BarElement)

export default function Analytics() {
  const fetchAnalyticsCb = useCallback(() => fetchAnalytics(), [])
  const { data: analytics } = usePolling(fetchAnalyticsCb, 5000)

  const statusDist = analytics?.status_distribution || []
  const serviceTraffic = analytics?.service_traffic || []
  const topEndpoints = analytics?.top_endpoints || []
  const s = analytics?.summary

  const statusChart = {
    labels: statusDist.map((d) => `${d.status_code} (${d.percentage.toFixed(1)}%)`),
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

  const serviceChart = {
    labels: serviceTraffic.map((s) => s.service_name.replace('-service', '')),
    datasets: [
      {
        label: 'Requests',
        data: serviceTraffic.map((s) => s.request_count),
        backgroundColor: 'rgba(59,130,246,0.6)',
        borderRadius: 6,
      },
      {
        label: 'Errors',
        data: serviceTraffic.map((s) => s.error_count),
        backgroundColor: 'rgba(239,68,68,0.6)',
        borderRadius: 6,
      },
    ],
  }

  const endpointChart = {
    labels: topEndpoints.map((e) => `${e.method} ${e.endpoint}`),
    datasets: [{
      label: 'Requests',
      data: topEndpoints.map((e) => e.request_count),
      backgroundColor: 'rgba(139,92,246,0.6)',
      borderRadius: 4,
    }],
  }

  const darkBarOptions = {
    responsive: true,
    maintainAspectRatio: false,
    indexAxis: 'y' as const,
    plugins: { legend: { display: false } },
    scales: {
      x: { ticks: { color: '#94a3b8' }, grid: { color: 'rgba(30,41,59,0.5)' } },
      y: { ticks: { color: '#94a3b8', font: { size: 10, family: 'monospace' } }, grid: { display: false } },
    },
  }

  const vertBarOptions = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: { legend: { labels: { color: '#94a3b8', font: { size: 11 } } } },
    scales: {
      x: { ticks: { color: '#94a3b8' }, grid: { color: 'rgba(30,41,59,0.5)' } },
      y: { ticks: { color: '#94a3b8' }, grid: { color: 'rgba(30,41,59,0.5)' } },
    },
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-white">Analytics</h2>
        <p className="text-sm text-[#94a3b8]">Detailed system metrics and distributions</p>
      </div>

      {/* Summary Row */}
      {s && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {[
            { label: 'Max Latency', value: `${s.max_latency_ms}ms`, color: 'text-red-400' },
            { label: 'Min Latency', value: `${s.min_latency_ms}ms`, color: 'text-green-400' },
            { label: 'Avg Latency', value: `${s.average_latency_ms}ms`, color: 'text-blue-400' },
            { label: 'Slow (>2s)', value: `${s.slow_requests}`, color: 'text-orange-400' },
          ].map((item) => (
            <div key={item.label} className="bg-[#111827] border border-[#1e293b] rounded-xl p-4">
              <p className={`text-xl font-bold ${item.color}`}>{item.value}</p>
              <p className="text-xs text-[#94a3b8] mt-1">{item.label}</p>
            </div>
          ))}
        </div>
      )}

      {/* Charts Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* Status Code Distribution */}
        <div className="bg-[#111827] border border-[#1e293b] rounded-xl p-5">
          <h3 className="text-sm font-medium text-[#94a3b8] mb-4">Status Code Distribution</h3>
          <div className="h-72 flex items-center justify-center">
            {statusDist.length > 0 && (
              <Doughnut data={statusChart} options={{ responsive: true, maintainAspectRatio: false, plugins: { legend: { position: 'right', labels: { color: '#94a3b8', padding: 10, font: { size: 11 } } } }, cutout: '60%' }} />
            )}
          </div>
        </div>

        {/* Service Traffic */}
        <div className="bg-[#111827] border border-[#1e293b] rounded-xl p-5">
          <h3 className="text-sm font-medium text-[#94a3b8] mb-4">Service Traffic</h3>
          <div className="h-72">
            {serviceTraffic.length > 0 && <Bar data={serviceChart} options={vertBarOptions} />}
          </div>
        </div>

        {/* Top Endpoints */}
        <div className="lg:col-span-2 bg-[#111827] border border-[#1e293b] rounded-xl p-5">
          <h3 className="text-sm font-medium text-[#94a3b8] mb-4">Top Endpoints</h3>
          <div className="h-80">
            {topEndpoints.length > 0 && <Bar data={endpointChart} options={darkBarOptions} />}
          </div>
        </div>
      </div>

      {/* Service Detail Table */}
      {serviceTraffic.length > 0 && (
        <div className="bg-[#111827] border border-[#1e293b] rounded-xl overflow-hidden">
          <div className="px-5 py-3 border-b border-[#1e293b]">
            <h3 className="text-sm font-medium text-[#94a3b8]">Service Breakdown</h3>
          </div>
          <table className="w-full text-sm">
            <thead>
              <tr className="text-xs text-[#94a3b8] border-b border-[#1e293b]">
                <th className="text-left px-4 py-2.5">Service</th>
                <th className="text-left px-4 py-2.5">Requests</th>
                <th className="text-left px-4 py-2.5">Errors</th>
                <th className="text-left px-4 py-2.5">Error Rate</th>
                <th className="text-left px-4 py-2.5">Avg Latency</th>
              </tr>
            </thead>
            <tbody>
              {serviceTraffic.map((st) => (
                <tr key={st.service_name} className="border-b border-[#1e293b]/50 hover:bg-[#1e293b]/30">
                  <td className="px-4 py-2.5 text-xs font-medium text-[#e2e8f0]">{st.service_name}</td>
                  <td className="px-4 py-2.5 text-xs text-[#94a3b8]">{st.request_count.toLocaleString()}</td>
                  <td className="px-4 py-2.5 text-xs text-red-400">{st.error_count}</td>
                  <td className={`px-4 py-2.5 text-xs font-medium ${st.error_rate > 10 ? 'text-red-400' : 'text-green-400'}`}>
                    {st.error_rate.toFixed(1)}%
                  </td>
                  <td className="px-4 py-2.5 text-xs text-[#94a3b8]">{st.avg_latency_ms.toFixed(0)}ms</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
