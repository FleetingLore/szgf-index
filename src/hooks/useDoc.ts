import { useState, useCallback } from 'react'
import type { DocFull } from '../types'
import type { GetCatResponse } from '../types/api'

interface UseDocReturn {
  currentDoc: DocFull | null
  loading: boolean
  fetchDoc: (id: string, onNotFound?: () => void) => Promise<boolean>
  toggleDeprecated: (id: string) => Promise<void>
  deleteDoc: (id: string) => Promise<boolean>
}

export function useDoc(loadCats?: () => Promise<void>): UseDocReturn {
  const [currentDoc, setCurrentDoc] = useState<DocFull | null>(null)
  const [loading, setLoading] = useState(false)

  const fetchDoc = useCallback(async (id: string, onNotFound?: () => void): Promise<boolean> => {
    setLoading(true)
    try {
      const res = await fetch(`/api/cats/${id}`)
      const data: GetCatResponse = await res.json()
      if (data.success && data.data) {
        setCurrentDoc(data.data)
        setLoading(false)
        return true
      }
      onNotFound?.()
    } catch {
      onNotFound?.()
    }
    setLoading(false)
    return false
  }, [])

  const toggleDeprecated = useCallback(async (id: string) => {
    const doc = currentDoc
    if (!doc) return
    await fetch(`/api/cats/${id}/deprecated`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ deprecated: !doc.deprecated })
    })
    await fetchDoc(id)
    if (loadCats) loadCats()
  }, [currentDoc, fetchDoc, loadCats])

  const deleteDoc = useCallback(async (id: string): Promise<boolean> => {
    const res = await fetch(`/api/cats/${id}/deleted`, { method: 'PUT' })
    return res.ok
  }, [])

  return { currentDoc, loading, fetchDoc, toggleDeprecated, deleteDoc }
}
