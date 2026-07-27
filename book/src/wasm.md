# WASM 构建

Ducia 使用 WebAssembly (WASM) 在前后端之间共享核心逻辑，确保 Markdown 渲染和序列码验证行为一致。

---

## 概述

WASM 模块位于 `backend/wasm/`，是一个独立的 Rust crate (`ducia-wasm`)，通过 [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) 编译为 WebAssembly，供前端 JavaScript 调用。

### 设计动机

| 场景 | 问题 | WASM 方案 |
|------|------|-----------|
| Markdown 渲染 | 后端使用 pulldown-cmark，前端使用 react-markdown；不同引擎可能产生不同的 HTML 输出 | 前后端共享同一个 Rust 渲染引擎 |
| 序列码验证 | 验证逻辑需要在前后端一致，避免客户端与服务端判断分歧 | 编译为 WASM，前端直接调用同一段 Rust 代码 |
| 格式检测 | `detectFormat()` 逻辑分散在 TS 和 Rust 两端，维护成本高 | WASM 统一实现，TS 端做 fallback |

---

## 模块结构

```
backend/wasm/
├── Cargo.toml          # crate 清单
├── Cargo.lock
├── src/
│   └── lib.rs          # WASM 导出函数
├── pkg/                # wasm-pack 构建产物（自动生成）
│   ├── ducia_wasm.js
│   ├── ducia_wasm_bg.wasm
│   ├── ducia_wasm.d.ts
│   └── package.json
└── target/             # Rust 编译缓存
```

### Cargo.toml

```toml
[package]
name = "ducia-wasm"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
pulldown-cmark = "0.12"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

关键配置：

- `crate-type = ["cdylib"]`：编译为 C 动态库（WASM 要求的格式）
- `pulldown-cmark`：与后端 server 使用相同的 Markdown 解析器
- `wasm-bindgen`：生成 JS 绑定和 TypeScript 类型声明

---

## 导出函数

`src/lib.rs` 通过 `#[wasm_bindgen]` 标记导出四个函数：

### markdown_to_html

```rust
#[wasm_bindgen]
pub fn markdown_to_html(markdown_input: &str) -> String {
    let parser = Parser::new(markdown_input);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
```

将 Markdown 文本渲染为 HTML。使用与后端相同的 `pulldown-cmark` 引擎，保证输出一致。

### check_sequence

```rust
#[wasm_bindgen]
pub fn check_sequence(input: Vec<usize>, expected: Vec<usize>) -> bool {
    if expected.is_empty() || input.len() < expected.len() {
        return false;
    }
    let offset = input.len() - expected.len();
    input[offset..] == expected[..]
}
```

检查用户点击的序列是否匹配目标序列。仅比较 `input` 末尾与 `expected` 长度相等的部分。

### check_sequence_prefix

```rust
#[wasm_bindgen]
pub fn check_sequence_prefix(input: Vec<usize>, expected: Vec<usize>) -> bool {
    if expected.is_empty() {
        return true;
    }
    if input.len() > expected.len() {
        return false;
    }
    expected.starts_with(&input)
}
```

检查用户输入的序列是否为预期序列的前缀。用于在输入过程中提供实时反馈。

### detect_format

```rust
#[wasm_bindgen]
pub fn detect_format(filename: &str) -> String {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "md" | "markdown" => "markdown".into(),
        "txt" => "plaintext".into(),
        "html" | "htm" => "html".into(),
        _ => "markdown".into(),
    }
}
```

从文件扩展名检测文档格式。与 `src/types/doc.ts` 中的 `detectFormat()` 逻辑一致。

---

## 构建流程

### 1. 编译 WASM

```bash
cd backend/wasm
wasm-pack build --target web
```

`--target web` 指定生成适用于浏览器直接加载的格式（不使用 bundler）。

编译成功后，产物写入 `pkg/` 目录：

| 文件 | 用途 |
|------|------|
| `ducia_wasm.js` | JS 胶水代码：加载 WASM、导出函数包装 |
| `ducia_wasm_bg.wasm` | WebAssembly 二进制 |
| `ducia_wasm.d.ts` | TypeScript 类型声明 |
| `package.json` | npm 包清单（可用于 npm link） |

### 2. 复制到前端

前端需要两份 WASM 产物：

- **JS 胶水代码** → `src/wasm/`（供 `import` 引用）
- **.wasm 二进制** → `public/`（浏览器运行时通过 fetch 加载）

```bash
# 从项目根目录执行
cp backend/wasm/pkg/ducia_wasm.js      src/wasm/
cp backend/wasm/pkg/ducia_wasm_bg.wasm public/
cp backend/wasm/pkg/ducia_wasm.d.ts    src/wasm/
```

> `public/` 是 Vite 的静态资源目录，构建时直接复制到 `dist/`。WASM 二进制文件必须位于 `public/`，因为浏览器通过相对路径加载 `.wasm` 文件。

### 3. 在前端使用

```typescript
import init, { markdown_to_html, check_sequence } from '../wasm/ducia_wasm'

// 初始化 WASM 模块（仅需一次）
await init()

// 调用函数
const html = markdown_to_html('# Hello World')
const valid = check_sequence([1, 2, 3, 2, 3, 4], [2, 3, 4])
```

---

## 与前端 Markdown 渲染的关系

Ducia 前端同时支持两种渲染路径：

| 路径 | 引擎 | 使用场景 |
|------|------|----------|
| WASM 渲染 | pulldown-cmark (Rust) | 需要与后端输出完全一致的场景 |
| React 渲染 | react-markdown + remark/rehype 插件 | 默认文档阅读页 |

当前渲染通过 `FormatRegistry` 统一管理：Markdown 渲染器在 `src/rendering/defaults.tsx` 中注册，作为多种可能格式之一。`FormatRegistry` 模式使得添加新格式渲染器变得简单——只需实现对应组件并注册即可。WASM 渲染路径作为备选，未来可用于：

- 渲染预览（本地即可获取与后端一致的 HTML）
- 构建时预渲染（SSG 场景）
- 离线模式下的文档渲染

两种路径的关键区别：

| 特性 | react-markdown | WASM (pulldown-cmark) |
|------|----------------|----------------------|
| 数学公式 | ✅ (rehype-katex) | ❌ (需额外处理) |
| GFM | ✅ (remark-gfm) | ❌ (需启用 feature) |
| React 组件 | ✅ | ❌ (纯 HTML 字符串) |
| 与后端一致 | ⚠️ (近似) | ✅ (完全一致) |

---

## 完整构建示例

从零开始构建 WASM 并集成到前端：

```bash
# 1. 安装 wasm-pack（如未安装）
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# 2. 编译 WASM
cd backend/wasm
wasm-pack build --target web

# 3. 回到项目根目录，复制产物
cd ../..
cp backend/wasm/pkg/ducia_wasm.js      src/wasm/
cp backend/wasm/pkg/ducia_wasm_bg.wasm public/
cp backend/wasm/pkg/ducia_wasm.d.ts    src/wasm/

# 4. 验证
ls -la src/wasm/ducia_wasm.js
ls -la public/ducia_wasm_bg.wasm

# 5. 启动前端测试
npm run dev
```

---

## 注意事项

- **WASM 文件大小**：`ducia_wasm_bg.wasm` 约 200KB，首次加载需网络传输。生产环境建议启用 HTTP 压缩（gzip/brotli）
- **跨域限制**：WASM 二进制通过 `fetch` 加载，需确保与页面同源或配置 CORS
- **版本同步**：修改 `lib.rs` 后必须重新构建并复制产物，否则前后端逻辑不一致
- **wasm-pack 版本**：推荐使用最新稳定版。可通过 `wasm-pack --version` 检查
- **Node.js 兼容**：当前仅构建 `--target web`。如果需要在 Node.js 端测试 WASM 函数，可使用 `--target nodejs`
