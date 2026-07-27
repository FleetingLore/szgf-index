.PHONY: setup install-hooks build build-frontend build-backend docs publish-docs deploy sync-docs fetch-docs

setup:
	@echo "Run scripts/setup_dev_env.sh to see required tools."
	@echo "To auto-install on macOS: scripts/setup_dev_env.sh --install"

install-hooks:
	bash scripts/git-hooks/install.sh

build: build-frontend build-backend

build-frontend:
	npm install
	npm run build

build-backend:
	cd backend && cargo build

docs:
	cd backend && cargo doc --no-deps

publish-docs:
	cd backend && cargo doc --no-deps
	# publish to gh-pages using a worktree
	rm -rf .gh-pages || true
	git worktree add -B gh-pages .gh-pages gh-pages || true
	rsync -a --delete backend/target/doc/ .gh-pages/
	cd .gh-pages && git add -A && git commit -m "Publish docs (cargo doc)" || echo "No changes to commit"
	git push -u origin gh-pages --force

deploy:
	./deploy.sh

sync-docs:
	./sync-docs.sh

fetch-docs:
	./fetch-docs.sh
