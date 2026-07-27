import { useState, useEffect, useCallback } from 'react'
import type { CheckSessionResponse, CreateSessionResponse } from '../types/api'

interface UseAdminReturn {
  isAdmin: boolean
  loading: boolean
  createSession: (token: string) => Promise<void>
  destroySession: () => Promise<void>
}

export function useAdmin(): UseAdminReturn {
  const [isAdmin, setIsAdmin] = useState(false)
  const [loading, setLoading] = useState(true)

  const checkSession = useCallback(async () => {
    const token = sessionStorage.getItem('session_token')
    if (!token) {
      setLoading(false)
      ;(window as any).__isAdmin = false
      return
    }

    try {
      const res = await fetch('/api/admin/session', {
        headers: { 'X-Session-Token': token }
      })
      const data: CheckSessionResponse = await res.json()
      const admin = data.isAdmin === true
      setIsAdmin(admin)
      ;(window as any).__isAdmin = admin
    } catch (err) {
      console.error('检查 session 失败:', err)
      ;(window as any).__isAdmin = false
    }
    setLoading(false)
  }, [])

  useEffect(() => {
    checkSession()
  }, [checkSession])

  const createSession = useCallback(async (token: string) => {
    sessionStorage.setItem('session_token', token)
    setIsAdmin(true)
    ;(window as any).__isAdmin = true
  }, [])

  const destroySession = useCallback(async () => {
    const token = sessionStorage.getItem('session_token')
    if (token) {
      await fetch('/api/admin/session', {
        method: 'DELETE',
        headers: { 'X-Session-Token': token }
      })
      sessionStorage.removeItem('session_token')
    }
    setIsAdmin(false)
    ;(window as any).__isAdmin = false
  }, [])

  return { isAdmin, loading, createSession, destroySession }
}
