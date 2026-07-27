#!/usr/bin/env bash
# scripts/stop-dev.sh — 停止开发服务器

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

if [ -f "$ROOT_DIR/.dev_backend.pid" ]; then
  PID=$(cat "$ROOT_DIR/.dev_backend.pid")
  echo "停止后端 (PID: $PID)..."
  kill "$PID" 2>/dev/null || true
  rm -f "$ROOT_DIR/.dev_backend.pid"
  echo "✅ 已停止"
else
  echo "没有运行中的后端进程"
fi
