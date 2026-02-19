# OpenClaw Manager - Makefile
# Build fully offline Tauri application
# Supports: macOS & Windows

.PHONY: help build dev clean check resources install test release

.DEFAULT_GOAL := help

# Detect OS
ifeq ($(OS),Windows_NT)
	DETECTED_OS := Windows
	CARGO_BIN := $(USERPROFILE)\.cargo\bin
	RESOURCES_DIR := src-tauri\resources
	PATH_SEP := ;
	RM := del /q
	RMDIR := rmdir /s /q
	MKDIR := mkdir
	OPEN := explorer
	NULL := 2>nul
else
	DETECTED_OS := $(shell uname -s)
	CARGO_BIN := $(HOME)/.cargo/bin
	RESOURCES_DIR := src-tauri/resources
	PATH_SEP := :
	RM := rm -f
	RMDIR := rm -rf
	MKDIR := mkdir -p
	NULL := 2>/dev/null
	ifeq ($(DETECTED_OS),Darwin)
		DETECTED_OS := macOS
		OPEN := open
	else
		OPEN := xdg-open
	endif
endif

help: ## Show help
	@echo ""
	@echo "OpenClaw Manager Build Tool"
	@echo "Platform: $(DETECTED_OS)"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  check       Check build environment"
	@echo "  resources   Download bundled resources"
	@echo "  install     Install dependencies"
	@echo "  dev         Run in development mode"
	@echo "  build       Build offline version"
	@echo "  release     Build and prepare release"
	@echo "  clean       Clean build artifacts"
	@echo "  info        Show project info"
	@echo ""

check: ## Check environment
	@echo ""
	@echo "Checking build environment ($(DETECTED_OS))..."
	@echo ""
	@node --version $(NULL) && echo "Node.js: OK" || echo "Node.js: MISSING"
	@npm --version $(NULL) && echo "npm: OK" || echo "npm: MISSING"
	@rustc --version $(NULL) && echo "Rust: OK" || echo "Rust: MISSING"
	@cargo --version $(NULL) && echo "Cargo: OK" || echo "Cargo: MISSING"
	@echo ""
ifeq ($(DETECTED_OS),Windows)
	@if exist "$(RESOURCES_DIR)\nodejs\node-windows-x64.zip" (echo Node.js resource: OK) else (echo Node.js resource: MISSING)
	@if exist "$(RESOURCES_DIR)\openclaw\openclaw-zh.tgz" (echo OpenClaw resource: OK) else (echo OpenClaw resource: MISSING)
else ifeq ($(DETECTED_OS),macOS)
	@test -f "$(RESOURCES_DIR)/nodejs/node-macos-arm64.tar.gz" && echo "Node.js ARM64 resource: OK" || echo "Node.js ARM64 resource: MISSING"
	@test -f "$(RESOURCES_DIR)/nodejs/node-macos-x64.tar.gz" && echo "Node.js x64 resource: OK" || echo "Node.js x64 resource: MISSING"
	@test -f "$(RESOURCES_DIR)/openclaw/openclaw-zh.tgz" && echo "OpenClaw resource: OK" || echo "OpenClaw resource: MISSING"
else
	@test -f "$(RESOURCES_DIR)/nodejs/node-linux-x64.tar.gz" && echo "Node.js resource: OK" || echo "Node.js resource: MISSING"
	@test -f "$(RESOURCES_DIR)/openclaw/openclaw-zh.tgz" && echo "OpenClaw resource: OK" || echo "OpenClaw resource: MISSING"
endif
	@echo ""

resources: ## Download resources
	@echo ""
	@echo "Downloading resources for $(DETECTED_OS)..."
	@echo ""
ifeq ($(DETECTED_OS),Windows)
	@cd $(RESOURCES_DIR) && powershell -ExecutionPolicy Bypass -File .\download-resources.ps1
else
	@cd $(RESOURCES_DIR) && bash ./download-resources.sh
endif

install: ## Install dependencies
	@echo ""
	@echo "Installing dependencies..."
	@echo ""
	npm install

dev: ## Run development mode
	@echo ""
	@echo "Starting development mode..."
	@echo ""
ifeq ($(DETECTED_OS),Windows)
	@set PATH=$(CARGO_BIN);%PATH% && npm run tauri:dev
else
	@export PATH="$(CARGO_BIN):$$PATH" && npm run tauri:dev
endif

build: ## Build application
	@echo ""
	@echo "Building offline version for $(DETECTED_OS)..."
	@echo "This may take 6-8 minutes on first build"
	@echo ""
ifeq ($(DETECTED_OS),Windows)
	@set PATH=$(CARGO_BIN);%PATH% && npm run tauri:build
	@echo ""
	@echo "Build complete!"
	@echo ""
	@if exist "src-tauri\target\release\bundle\msi\*.msi" (echo Output: src-tauri\target\release\bundle\msi\) else (echo No MSI found)
else ifeq ($(DETECTED_OS),macOS)
	@export PATH="$(CARGO_BIN):$$PATH" && npm run tauri:build
	@echo ""
	@echo "Build complete!"
	@echo ""
	@test -d "src-tauri/target/release/bundle/dmg" && echo "Output: src-tauri/target/release/bundle/dmg/" || echo "No DMG found"
	@test -d "src-tauri/target/release/bundle/macos" && echo "Output: src-tauri/target/release/bundle/macos/" || echo "No .app found"
else
	@export PATH="$(CARGO_BIN):$$PATH" && npm run tauri:build
	@echo ""
	@echo "Build complete!"
	@echo ""
	@test -d "src-tauri/target/release/bundle/appimage" && echo "Output: src-tauri/target/release/bundle/appimage/" || echo "No AppImage found"
	@test -d "src-tauri/target/release/bundle/deb" && echo "Output: src-tauri/target/release/bundle/deb/" || echo "No DEB found"
endif
	@echo ""

build-frontend: ## Build frontend only
	@echo "Building frontend..."
	npm run build

build-backend: ## Build backend only (debug)
	@echo "Building Rust backend (debug)..."
ifeq ($(DETECTED_OS),Windows)
	@set PATH=$(CARGO_BIN);%PATH% && cd src-tauri && cargo build
else
	@export PATH="$(CARGO_BIN):$$PATH" && cd src-tauri && cargo build
endif

build-backend-release: ## Build backend only (release)
	@echo "Building Rust backend (release)..."
ifeq ($(DETECTED_OS),Windows)
	@set PATH=$(CARGO_BIN);%PATH% && cd src-tauri && cargo build --release
else
	@export PATH="$(CARGO_BIN):$$PATH" && cd src-tauri && cargo build --release
endif

test: ## Run tests
	@echo "Running tests..."
	npm run test
ifeq ($(DETECTED_OS),Windows)
	@set PATH=$(CARGO_BIN);%PATH% && cd src-tauri && cargo test
else
	@export PATH="$(CARGO_BIN):$$PATH" && cd src-tauri && cargo test
endif

clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
ifeq ($(DETECTED_OS),Windows)
	@if exist "dist" $(RMDIR) "dist"
	@if exist "src-tauri\target" $(RMDIR) "src-tauri\target"
	@if exist "node_modules\.cache" $(RMDIR) "node_modules\.cache"
else
	@$(RMDIR) dist $(NULL) || true
	@$(RMDIR) src-tauri/target $(NULL) || true
	@$(RMDIR) node_modules/.cache $(NULL) || true
endif
	@echo "Clean complete!"

clean-resources: ## Clean downloaded resources
	@echo "Cleaning resources..."
ifeq ($(DETECTED_OS),Windows)
	@if exist "$(RESOURCES_DIR)\nodejs\*.zip" $(RM) "$(RESOURCES_DIR)\nodejs\*.zip"
	@if exist "$(RESOURCES_DIR)\nodejs\*.tar.gz" $(RM) "$(RESOURCES_DIR)\nodejs\*.tar.gz"
	@if exist "$(RESOURCES_DIR)\openclaw\*.tgz" $(RM) "$(RESOURCES_DIR)\openclaw\*.tgz"
else
	@$(RM) $(RESOURCES_DIR)/nodejs/*.zip $(NULL) || true
	@$(RM) $(RESOURCES_DIR)/nodejs/*.tar.gz $(NULL) || true
	@$(RM) $(RESOURCES_DIR)/openclaw/*.tgz $(NULL) || true
endif
	@echo "Resources cleaned!"

clean-all: clean clean-resources ## Clean everything
	@echo "Cleaning all..."
ifeq ($(DETECTED_OS),Windows)
	@if exist "node_modules" $(RMDIR) "node_modules"
else
	@$(RMDIR) node_modules $(NULL) || true
endif
	@echo "All cleaned!"

info: ## Show project info
	@echo ""
	@echo "Project: OpenClaw Manager"
	@echo "Version: 0.0.25"
	@echo "Platform: $(DETECTED_OS)"
	@echo ""
	@echo "Resource Status:"
ifeq ($(DETECTED_OS),Windows)
	@if exist "$(RESOURCES_DIR)\nodejs\node-windows-x64.zip" (echo   Node.js Windows: Downloaded) else (echo   Node.js Windows: Not downloaded)
	@if exist "$(RESOURCES_DIR)\openclaw\openclaw-zh.tgz" (echo   OpenClaw: Downloaded) else (echo   OpenClaw: Not downloaded)
else ifeq ($(DETECTED_OS),macOS)
	@test -f "$(RESOURCES_DIR)/nodejs/node-macos-arm64.tar.gz" && echo "  Node.js ARM64: Downloaded" || echo "  Node.js ARM64: Not downloaded"
	@test -f "$(RESOURCES_DIR)/nodejs/node-macos-x64.tar.gz" && echo "  Node.js x64: Downloaded" || echo "  Node.js x64: Not downloaded"
	@test -f "$(RESOURCES_DIR)/openclaw/openclaw-zh.tgz" && echo "  OpenClaw: Downloaded" || echo "  OpenClaw: Not downloaded"
else
	@test -f "$(RESOURCES_DIR)/nodejs/node-linux-x64.tar.gz" && echo "  Node.js: Downloaded" || echo "  Node.js: Not downloaded"
	@test -f "$(RESOURCES_DIR)/openclaw/openclaw-zh.tgz" && echo "  OpenClaw: Downloaded" || echo "  OpenClaw: Not downloaded"
endif
	@echo ""

size: ## Show bundle sizes
	@echo ""
	@echo "Build Artifacts:"
ifeq ($(DETECTED_OS),Windows)
	@if exist "src-tauri\target\release\bundle\msi" (dir "src-tauri\target\release\bundle\msi\*.msi" /s) else (echo No MSI found)
	@if exist "src-tauri\target\release\openclaw-manager.exe" (dir "src-tauri\target\release\openclaw-manager.exe") else (echo No EXE found)
else ifeq ($(DETECTED_OS),macOS)
	@test -d "src-tauri/target/release/bundle/dmg" && ls -lh src-tauri/target/release/bundle/dmg/*.dmg || echo "No DMG found"
	@test -d "src-tauri/target/release/bundle/macos" && du -sh src-tauri/target/release/bundle/macos/*.app || echo "No .app found"
else
	@test -d "src-tauri/target/release/bundle/appimage" && ls -lh src-tauri/target/release/bundle/appimage/*.AppImage || echo "No AppImage found"
	@test -d "src-tauri/target/release/bundle/deb" && ls -lh src-tauri/target/release/bundle/deb/*.deb || echo "No DEB found"
endif
	@echo ""

open-bundle: ## Open bundle directory
ifeq ($(DETECTED_OS),Windows)
	@if exist "src-tauri\target\release\bundle" ($(OPEN) "src-tauri\target\release\bundle") else (echo Bundle directory not found)
else
	@test -d "src-tauri/target/release/bundle" && $(OPEN) "src-tauri/target/release/bundle" || echo "Bundle directory not found"
endif

release: build ## Build and prepare release
	@echo ""
	@echo "Release package ready!"
	@echo ""
	@echo "Next steps:"
ifeq ($(DETECTED_OS),Windows)
	@echo "  1. Test the MSI installer"
else ifeq ($(DETECTED_OS),macOS)
	@echo "  1. Test the DMG installer"
else
	@echo "  1. Test the package"
endif
	@echo "  2. Create GitHub Release"
	@echo "  3. Upload the installer"
	@echo "  4. Write release notes"
	@echo ""

quickstart: install resources build ## Quick start: install + resources + build
	@echo ""
	@echo "Quickstart complete!"
ifeq ($(DETECTED_OS),Windows)
	@echo "Next: Run 'make dev' or test the MSI installer"
else ifeq ($(DETECTED_OS),macOS)
	@echo "Next: Run 'make dev' or test the DMG installer"
else
	@echo "Next: Run 'make dev' or test the package"
endif
	@echo ""

# Manual build command for reference
manual-build: ## Show manual build command
	@echo ""
	@echo "Manual build command for $(DETECTED_OS):"
ifeq ($(DETECTED_OS),Windows)
	@echo "  set PATH=$(CARGO_BIN);%%PATH%% && npm run tauri:build"
	@echo ""
	@echo "With environment setup:"
	@echo "  set PATH=$(CARGO_BIN);%%PATH%%"
	@echo "  npm run tauri:build"
else
	@echo "  export PATH=\"$(CARGO_BIN):$$PATH\" && npm run tauri:build"
	@echo ""
	@echo "With environment setup:"
	@echo "  export PATH=\"$(CARGO_BIN):$$PATH\""
	@echo "  npm run tauri:build"
endif
	@echo ""
