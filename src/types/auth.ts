/**
 * 身份与认证相关类型
 *
 * 框架本体不预设角色，角色由部署者通过 config/roles.json 定义。
 * 这里的类型只描述框架需要的最少信息。
 */

/** 权限字符串，格式 "resource:action"，如 "doc:read" */
export type Permission = string

/** 用户身份（从认证插件获取） */
export interface Identity {
  id: string
  /** 拥有的角色列表 */
  roles: string[]
  /** 直接授予的权限列表 */
  permissions: Permission[]
  /** 插件可自由扩展的元数据 */
  metadata: Record<string, string>
}

/**
 * 认证插件的接口契约
 *
 * 每个认证插件需提供：
 * - 如何从请求中提取身份
 * - 前端需要渲染的 UI 组件（登录表单、管理面板等）
 */
export interface AuthPlugin {
  name: string
  /** 检查当前 session 是否有效 */
  checkSession: () => Promise<Identity | null>
  /** 销毁 session */
  destroySession: () => Promise<void>
  /** 前端渲染的登录组件（React 组件类型） */
  LoginComponent?: React.ComponentType<{ onSuccess: (identity: Identity) => void }>
  /** 管理面板中额外渲染的内容 */
  AdminPanelComponent?: React.ComponentType
}

/** 用于 useAuth hook 的返回值 */
export interface AuthState {
  /** 当前用户身份，未登录为 null */
  identity: Identity | null
  /** 是否正在检查认证状态 */
  loading: boolean
  /** 是否拥有某个权限 */
  can: (permission: Permission) => boolean
  /** 是否拥有某个角色 */
  hasRole: (role: string) => boolean
  /** 创建 session（登录回调） */
  createSession: (identity: Identity) => void
  /** 销毁 session */
  destroySession: () => Promise<void>
}
