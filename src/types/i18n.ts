/**
 * i18n 国际化类型
 *
 * 翻译 key 规范：使用点分隔的层级命名，如 "doc.action.delete"
 * 所有面向用户的字符串都从语言包加载，代码中零硬编码。
 */

/** 翻译函数 */
export type TFunc = (key: string, params?: Record<string, string | number>) => string

/** 语言包：key → 翻译文本 */
export type LocalePack = Record<string, string>

/** 支持的语言代码 */
export type LocaleCode = string

/** i18n 上下文 */
export interface I18nContext {
  /** 当前语言 */
  locale: LocaleCode
  /** 翻译函数 */
  t: TFunc
  /** 切换语言 */
  setLocale: (locale: LocaleCode) => void
  /** 可用语言列表 */
  availableLocales: LocaleCode[]
}
