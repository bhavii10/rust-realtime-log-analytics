import { NavLink, Outlet } from 'react-router-dom'
import {
  LayoutDashboard,
  ScrollText,
  BarChart3,
  AlertTriangle,
  Activity,
} from 'lucide-react'
import { usePolling } from '@/hooks/usePolling'
import { fetchHealth } from '@/lib/api'
import { cn } from '@/lib/utils'

const navItems = [
  { to: '/', icon: LayoutDashboard, label: 'Dashboard' },
  { to: '/logs', icon: ScrollText, label: 'Logs' },
  { to: '/analytics', icon: BarChart3, label: 'Analytics' },
  { to: '/alerts', icon: AlertTriangle, label: 'Alerts' },
]

export default function Layout() {
  const { data: health } = usePolling(fetchHealth, 10000)
  const isConnected = health?.status === 'ok'

  return (
    <div className="flex h-screen overflow-hidden">
      {/* Sidebar */}
      <aside className="w-64 border-r border-[#1e293b] bg-[#0d1117] flex flex-col">
        {/* Logo */}
        <div className="p-5 border-b border-[#1e293b]">
          <div className="flex items-center gap-2">
            <Activity className="w-6 h-6 text-blue-500" />
            <div>
              <h1 className="text-base font-semibold text-white">Log Analytics</h1>
              <p className="text-xs text-[#94a3b8]">Real-time Monitoring</p>
            </div>
          </div>
        </div>

        {/* Navigation */}
        <nav className="flex-1 p-3 space-y-1">
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.to === '/'}
              className={({ isActive }) =>
                cn(
                  'flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm transition-all',
                  isActive
                    ? 'bg-blue-500/15 text-blue-400'
                    : 'text-[#94a3b8] hover:text-white hover:bg-[#1e293b]'
                )
              }
            >
              <item.icon className="w-4 h-4" />
              {item.label}
            </NavLink>
          ))}
        </nav>

        {/* Connection Status */}
        <div className="p-4 border-t border-[#1e293b]">
          <div className="flex items-center gap-2 text-xs">
            <div
              className={cn(
                'w-2 h-2 rounded-full',
                isConnected ? 'bg-green-500 shadow-[0_0_6px_rgba(34,197,94,0.5)]' : 'bg-red-500'
              )}
            />
            <span className="text-[#94a3b8]">
              {isConnected ? 'Backend Connected' : 'Disconnected'}
            </span>
          </div>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-y-auto bg-[#0a0a0f] p-6">
        <Outlet />
      </main>
    </div>
  )
}
