import React from 'react'
import { prefetchDoc } from '../utils/prefetch'
import type { DocListItem } from '../types'
import styles from './DocItem.module.css'

interface DocItemProps {
  cat: DocListItem
  onClick: () => void
}

export default function DocItem({ cat, onClick }: DocItemProps) {
  return (
    <div
      className={styles.item}
      onClick={onClick}
      onMouseEnter={() => prefetchDoc(cat.id)}
    >
      <span className={styles.title}>{cat.title}</span>
      <span className={styles.date}>
        {new Date(cat.created_at).toLocaleString()}
      </span>
    </div>
  )
}
