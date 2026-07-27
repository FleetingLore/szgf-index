#!/usr/bin/env bash
set -euo pipefail
# sync-docs.sh - sync local docs to remote server

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
echo "  同步文档到服务器: $REMOTE_USER@$SERVER"
echo "=========================================="

# 检查本地 docs 目录
if [ ! -d "docs" ]; then
    echo "❌ 本地 docs 目录不存在"
    exit 1
fi

# 统计文件
DOC_COUNT=$(find docs -name "*.md" -type f | wc -l | tr -d ' ')
echo "📁 本地文档数量: $DOC_COUNT"

# 同步文档（保留服务器上的 docs.json）
echo "🔄 同步中..."
rsync -avz --delete \
    --exclude='.DS_Store' \
    docs/ "$REMOTE_USER@$SERVER:$REMOTE_PATH/docs/" -e "ssh -p $SSH_PORT"

# 同步 docs.json（保留服务器上的 IDs）
echo "📋 同步文档映射..."
scp -P "$SSH_PORT" config/docs.json "$REMOTE_USER@$SERVER:/tmp/docs.new.json"
ssh -p "$SSH_PORT" "$REMOTE_USER@$SERVER" << EOF
    set -euo pipefail
    cd "$REMOTE_PATH"

    # 合并文档映射（保留已有 ID）
    python3 << 'PY'
import json, os
old_path = 'config/docs.json'
new_path = '/tmp/docs.new.json'
old = {}
if os.path.exists(old_path):
    with open(old_path) as f:
        old = json.load(f)
with open(new_path) as f:
    new = json.load(f)
if 'next_id' in new:
    old['next_id'] = max(old.get('next_id', 1), new['next_id'])
old.setdefault('docs', {})
old['docs'].update(new.get('docs', {}))
with open(old_path, 'w') as f:
    json.dump(old, f, indent=2, ensure_ascii=False)
print('文档映射合并完成')
PY
    rm /tmp/docs.new.json || true
    chown -R $REMOTE_USER:$REMOTE_USER docs config/docs.json || true
    echo "文档同步完成"
EOF

echo ""
echo "✅ 同步完成！"
echo "   服务器文档已更新"
echo "   💡 运行 ./deploy.sh 重启服务（如需）"
