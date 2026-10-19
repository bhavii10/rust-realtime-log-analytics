import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatNumber(n: number): string {
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M'
  if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K'
  return n.toLocaleString()
}

export function levelColor(level: string): string {
  switch (level) {
    case 'ERROR': return 'bg-red-500/20 text-red-400 border-red-500/30'
    case 'WARN': return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30'
    case 'INFO': return 'bg-blue-500/20 text-blue-400 border-blue-500/30'
    case 'DEBUG': return 'bg-gray-500/20 text-gray-400 border-gray-500/30'
    default: return 'bg-gray-500/20 text-gray-400 border-gray-500/30'
  }
}

export function severityColor(severity: string): string {
  switch (severity) {
    case 'CRITICAL': return 'bg-red-600/20 text-red-400 border-red-600/30'
    case 'HIGH': return 'bg-orange-500/20 text-orange-400 border-orange-500/30'
    case 'MEDIUM': return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30'
    case 'LOW': return 'bg-blue-500/20 text-blue-400 border-blue-500/30'
    default: return 'bg-gray-500/20 text-gray-400 border-gray-500/30'
  }
}

export function statusColor(code: number): string {
  if (code >= 500) return 'text-red-400'
  if (code >= 400) return 'text-yellow-400'
  if (code >= 200 && code < 300) return 'text-green-400'
  return 'text-gray-400'
}
