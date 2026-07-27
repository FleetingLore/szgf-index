#!/usr/bin/env bash
set -euo pipefail
# scripts/run-dev.sh — 启动后端 + 前端开发服务器

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

mkdir -p logs data docs config/i18n

trap 'echo "停止服务..."; if [ -f "$ROOT_DIR/.dev_backend.pid" ]; then kill "$(cat $ROOT_DIR/.dev_backend.pid)" 2>/dev/null || true; rm -f "$ROOT_DIR/.dev_backend.pid"; fi; exit' INT TERM EXIT

# 检查依赖
if ! command -v cargo >/dev/null 2>&1; then
  echo "❌ cargo 未安装 — https://rustup.rs/"
  exit 1
fi
if ! command -v npm >/dev/null 2>&1; then
  echo "❌ npm 未安装 — https://nodejs.org/"
  exit 1
fi

# 创建示例文档（如果 docs 目录为空）
if [ ! "$(ls -A docs/*.md 2>/dev/null)" ]; then
  echo "📝 创建示例文档..."
  cat > docs/welcome.md << 'EOF'
# 欢迎使用 Ducia

这是一个轻量级文档管理系统。

## 功能

- 📄 Markdown 文档渲染（支持数学公式 $E=mc^2$）
- 🔐 身份认证（序列码 / 用户登录）
- 🌍 多语言支持（中/英）
- 📦 插件化架构（存储/认证可替换）

## 开始

点击右上角上传按钮，选择 `.md` 文件上传你的第一篇文档。
EOF
fi

# 首次运行安装前端依赖
if [ ! -d "node_modules" ]; then
  echo "📦 安装前端依赖..."
  npm install
fi

echo "🚀 启动后端 (cargo run -p ducia-server)..."
cd backend/server
cargo run > "$ROOT_DIR/logs/backend.log" 2>&1 &
echo $! > "$ROOT_DIR/.dev_backend.pid"
cd "$ROOT_DIR"

# 等后端就绪
sleep 2
if curl -s http://127.0.0.1:3001/api/locales > /dev/null 2>&1; then
  echo "✅ 后端已就绪: http://127.0.0.1:3001"
else
  echo "⏳ 后端启动中... (查看 logs/backend.log)"
fi

echo "🌐 启动前端: http://localhost:5173"
npm run dev
