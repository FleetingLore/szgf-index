import { useState, useEffect, useRef, useCallback } from 'react'

interface StickyHeaderReturn {
  headerRef: React.RefObject<HTMLDivElement | null>
  isSticky: boolean
  translateY: number
}

export function useStickyHeader(): StickyHeaderReturn {
  const [isSticky, setIsSticky] = useState(false)
  const [translateY, setTranslateY] = useState(0)
  const headerRef = useRef<HTMLDivElement | null>(null)
  const lastScrollY = useRef(0)
  const ticking = useRef(false)

  const handleScroll = useCallback(() => {
    if (ticking.current) return
    ticking.current = true

    requestAnimationFrame(() => {
      const scrollTop = window.pageYOffset || document.documentElement.scrollTop
      const headerHeight = headerRef.current?.offsetHeight || 56

      if (scrollTop > headerHeight) {
        setIsSticky(true)
      } else {
        setIsSticky(false)
        setTranslateY(0)
        lastScrollY.current = scrollTop
        ticking.current = false
        return
      }

      const delta = scrollTop - lastScrollY.current
      if (delta > 5) {
        setTranslateY(-headerHeight)
      } else if (delta < -5) {
        setTranslateY(0)
      }

      lastScrollY.current = scrollTop
      ticking.current = false
    })
  }, [])

  useEffect(() => {
    window.addEventListener('scroll', handleScroll, { passive: true })
    return () => window.removeEventListener('scroll', handleScroll)
  }, [handleScroll])

  return { headerRef, isSticky, translateY }
}
