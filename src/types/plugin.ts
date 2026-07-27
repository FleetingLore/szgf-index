/**
 * 插件系统类型定义
 *
 * 前端插件注册表允许认证、存储、渲染等模块以插件形式接入。
 * 每个插件实现 Plugin 接口并通过 PluginRegistry 注册。
 */

import type { AuthPlugin } from './auth'

/** 插件种类 */
export type PluginKind = 'auth' | 'renderer' | 'upload' | 'panel'

/** 通用插件接口 */
export interface Plugin {
  /** 插件唯一标识 */
  id: string
  /** 显示名称 */
  name: string
  /** 插件类型 */
  kind: PluginKind
  /** 版本号 */
  version: string
}

/** 插件注册表 */
export interface PluginRegistry {
  /** 注册插件 */
  register(plugin: Plugin): void
  /** 按类型获取所有插件 */
  getByKind(kind: PluginKind): Plugin[]
  /** 按 ID 获取 */
  get(id: string): Plugin | undefined
  /** 获取认证插件（约定只有一个） */
  getAuthPlugin(): AuthPlugin | undefined
}
