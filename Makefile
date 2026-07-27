.PHONY: build dev deploy deploy-docs deploy-full deploy-nginx clean

# ---- 初始化 ----
setup:
	npm install

# ---- 开发 ----
dev:
	npm run dev

# ---- 构建 ----
build:
	rm -rf dist
	npm run build
	cd backend && cargo build --release

build-frontend:
	rm -rf dist
	npm run build

build-backend:
	cd backend && cargo build --release

# ---- 部署 ----
deploy:
	./deploy.sh

deploy-docs:
	./deploy.sh docs

deploy-full:
	./deploy.sh full

deploy-nginx:
	./deploy.sh nginx

# ---- 清理 ----
clean:
	rm -rf dist
	cd backend && cargo clean
