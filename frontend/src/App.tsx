import { BrowserRouter, Routes, Route } from 'react-router-dom'
import Layout from '@/components/Layout'
import Dashboard from '@/pages/Dashboard'
import Logs from '@/pages/Logs'
import Analytics from '@/pages/Analytics'
import Alerts from '@/pages/Alerts'
import Benchmark from '@/pages/Benchmark'

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<Layout />}>
          <Route path="/" element={<Dashboard />} />
          <Route path="/logs" element={<Logs />} />
          <Route path="/analytics" element={<Analytics />} />
          <Route path="/alerts" element={<Alerts />} />
          <Route path="/benchmark" element={<Benchmark />} />
        </Route>
      </Routes>
    </BrowserRouter>
  )
}
