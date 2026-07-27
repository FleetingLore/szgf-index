# 服务器部署

本章以**腾讯云轻量应用服务器**为例，介绍如何将 Ducia 部署到公网。

## 前置准备

- 一台云服务器（本文使用腾讯云轻量应用服务器，2C2G，Ubuntu 22.04）
- 一个域名（可选，用于配置 HTTPS）
- 项目代码已推送到 GitHub

## 第一步：登录服务器

```bash
ssh root@<你的服务器IP>
```

腾讯云控制台 → 轻量应用服务器 → 远程连接，或使用本地终端 SSH。

## 第二步：安装 Docker

```bash
# 安装 Docker
curl -fsSL https://get.docker.com | sh

# 启动 Docker 服务
systemctl enable docker
systemctl start docker

# 验证
docker --version
```

> 腾讯云轻量服务器已预装 Docker，可跳过此步。

## 第三步：克隆项目

```bash
git clone https://github.com/FleetingLore/ducia-listing-framework.git
cd ducia-listing-framework
git checkout indev
```

## 第四步：配置

编辑 `config/site.json` 设置站点名称：

```json
{ "name": "我的文档站" }
```

编辑 `config/auth.json` 修改 JWT 密钥（务必更换默认值）：

```json
{ "jwt_secret": "生成一个随机字符串" }
```

```bash
# 生成随机密钥
openssl rand -hex 32
```

编辑 `config/sequence.json` 修改管理员登录序列（可选）：

```json
{ "sequence": [3, 1, 4, 2, 4] }
```

## 第五步：启动服务

```bash
docker compose up -d
```

首次构建需要 5-10 分钟（编译 Rust + 构建前端）。之后重启秒级完成。

查看运行状态：

```bash
docker compose ps
docker compose logs -f
```

## 第六步：配置防火墙

默认服务监听 `0.0.0.0:3001`，需要在云服务器防火墙放行端口。

**腾讯云控制台操作**：

1. 进入轻量应用服务器 → 防火墙
2. 添加规则：协议 `TCP`，端口 `3001`，策略 `允许`
3. 保存

验证：

```bash
curl http://<服务器IP>:3001/api/locales
# 应返回语言列表 JSON
```

浏览器访问 `http://<服务器IP>:3001` 即可看到文档站。

## 第七步：配置 Nginx 反向代理（可选）

使用域名访问并开启 HTTPS：

### 安装 Nginx

```bash
apt update && apt install -y nginx certbot python3-certbot-nginx
```

### 配置站点

创建 `/etc/nginx/sites-available/ducia`：

```nginx
server {
    listen 80;
    server_name docs.example.com;   # 替换为你的域名

    location / {
        proxy_pass http://127.0.0.1:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

启用站点：

```bash
ln -s /etc/nginx/sites-available/ducia /etc/nginx/sites-enabled/
rm /etc/nginx/sites-enabled/default
nginx -t && systemctl reload nginx
```

### 配置 HTTPS

```bash
certbot --nginx -d docs.example.com
```

按提示操作，Certbot 自动申请 Let's Encrypt 证书并更新 Nginx 配置。

### 自动续期

```bash
# certbot 已自动添加定时任务，验证：
systemctl status certbot.timer
```

## 第八步：开机自启

Docker Compose 服务已配置 `restart: unless-stopped`，服务器重启后自动恢复。

如需手动管理：

```bash
docker compose up -d      # 启动
docker compose down       # 停止
docker compose restart    # 重启（修改配置后）
docker compose pull       # 更新镜像
docker compose up -d --build  # 重新构建
```

## 更新部署

代码更新后重新部署：

```bash
cd ducia-listing-framework
git pull
docker compose up -d --build
```

## 常用运维命令

```bash
# 查看日志
docker compose logs -f ducia

# 进入容器
docker compose exec ducia sh

# 备份数据
tar -czf backup-$(date +%Y%m%d).tar.gz config/ docs/ data/

# 恢复数据
tar -xzf backup-20240623.tar.gz
```

## 完整架构

```text
Internet
  │
  ▼
Nginx (:80/:443) ─── 反向代理 ──→ ducia-server (:3001)
  │                                  │
  ├── HTTPS (Let's Encrypt)          ├── serve dist/ (前端静态文件)
  └── 域名 docs.example.com          ├── config/ (配置文件)
                                     ├── docs/ (文档内容)
                                     └── data/ (SQLite 数据)
```
