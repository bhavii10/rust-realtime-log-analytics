import { useEffect, useRef, useState, useCallback } from 'react'

export function usePolling<T>(
  fetcher: () => Promise<T>,
  intervalMs: number = 5000,
  deps: unknown[] = []
) {
  const [data, setData] = useState<T | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const mountedRef = useRef(true)

  const refetch = useCallback(async () => {
    try {
      const result = await fetcher()
      if (mountedRef.current) {
        setData(result)
        setError(null)
        setLoading(false)
      }
    } catch (err) {
      if (mountedRef.current) {
        setError(err instanceof Error ? err.message : 'Failed to fetch')
        setLoading(false)
      }
    }
  }, [fetcher, ...deps])

  useEffect(() => {
    mountedRef.current = true
    refetch()
    const id = setInterval(refetch, intervalMs)
    return () => {
      mountedRef.current = false
      clearInterval(id)
    }
  }, [refetch, intervalMs])

  return { data, error, loading, refetch }
}
