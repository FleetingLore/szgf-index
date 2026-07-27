# 前端架构

## 技术栈

| 层 | 技术 |
|----|------|
| 框架 | React 18 |
| 语言 | TypeScript (strict mode) |
| 构建 | Vite 5 + @vitejs/plugin-react |
| 样式 | CSS Modules + 设计令牌 (Design Tokens) |
| Markdown 渲染 | react-markdown + remark-gfm + remark-math + rehype-katex |
| 国际化 | 自研 useI18n hook + JSON 语言包 |
| 数学公式 | KaTeX (LaTeX 渲染) |

## 项目结构

```text
src/
├── main.tsx                  # 入口
├── App.tsx                   # 路由分发
├── components/               # UI 组件
│   ├── Header.tsx
│   ├── DocHeader.tsx
│   ├── DocItem.tsx
│   ├── DocFooter.tsx
│   └── DeprecatedBanner.tsx
├── pages/                    # 页面
│   ├── Listing.tsx
│   ├── DocPage.tsx
│   └── AdminPage.tsx
├── hooks/                    # Hooks
│   ├── useAdmin.ts
│   ├── useCats.ts
│   ├── useDoc.ts
│   ├── useI18n.tsx
│   ├── useStickyHeader.ts
│   └── useUpload.ts
├── types/                    # TypeScript 类型
├── utils/                    # 工具函数
├── styles/                   # 全局样式
├── views/                    # 视图注册表
├── rendering/                # 格式注册表
└── wasm/                     # WASM 模块
```

## 路由设计

Ducia 是一个**单页应用 (SPA)**，不使用 React Router，而是直接读取 `location.pathname` 进行路由分发：

| 路径 | 视图组件 | 说明 |
|------|----------|------|
| `/` | `Listing` | 文档列表首页 |
| `/listing` | `Listing` | 同上（别名） |
| `/listing/lib/{id}` | `DocPage` | 文档阅读页，`id` 为文档 ID |
| `/listing/lib/0` | `AdminPage` | 管理员认证入口（序列码输入） |

路由逻辑集中在 `App.tsx` 的 `useEffect` 中。切换页面通过 `location.href` 整体跳转实现，不保留客户端历史栈。

## 入口文件

`src/main.tsx` 职责单一：

1. 引入全局样式 (`tokens.css`, `global.css`, `markdown.css`)
2. 用 `I18nProvider` 包裹整个应用
3. 渲染 `App` 到 `#root`

```typescript
ReactDOM.createRoot(document.getElementById("root")!).render(
    <I18nProvider>
        <App />
    </I18nProvider>,
);
```

## 核心组件

### App.tsx

应用状态机，维护 `view` 状态 (`'loading' | 'listing' | 'doc' | 'admin'`)，根据 URL 决定渲染哪个页面：

- **加载态**：`adminLoading` 或 `catsLoading` 为 `true` 时渲染空 `<div>`
- **列表页**：渲染 `Listing`，传入 `siteName`、文档列表、管理员状态、上传回调
- **文档页**：先从 `prefetch` 缓存读取文档，立即渲染骨架（标题为 `...`），随后异步拉取完整内容
- **管理页**：渲染 `AdminPage`，成功后创建 session 并跳转首页

### Header / DocHeader

两个顶栏组件使用同一 CSS Module (`MenuBar.module.css`)，通过 `useStickyHeader` 实现吸顶效果：

- 向上滚动 → 顶栏隐藏 (`translateY: -height`)
- 向下滚动 → 顶栏显示 (`translateY: 0`)
- 滚动到顶部以上 → 固定定位 (`position: sticky`)

### DocItem

文档列表项组件，在 `onMouseEnter` 时触发预加载缓存：

```typescript
onMouseEnter={() => prefetchDoc(cat.id)}
```

### DocFooter

文档底部操作栏，管理员可见"标记弃用"和"删除"按钮。删除操作使用**二次确认**机制：首次点击显示"确认删除"，3 秒后自动恢复。

### DeprecatedBanner

弃用文档的黄色警告横幅，提供"恢复"按钮。仅在文档 `deprecated: true` 时显示。

## 状态管理

不使用 Redux、Zustand 等状态管理库，**全部状态通过 React 内置 Hooks 管理**：

| Hook | 职责 | 状态 |
|------|------|------|
| `useCats` | 文档列表 | `cats[]`, `siteName`, `loading` |
| `useAdmin` | 管理员认证 | `isAdmin`, `loading` |
| `useI18n` | 国际化翻译 | `locale`, `t()`, `availableLocales` |
| `useDoc` | 单文档操作 | `currentDoc`, `loading` |
| `useStickyHeader` | 吸顶逻辑 | `isSticky`, `translateY` |
| `useUpload` | 文件上传 | `upload()` |

组件间通过 props 传递数据和回调，无需全局 store。

## 预加载缓存 (`prefetch.ts`)

基于 `Map<string, DocFull | Promise<DocFull | null>>` 的内存缓存：

```typescript
const cache = new Map<string, DocFull | Promise<DocFull | null>>()
```

- `prefetchDoc(id)`: 异步拉取文档 API 并缓存。若已缓存则直接返回；若正在请求中，返回 Promise（避免重复请求）
- `getCachedDoc(id)`: 同步获取已完成加载的文档。只返回非 Promise 的条目，确保拿到的是确定数据
- `clearCache()`: 清空缓存

缓存策略：列表页首次加载时，预加载前 5 篇文档；鼠标悬停 `DocItem` 时触发 `prefetchDoc`。

## 样式体系

采用 **CSS Modules + 设计令牌** 的两层架构：

### 设计令牌 (`tokens.css`)

定义全局 CSS 自定义属性：

```css
:root {
  --color-bg: #ffffff;
  --color-text: #1a1a1a;
  --color-accent: #0969da;
  --color-border: #d0d7de;
  --font-mono: 'SF Mono', monospace;
  --radius: 6px;
  /* ... */
}
```

深色模式下通过 `[data-theme="dark"]` 覆盖变量值。

### CSS Modules

每个组件有独立的 `.module.css` 文件，通过 `styles.xxx` 引用，样式隔离无冲突。

全局样式单独在 `styles/` 目录：

- `global.css`: 重置样式、基础排版、布局
- `markdown.css`: Markdown 渲染内容样式（标题、表格、代码块、引用块等）

## 文档渲染（FormatRegistry）

文档渲染不再硬编码 react-markdown，而是通过 **FormatRegistry** 模式，根据文件扩展名/格式动态选择渲染器：

- **`src/rendering/registry.ts`** — `FormatRegistry` 类，提供 `register()` 注册格式、`resolveByFilename()` 根据文件名匹配渲染器
- **`src/rendering/defaults.tsx`** — 默认注册 `Markdown` 和 `PlainText` 两种格式

Markdown 是默认注册的格式之一（`isDefault: true`），其渲染基于 [react-markdown](https://github.com/remarkjs/react-markdown)，插件栈：

| 插件 | 功能 |
|------|------|
| `remark-gfm` | GitHub Flavored Markdown（表格、任务列表、删除线） |
| `remark-math` | 数学公式语法支持（`$...$` 行内、`$$...$$` 块级） |
| `rehype-katex` | 将数学 AST 节点渲染为 KaTeX HTML |

示例渲染效果：

- 行内公式：`$E=mc^2$` → KaTeX 渲染
- 块级公式：`$$\int_0^\infty e^{-x^2} dx$$` → KaTeX 渲染
- GFM 表格、任务列表等由 remark-gfm 处理

KaTeX CSS 从 npm 包 `katex` 引入。

> 通过 `formatRegistry.register({format, name, extensions, component})` 可注册新的渲染格式（如 AsciiDoc、reStructuredText 等）。`DocPage` 在渲染时调用 `formatRegistry.resolveByFilename(doc.file)` 自动匹配对应渲染器。

## Vite 开发代理

Vite 开发服务器配置了 API 代理，将 `/api` 请求转发到后端：

```js
// vite.config.js
server: {
  proxy: {
    '/api': 'http://127.0.0.1:3001'
  }
}
```

前端开发运行在 `localhost:5173`，所有 `/api/*` 请求自动代理到后端 `127.0.0.1:3001`，避免跨域问题。
