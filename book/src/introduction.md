# 简介

**Ducia Listing Framework** 是一个轻量级、插件化的文档管理系统。

## 设计理念

- **框架本体极简**：只定义接口契约，不绑定具体实现
- **一切皆插件**：存储、认证、可见性——通过插件注入
- **零硬编码**：所有字符串从语言包资源文件加载
- **动态角色**：不预设 admin/editor/viewer，部署者自定义

## 技术栈

| 层 | 技术 |
|----|------|
| 后端 | Rust + Actix-web 4 |
| 前端 | React 18 + TypeScript + Vite |
| 数据库（可选） | SQLite (rusqlite) |
| 身份认证 | JWT + bcrypt（内置），或外部服务 |
| 文档渲染 | pulldown-cmark (WASM) / react-markdown |
| 国际化 | JSON 语言包 + 请求级语言解析 |
| 构建工具 | Cargo + npm + wasm-pack |

## 项目结构

```text
ducia-listing-framework/
├── config/                 # 所有配置文件
│   ├── auth.json           #   JWT 密钥
│   ├── roles.json          #   动态角色定义
│   ├── sequence.json       #   序列码
│   ├── settings.json       #   功能开关
│   └── i18n/               #   语言资源文件
├── backend/
│   ├── ducia-core/         # 框架核心（trait 定义）
│   ├── plugins/            # 官方插件
│   │   ├── auth-simple/    #   序列码认证
│   │   ├── auth-db/        #   JWT + 数据库认证
│   │   ├── storage-fs/     #   文件系统存储
│   │   └── storage-sqlite/ #   SQLite 存储
│   ├── server/             # 二进制入口
│   └── wasm/               # WASM 共享逻辑
├── src/                    # 前端源码
│   ├── types/              #   TypeScript 类型
│   ├── hooks/              #   React Hooks
│   ├── components/         #   UI 组件
│   └── pages/              #   页面
└── book/                   # 本书源码
```
