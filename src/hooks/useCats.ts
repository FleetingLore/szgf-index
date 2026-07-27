import { useState, useEffect, useCallback } from 'react'
import type { DocListItem } from '../types'
import type { ListCatsResponse } from '../types/api'

interface UseCatsReturn {
  cats: DocListItem[]
  siteName: string
  loading: boolean
  loadCats: () => Promise<void>
}

export function useCats(): UseCatsReturn {
  const [cats, setCats] = useState<DocListItem[]>([])
  const [siteName, setSiteName] = useState('')
  const [loading, setLoading] = useState(true)

  const loadCats = useCallback(async () => {
    const res = await fetch('/api/cats')
    const data: ListCatsResponse = await res.json()
    if (data.success && data.data) {
      setCats(data.data)
      if (data.siteName) setSiteName(data.siteName)
    }
    setLoading(false)
  }, [])

  useEffect(() => { loadCats() }, [loadCats])

  return { cats, siteName, loading: loading, loadCats }
}
