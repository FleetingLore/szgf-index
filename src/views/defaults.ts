/**
 * 默认视图注册 — 在应用入口调用，注册内置的 3 个视图。
 * 添加新页面只需在此文件中加一条 register()。
 */

import { viewRegistry } from "./registry";
import Listing from "../pages/Listing";
import DocPage from "../pages/DocPage";
import AdminPage from "../pages/AdminPage";
import type { ViewProps } from "./registry";

/** 注册所有默认视图 */
export function registerDefaultViews() {
    // 文档列表
    viewRegistry.register({
        name: "listing",
        match: (path) => {
            if (path === "/" || path === "/listing") return "listing";
            return null;
        },
        component: () => {
            // ⚠️ Listing 需要额外 props（siteName, cats 等），这里用别名模式处理
            // 实际渲染时由 App.tsx 通过 viewRegistry 获取外部状态注入
            throw new Error("Listing view requires app state injection");
        },
    });

    // 管理面板（必须在 doc 之前注册，因为 /listing/lib/0 同时匹配两者）
    viewRegistry.register({
        name: "admin",
        match: (path) => {
            if (path === "/listing/lib/0") return "admin";
            return null;
        },
        component: () => {
            throw new Error("AdminPage view requires app state injection");
        },
    });

    // 文档详情
    viewRegistry.register({
        name: "doc",
        match: (path) => {
            const m = path.match(/^\/listing\/lib\/(\d+)$/);
            if (m && m[1] !== "0") return "doc";
            return null;
        },
        component: () => {
            throw new Error("DocPage view requires app state injection");
        },
    });
}
