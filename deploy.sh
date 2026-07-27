#!/usr/bin/env bash
set -euo pipefail

# Load deployment config if present
CONF_FILE="scripts/deploy.conf"
if [ -f "$CONF_FILE" ]; then
  . "$CONF_FILE"
fi

# Defaults
SERVER="${SERVER:-175.178.183.209}"
REMOTE_USER="${REMOTE_USER:-root}"
REMOTE_PATH="${REMOTE_PATH:-/www/wwwroot/ducia}"
SSH_PORT="${SSH_PORT:-22}"
DOCS_SYNC=${1:-"no"}

BIN_NAME="ducia"

echo "=========================================="
echo "  部署到: $REMOTE_USER@$SERVER:$REMOTE_PATH"
echo "=========================================="

# 1. 构建前端
echo "[1/5] 构建前端..."
rm -rf dist
npm run build

# 2. 打包（包含所有 config，排除 docs.json）
echo "[2/5] 打包文件..."
EXCLUDE="--exclude=node_modules --exclude=.git --exclude=config/docs.json"

if [ "$DOCS_SYNC" = "docs" ]; then
    echo "  📁 包含文档目录（将覆盖服务器文档）"
    TAR_EXTRA="docs"
else
    echo "  📁 排除文档目录（保留服务器文档）"
    TAR_EXTRA=""
fi

TMP_TAR="/tmp/ducia-deploy.tar.gz"
tar -czf "$TMP_TAR" \
    $EXCLUDE \
    dist/ src/ public/ index.html package.json package-lock.json vite.config.js backend/ config/ $TAR_EXTRA

# 3. 上传
echo "[3/5] 上传到服务器..."
scp -P "$SSH_PORT" "$TMP_TAR" "$REMOTE_USER@$SERVER:/tmp/"

# 4. 服务器端更新
echo "[4/5] 服务器端更新..."
ssh -p "$SSH_PORT" "$REMOTE_USER@$SERVER" << 'ENDSSH'
    set -e
    REMOTE_PATH="/www/wwwroot/ducia"
    BIN_NAME="ducia"
    
    cd "$REMOTE_PATH"
    
    # 备份 docs.json（用户数据）
    if [ -f config/docs.json ]; then
        cp config/docs.json /tmp/docs.json.bak
    fi
    
    # 解压（会覆盖 config 目录下的所有文件除了 docs.json）
    tar -xzf /tmp/ducia-deploy.tar.gz --overwrite
    
    # 恢复 docs.json
    if [ -f /tmp/docs.json.bak ]; then
        mv /tmp/docs.json.bak config/docs.json
    fi
    
    # 确保目录存在
    mkdir -p docs
    
    # 编译后端
    echo "  编译后端..."
    cd backend
    cargo build --release --quiet 2>/dev/null || cargo build --release
    
    # 停止旧进程
    pkill -f "$BIN_NAME" 2>/dev/null || true
    sleep 1
    
    # 启动新进程
    nohup ./target/release/"$BIN_NAME" > /tmp/ducia.log 2>&1 &
    
    rm -f /tmp/ducia-deploy.tar.gz
    
    echo "  后端已启动"
ENDSSH

# 5. 重载 Nginx
echo "[5/5] 重载 Nginx..."
ssh -p "$SSH_PORT" "$REMOTE_USER@$SERVER" "nginx -s reload 2>/dev/null || systemctl reload nginx 2>/dev/null || true"

echo ""
echo "✅ 部署完成！"
echo "   访问: http://local.ducia.site"
echo ""
echo "💡 提示:"
echo "   只更新代码: ./deploy.sh"
echo "   同时更新文档: ./deploy.sh docs"
