.PHONY: help dev build release tag push bump-version clean install check

# 版本号变量 - 修改这里来更新版本
TAG := 0.0.13

help: ## 显示帮助信息
	@echo "OpenClaw Manager - 常用命令"
	@echo ""
	@echo "使用方法: make [命令]"
	@echo ""
	@echo "可用命令:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "当前版本: $(TAG)"

install: ## 安装依赖
	@echo "📦 安装依赖..."
	npm install

dev: ## 启动开发服务器
	@echo "🚀 启动开发服务器..."
	npm run tauri dev

build: ## 构建应用
	@echo "🔨 构建应用..."
	npm run tauri build

check: ## 检查 Rust 代码
	@echo "🔍 检查 Rust 代码..."
	cd src-tauri && cargo check

fmt: ## 格式化 Rust 代码
	@echo "✨ 格式化 Rust 代码..."
	cd src-tauri && cargo fmt

lint: ## Lint 前端代码
	@echo "🔍 Lint 前端代码..."
	npm run lint

bump-version: ## 更新版本号到 TAG 变量指定的版本
	@echo "📝 更新版本号到 $(TAG)..."
	@sed -i '' 's/"version": "[0-9]*\.[0-9]*\.[0-9]*"/"version": "$(TAG)"/g' package.json
	@sed -i '' 's/version = "[0-9]*\.[0-9]*\.[0-9]*"/version = "$(TAG)"/g' src-tauri/Cargo.toml
	@sed -i '' 's/"version": "[0-9]*\.[0-9]*\.[0-9]*"/"version": "$(TAG)"/g' src-tauri/tauri.conf.json
	@echo "✅ 版本号已更新到 $(TAG)"

tag: bump-version ## 创建 git tag（会先更新版本号）
	@echo "🏷️  创建 tag v$(TAG)..."
	git add -A
	git commit -m "chore: bump version to $(TAG)" || true
	git tag -a v$(TAG) -m "Release v$(TAG)"
	@echo "✅ Tag v$(TAG) 已创建"

push: ## 推送代码和 tag 到远程仓库
	@echo "🚀 推送到远程仓库..."
	git push origin main
	git push origin v$(TAG)
	@echo "✅ 已推送到远程仓库"

release: tag push ## 完整发布流程：更新版本 -> 创建 tag -> 推送
	@echo "🎉 发布 v$(TAG) 完成！"
	@echo "查看构建状态: https://github.com/jerryan999/openclaw-manager/actions"

rollback-tag: ## 删除本地和远程的当前 TAG
	@echo "⚠️  删除 tag v$(TAG)..."
	git tag -d v$(TAG) || true
	git push origin :refs/tags/v$(TAG) || true
	@echo "✅ Tag v$(TAG) 已删除"

list-tags: ## 列出所有 tags
	@echo "📋 现有 tags:"
	@git tag -l

status: ## 查看 git 状态
	@git status

clean: ## 清理构建产物
	@echo "🧹 清理构建产物..."
	rm -rf dist
	rm -rf src-tauri/target
	rm -rf node_modules
	@echo "✅ 清理完成"

update-deps: ## 更新依赖
	@echo "📦 更新前端依赖..."
	npm update
	@echo "📦 更新 Rust 依赖..."
	cd src-tauri && cargo update

# 快速发布命令（指定版本）
# 使用方法: make quick-release TAG=0.0.11
quick-release: ## 快速发布：更新版本 -> 提交 -> 创建 tag -> 推送
	@echo "🚀 快速发布 v$(TAG)..."
	@$(MAKE) release TAG=$(TAG)
