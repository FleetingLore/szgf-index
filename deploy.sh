#!/usr/bin/env bash
set -euo pipefail

# ═══════════════════════════════════════════
#  部署脚本 — szgf.ducia.site
#  服务器: 160.202.47.107 (Ubuntu / Nginx)
# ═══════════════════════════════════════════

# ---- 服务器配置 ----
SERVER="${DEPLOY_SERVER:-160.202.47.107}"
REMOTE_USER="${DEPLOY_USER:-root}"
REMOTE_PATH="${DEPLOY_PATH:-/var/www/szgf}"
SSH_PORT="${DEPLOY_PORT:-22}"
NGINX_CONF="/etc/nginx/conf.d/projects/szgf.conf"
DOMAIN="szgf.ducia.site"
BIN_NAME="ducia-server"

# ---- 模式选择 ----
MODE="${1:-code}"   # code | docs | full | nginx

# ---- 颜色 ----
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log()  { echo -e "${CYAN}[*]${NC} $*"; }
ok()   { echo -e "${GREEN}[✓]${NC} $*"; }
warn() { echo -e "${YELLOW}[!]${NC} $*"; }
err()  { echo -e "${RED}[✗]${NC} $*"; }

# ---- 帮助 ----
usage() {
    echo "用法: ./deploy.sh [code|docs|full|nginx]"
    echo ""
    echo "  code    仅更新代码（默认，保留服务器上的文档和数据）"
    echo "  docs    更新代码并覆盖文档"
    echo "  full    完全部署（代码 + 文档 + 所有配置）"
    echo "  nginx   仅更新 Nginx 配置（首次部署时使用）"
    echo ""
    echo "环境变量（可选）："
    echo "  DEPLOY_SERVER  服务器 IP（默认 160.202.47.107）"
    echo "  DEPLOY_USER    SSH 用户（默认 root）"
    echo "  DEPLOY_PATH    远程路径（默认 /var/www/szgf）"
    echo "  DEPLOY_PORT    SSH 端口（默认 22）"
    exit 0
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    usage
fi

# ---- nginx only ----
if [ "$MODE" == "nginx" ]; then
    log "写入 Nginx 配置 → ${NGINX_CONF}"
    ssh -p "$SSH_PORT" "${REMOTE_USER}@${SERVER}" "cat > $NGINX_CONF" << 'NGINXEOF'
# szgf.ducia.site
server {
    listen 80;
    server_name szgf.ducia.site;

    # 前端静态文件
    location / {
        root /var/www/szgf/dist;
        try_files $uri $uri/ /index.html;
        index index.html;
    }

    # API 代理到后端
    location /api/ {
        proxy_pass http://127.0.0.1:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
NGINXEOF

    log "测试并重载 Nginx..."
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
log "打包项目文件..."

# 基础排除项
EXCLUDE=(
    --exclude=node_modules
    --exclude=.git
    --exclude=.github
    --exclude=target
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
    *)
        err "未知模式: $MODE"
        usage
        ;;
esac

TMP_TAR="/tmp/szgf-deploy.tar.gz"
tar -czf "$TMP_TAR" \
    "${EXCLUDE[@]}" \
    dist/ src/ public/ index.html \
    package.json package-lock.json vite.config.js tsconfig.json \
    backend/ config/ scripts/ $TAR_EXTRA

ok "打包完成: $(du -h "$TMP_TAR" | cut -f1)"

# ═══ 3. 上传 ═══
log "上传到服务器..."
scp -P "$SSH_PORT" "$TMP_TAR" "${REMOTE_USER}@${SERVER}:/tmp/"
ok "上传完成"

# ═══ 4. 服务器端操作 ═══
log "服务器端更新..."

ssh -p "$SSH_PORT" "${REMOTE_USER}@${SERVER}" << ENDSSH
set -e

REMOTE_PATH="$REMOTE_PATH"
BIN_NAME="$BIN_NAME"

# 首次部署创建目录
mkdir -p "\$REMOTE_PATH"
cd "\$REMOTE_PATH"

# 备份用户数据
echo "  备份用户数据..."
for f in docs.json auth.json sequence.json roles.json settings.json site.json; do
    [ -f "config/\$f" ] && cp "config/\$f" "/tmp/\$f.bak" || true
done

# 解压新文件
echo "  解压文件..."
tar -xzf /tmp/szgf-deploy.tar.gz 2>/dev/null

# 恢复用户数据
echo "  恢复用户数据..."
for f in docs.json auth.json sequence.json roles.json settings.json site.json; do
    [ -f "/tmp/\$f.bak" ] && mv "/tmp/\$f.bak" "config/\$f" || true
done

# 确保目录存在
mkdir -p docs data

# 编译后端（首次自动安装 Rust）
echo "  编译后端..."
if ! command -v cargo &>/dev/null; then
    echo "  安装 Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    export PATH="\$HOME/.cargo/bin:\$PATH"
fi
cd backend
cargo build --release --quiet 2>/dev/null || cargo build --release
cd ..

# 重启服务
echo "  重启服务..."
pkill -f "\$BIN_NAME" 2>/dev/null || true
sleep 1

nohup ./backend/target/release/"\$BIN_NAME" > /tmp/szgf-server.log 2>&1 &
echo "  PID: \$!"

# 清理
rm -f /tmp/szgf-deploy.tar.gz /tmp/docs.json.bak /tmp/auth.json.bak \
      /tmp/sequence.json.bak /tmp/roles.json.bak /tmp/settings.json.bak /tmp/site.json.bak
ENDSSH

ok "服务器端更新完成"

# ═══ 5. 重载 Nginx ═══
log "重载 Nginx..."
ssh -p "$SSH_PORT" "${REMOTE_USER}@${SERVER}" "nginx -t && nginx -s reload" 2>/dev/null \
    || warn "Nginx 重载失败，请手动检查"
ok "Nginx 已重载"

# ═══ 清理本地 ═══
rm -f "$TMP_TAR"

# ═══ 完成 ═══
ok "部署完成 → http://${DOMAIN}"
