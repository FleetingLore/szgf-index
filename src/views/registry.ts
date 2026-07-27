/**
 * 视图注册表 — 页面不再是写死的 3 个，而是通过注册表动态匹配 URL。
 * 未来加新页面只需 `register()` 一条调用。
 */

import type { ComponentType } from 'react'

export interface ViewDefinition {
  /** 视图名称（调试用） */
  name: string
  /** URL 匹配函数：返回 true 表示此视图应渲染 */
  match: (pathname: string) => string | null
  /** 视图组件 */
  component: ComponentType<ViewProps>
  /** 需要的权限（为空表示无需认证） */
  requiredPermission?: string
}

/** 传递给视图组件的 props */
export interface ViewProps {
  /** 从 URL 中提取的参数，如文档 ID */
  params: Record<string, string>
}

/** 视图注册表 */
class ViewRegistry {
  private views: ViewDefinition[] = []

  /** 注册一个视图 */
  register(view: ViewDefinition) {
    this.views.push(view)
  }

  /** 根据 pathname 查找匹配的视图 */
  resolve(pathname: string): ViewDefinition | null {
    for (const view of this.views) {
      const matched = view.match(pathname)
      if (matched !== null) {
        return view
      }
    }
    return null
  }

  /** 获取所有已注册的视图 */
  all(): ViewDefinition[] {
    return [...this.views]
  }
}

/** 全局单例 */
export const viewRegistry = new ViewRegistry()
