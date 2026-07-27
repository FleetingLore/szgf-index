# ═══ Stage 1: 构建前端 ═══
FROM node:20-alpine AS frontend
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY index.html vite.config.js tsconfig.json ./
COPY public/ public/
COPY src/ src/
RUN npm run build

# ═══ Stage 2: 构建后端 ═══
FROM rust:1.88-alpine AS backend
RUN apk add --no-cache musl-dev pkgconfig openssl-dev
WORKDIR /app
COPY backend/ backend/
WORKDIR /app/backend
RUN cargo build --release

# ═══ Stage 3: 运行时 ═══
FROM alpine:3.20
RUN apk add --no-cache ca-certificates tzdata
WORKDIR /app

# 后端二进制
COPY --from=backend /app/backend/target/release/ducia-server .

# 前端产物
COPY --from=frontend /app/dist/ ./dist/

# 配置文件和数据目录
COPY config/ ./config/
COPY docs/ ./docs/
RUN mkdir -p data

ENV TZ=Asia/Shanghai
EXPOSE 3001
CMD ["./ducia-server"]
