#!/usr/bin/env bash
set -euo pipefail

# ═══════════════════════════════════════════
#  部署脚本 — szgf.ducia.site
# ═══════════════════════════════════════════

SERVER="${DEPLOY_SERVER:-160.202.47.107}"
REMOTE_USER="${DEPLOY_USER:-root}"
REMOTE_PATH="${DEPLOY_PATH:-/var/www/szgf}"
SSH_PORT="${DEPLOY_PORT:-22}"
DOMAIN="szgf.ducia.site"
BIN_NAME="ducia-server"
MODE="${1:-code}"

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; NC='\033[0m'
log() { echo -e "${CYAN}[*]${NC} $*"; }
ok()  { echo -e "${GREEN}[✓]${NC} $*"; }
err() { echo -e "${RED}[✗]${NC} $*"; }

usage() {
    echo "用法: ./deploy.sh [code|docs|full|nginx]"
    exit 0
}
[[ "${1:-}" == "-h" || "${1:-}" == "--help" ]] && usage

if [ "$MODE" == "nginx" ]; then
    log "写入 Nginx 配置"
    ssh -p "$SSH_PORT" "${REMOTE_USER}@${SERVER}" "cat > /etc/nginx/conf.d/szgf.conf" << 'NGINXEOF'
server {
    listen 80;
    server_name szgf.ducia.site;
    location / {
        root /var/www/szgf/dist;
        try_files $uri $uri/ /index.html;
        index index.html;
    }
    location /api/ {
        proxy_pass http://127.0.0.1:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
NGINXEOF
    ssh -p "$SSH_PORT" "${REMOTE_USER}@${SERVER}" "nginx -t && nginx -s reload"
    ok "Nginx 已重载"
    exit 0
fi

# ═══ 1. 构建前端 ═══
log "构建前端..."
rm -rf dist
npm run build
ok "前端构建完成"

# ═══ 2. 打包 ═══
log "打包..."

EXCLUDE=(
    --exclude=node_modules
    --exclude=.git
    --exclude=.github
    --exclude=backend/target
    --exclude=nohup.out
    --exclude='*.tar.gz'
)

case "$MODE" in
    code)
        EXCLUDE+=(--exclude=docs --exclude=config/docs.json --exclude=data)
        TAR_EXTRA=""
        ;;
    docs)
        EXCLUDE+=(--exclude=config/docs.json --exclude=data)
        TAR_EXTRA="docs"
        ;;
    full)
        TAR_EXTRA="docs data config"
        ;;
    *) err "未知模式: $MODE"; usage ;;
esac

TMP_TAR="/tmp/szgf-deploy.tar.gz"
tar -czf "$TMP_TAR" "${EXCLUDE[@]}" \
    dist/ config/ backend/ $TAR_EXTRA

ok "打包完成: $(du -h "$TMP_TAR" | cut -f1)"

# ═══ 3. 上传 ═══
log "上传..."
scp -P "$SSH_PORT" "$TMP_TAR" "${REMOTE_USER}@${SERVER}:/tmp/"
ok "上传完成"

# ═══ 4. 服务器端 ═══
log "服务器端更新..."
ssh -p "$SSH_PORT" "${REMOTE_USER}@${SERVER}" << 'ENDSSH'
set -e
cd /var/www/szgf

# 备份用户数据（仅 docs.json）
[ -f config/docs.json ] && cp config/docs.json /tmp/docs.json.bak || true

# 解压
tar -xzf /tmp/szgf-deploy.tar.gz

# 恢复用户数据
[ -f /tmp/docs.json.bak ] && mv /tmp/docs.json.bak config/docs.json || true
mkdir -p docs data

# 编译后端
echo "  编译..."
source "$HOME/.cargo/env" 2>/dev/null || true
cd backend && cargo build --release --quiet 2>/dev/null || cargo build --release
cd ..

# 重启
pkill -f ducia-server 2>/dev/null || true
sleep 1
nohup ./backend/target/release/ducia-server > /tmp/szgf-server.log 2>&1 &

# 清理
rm -f /tmp/szgf-deploy.tar.gz /tmp/docs.json.bak
ENDSSH

ok "服务器端更新完成"

# ═══ 5. 重载 Nginx ═══
ssh -p "$SSH_PORT" "${REMOTE_USER}@${SERVER}" "nginx -t && nginx -s reload" 2>/dev/null || true

rm -f "$TMP_TAR"
ok "部署完成 → http://${DOMAIN}"
