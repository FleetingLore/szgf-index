/**
 * 格式注册表 — Markdown 只是一个默认项。
 * 每种文档格式对应一个渲染器组件。
 * 未来加新格式只需 `register()` 一条调用。
 */

import type { ComponentType } from 'react'
import type { DocFormat } from '../types'

/** 格式渲染器接口 */
export interface FormatRenderer {
  /** 格式标识 */
  format: DocFormat
  /** 格式名称 */
  name: string
  /** 支持的文件扩展名 */
  extensions: string[]
  /** 渲染组件：接收文档内容，返回 React 节点 */
  component: ComponentType<{ content: string }>
  /** 是否为默认格式 */
  isDefault?: boolean
}

/** 格式注册表 */
class FormatRegistry {
  private renderers: FormatRenderer[] = []

  /** 注册一个格式渲染器 */
  register(renderer: FormatRenderer) {
    this.renderers.push(renderer)
  }

  /** 根据文件名查找渲染器 */
  resolveByFilename(filename: string): FormatRenderer | null {
    const ext = filename.split('.').pop()?.toLowerCase()
    if (!ext) return this.default()
    return (
      this.renderers.find((r) => r.extensions.includes(ext)) ?? this.default()
    )
  }

  /** 根据格式类型查找渲染器 */
  resolveByFormat(format: DocFormat): FormatRenderer | null {
    return this.renderers.find((r) => r.format === format) ?? this.default()
  }

  /** 获取默认渲染器 */
  default(): FormatRenderer | null {
    return (
      this.renderers.find((r) => r.isDefault) ??
      this.renderers[0] ??
      null
    )
  }

  /** 所有已注册的扩展名（用于上传文件类型校验） */
  allExtensions(): string[] {
    return this.renderers.flatMap((r) => r.extensions)
  }

  /** 所有已注册的渲染器 */
  all(): FormatRenderer[] {
    return [...this.renderers]
  }
}

/** 全局单例 */
export const formatRegistry = new FormatRegistry()
