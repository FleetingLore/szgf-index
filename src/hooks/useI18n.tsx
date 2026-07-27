/**
 * useI18n — 前端国际化 hook
 *
 * 核心思路：
 * 1. 从后端 API 加载语言包资源文件
 * 2. 根据浏览器 Accept-Language / 用户选择解析 locale
 * 3. t('key') 查表翻译，代码中零硬编码字符串
 * 4. 翻译 key 缺失时 fallback 到默认语言 → key 本身
 */

import {
    useState,
    useEffect,
    useCallback,
    createContext,
    useContext,
} from "react";
import type { LocalePack, LocaleCode, TFunc } from "../types/i18n";

interface I18nState {
    /** 当前语言 */
    locale: LocaleCode;
    /** 翻译函数 */
    t: TFunc;
    /** 切换语言 */
    setLocale: (locale: LocaleCode) => void;
    /** 可用语言列表 */
    availableLocales: LocaleCode[];
    /** 是否加载中 */
    loading: boolean;
}

const I18nContext = createContext<I18nState>({
    locale: "zh-CN",
    t: (key: string) => key,
    setLocale: () => {},
    availableLocales: [],
    loading: true,
});

/** 翻译包缓存（避免重复请求） */
const packCache = new Map<LocaleCode, LocalePack>();

/**
 * 从浏览器偏好解析初始语言
 */
function resolveInitialLocale(available: LocaleCode[]): LocaleCode {
    if (!available.length) return "zh-CN";

    // 1. sessionStorage 中的用户选择
    const stored = sessionStorage.getItem("locale");
    if (stored && available.includes(stored)) return stored;

    // 2. 浏览器 Accept-Language
    const navLang = navigator.language;
    if (available.includes(navLang)) return navLang;
    // 短代码匹配（zh → zh-CN）
    const short = navLang.split("-")[0];
    const match = available.find((l) => l.startsWith(short));
    if (match) return match;

    // 3. 第一个可用语言
    return available[0];
}

export function I18nProvider({ children }: { children: React.ReactNode }) {
    const [locale, setLocaleState] = useState<LocaleCode>("zh-CN");
    const [availableLocales, setAvailableLocales] = useState<LocaleCode[]>([]);
    const [currentPack, setCurrentPack] = useState<LocalePack>({});
    const [loading, setLoading] = useState(true);

    // 加载可用语言列表
    useEffect(() => {
        fetch("/api/locales")
            .then((res) => res.json())
            .then((data) => {
                if (data.success) {
                    const codes = data.data.map(
                        (l: { code: string }) => l.code,
                    );
                    setAvailableLocales(codes);
                    const initial = resolveInitialLocale(codes);
                    setLocaleState(initial);
                }
                setLoading(false);
            })
            .catch(() => setLoading(false));
    }, []);

    // 加载当前语言包
    useEffect(() => {
        if (!locale) return;

        if (packCache.has(locale)) {
            setCurrentPack(packCache.get(locale)!);
            return;
        }

        fetch(`/api/locales/${locale}`)
            .then((res) => res.json())
            .then((data) => {
                if (data.success) {
                    packCache.set(locale, data.data);
                    setCurrentPack(data.data);
                }
            })
            .catch((err) =>
                console.error("Failed to load locale pack:", locale, err),
            );
    }, [locale]);

    const t: TFunc = useCallback(
        (key: string, params?: Record<string, string | number>) => {
            let text = currentPack[key] ?? key;
            if (params) {
                for (const [k, v] of Object.entries(params)) {
                    text = text.replace(`{{${k}}}`, String(v));
                }
            }
            return text;
        },
        [currentPack],
    );

    const setLocale = useCallback((newLocale: LocaleCode) => {
        sessionStorage.setItem("locale", newLocale);
        setLocaleState(newLocale);
    }, []);

    return (
        <I18nContext.Provider
            value={{ locale, t, setLocale, availableLocales, loading }}
        >
            {children}
        </I18nContext.Provider>
    );
}

/** 在任意组件中使用翻译 */
export function useI18n(): I18nState {
    return useContext(I18nContext);
}
