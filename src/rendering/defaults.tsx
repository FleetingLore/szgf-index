/**
 * 默认格式渲染器注册 — Markdown 只是一个默认项。
 * 添加新格式只需在此文件中加一条 register()。
 */

import 'katex/dist/katex.min.css'
import React from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import remarkMath from 'remark-math'
import rehypeKatex from 'rehype-katex'
import { formatRegistry } from './registry'

/** Markdown 渲染组件 */
const MarkdownRenderer: React.FC<{ content: string }> = ({ content }) => (
  <ReactMarkdown
    remarkPlugins={[remarkGfm, remarkMath]}
    rehypePlugins={[rehypeKatex]}
  >
    {content}
  </ReactMarkdown>
)

/** 纯文本渲染组件 */
const PlainTextRenderer: React.FC<{ content: string }> = ({ content }) => (
  <pre style={{ whiteSpace: 'pre-wrap', fontFamily: 'var(--font-mono)' }}>
    {content}
  </pre>
)

/** 注册所有默认格式渲染器 */
export function registerDefaultFormats() {
  formatRegistry.register({
    format: 'markdown',
    name: 'Markdown',
    extensions: ['md', 'markdown'],
    component: MarkdownRenderer,
    isDefault: true,
  })

  formatRegistry.register({
    format: 'plaintext',
    name: 'Plain Text',
    extensions: ['txt'],
    component: PlainTextRenderer,
  })
}
