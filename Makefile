.PHONY: setup dev build deploy deploy-docs deploy-full deploy-nginx clean

setup:
	npm install

dev:
	npm run dev

build:
	rm -rf dist
	npm run build

deploy:
	./deploy.sh

deploy-docs:
	./deploy.sh docs

deploy-full:
	./deploy.sh full

deploy-nginx:
	./deploy.sh nginx

clean:
	rm -rf dist
