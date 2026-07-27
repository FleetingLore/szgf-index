#!/usr/bin/env bash
set -euo pipefail
# fetch-docs.sh - pull docs from remote server

# Load deployment config if present
CONF_FILE="scripts/deploy.conf"
if [ -f "$CONF_FILE" ]; then
	# shellcheck source=/dev/null
	. "$CONF_FILE"
fi

SERVER="${SERVER:-175.178.183.209}"
REMOTE_USER="${REMOTE_USER:-root}"
REMOTE_PATH="${REMOTE_PATH:-/root/ducia-local}"
SSH_PORT="${SSH_PORT:-22}"

echo "=========================================="
echo "  从服务器拉取文档: $REMOTE_USER@$SERVER"
echo "=========================================="

# 创建目录
mkdir -p docs config

# 拉取文档文件
echo "📁 拉取 .md 文件..."
rsync -avz -e "ssh -p $SSH_PORT" "$REMOTE_USER@$SERVER:$REMOTE_PATH/docs/" docs/

# 拉取文档映射
echo "📋 拉取文档映射..."
scp -P "$SSH_PORT" "$REMOTE_USER@$SERVER:$REMOTE_PATH/config/docs.json" config/docs.json

# 统计
DOC_COUNT=$(find docs -name "*.md" -type f | wc -l | tr -d ' ')
echo ""
echo "✅ 拉取完成！本地文档数量: $DOC_COUNT"
