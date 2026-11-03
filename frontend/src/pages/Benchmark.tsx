import { useEffect, useState } from 'react'
import { fetchBenchmark } from '@/lib/api'
import type { BenchmarkResult } from '@/types'
import { cn } from '@/lib/utils'

export default function Benchmark() {
  const [results, setResults] = useState<BenchmarkResult[]>([])
  const [error, setError] = useState(false)

  useEffect(() => {
    fetchBenchmark()
      .then((d) => setResults(d.results))
      .catch(() => setError(true))
  }, [])

  const fmt = (n: number, dec = 2) => n.toFixed(dec)

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-semibold text-white">Load Test Benchmark</h2>
        <p className="text-sm text-[#94a3b8] mt-1">Results from load-tester CSV files</p>
      </div>

      {error && (
        <div className="text-red-400 text-sm bg-red-500/10 border border-red-500/20 rounded-lg p-4">
          Failed to load benchmark data. Make sure the backend is running and CSV files exist.
        </div>
      )}

      {results.length > 0 && (
        <div className="overflow-x-auto rounded-xl border border-[#1e293b]">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-[#1e293b] bg-[#0d1117]">
                {['Target (rec/s)', 'Achieved (rec/s)', 'Avg Latency', 'P95 Latency', 'P99 Latency', 'Max Latency', 'Failure Rate', 'Total Logs', 'Concurrency'].map((h) => (
                  <th key={h} className="px-4 py-3 text-left text-[#94a3b8] font-medium whitespace-nowrap">{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {results.map((r, i) => (
                <tr key={i} className="border-b border-[#1e293b] hover:bg-[#1e293b]/40 transition-colors">
                  <td className="px-4 py-3 text-white font-medium">{r.target_rate.toLocaleString()}</td>
                  <td className="px-4 py-3 text-white">{r.actual_throughput.toLocaleString()}</td>
                  <td className="px-4 py-3 text-blue-400">{fmt(r.latency_avg_ms)} ms</td>
                  <td className="px-4 py-3 text-blue-300">{fmt(r.latency_p95_ms)} ms</td>
                  <td className="px-4 py-3 text-[#94a3b8]">{fmt(r.latency_p99_ms)} ms</td>
                  <td className="px-4 py-3 text-[#94a3b8]">{fmt(r.latency_max_ms)} ms</td>
                  <td className="px-4 py-3">
                    <span className={cn(
                      'px-2 py-0.5 rounded-full text-xs font-medium',
                      r.failure_rate_pct === 0
                        ? 'bg-green-500/15 text-green-400'
                        : 'bg-red-500/15 text-red-400'
                    )}>
                      {fmt(r.failure_rate_pct)}%
                    </span>
                  </td>
                  <td className="px-4 py-3 text-[#94a3b8]">{r.total_logs.toLocaleString()}</td>
                  <td className="px-4 py-3 text-[#94a3b8]">{r.concurrency}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
