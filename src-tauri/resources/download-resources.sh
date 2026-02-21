#!/bin/bash
set -e

# 下载打包资源的脚本
# 用于 CI/CD 或本地开发

NODE_VERSION="22.12.0"
OPENCLAW_PACKAGE="openclaw"
GIT_VERSION="2.53.0"

echo "=========================================="
echo "  下载打包资源"
echo "=========================================="
echo ""

# 创建目录
mkdir -p nodejs
mkdir -p openclaw
mkdir -p git

# 检测当前平台
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

echo "当前平台: $OS-$ARCH"
echo ""

# 下载 Node.js
echo "📦 下载 Node.js v${NODE_VERSION}..."
cd nodejs

case "$OS-$ARCH" in
  darwin-arm64|darwin-x86_64)
    echo "  - macOS ARM64"
    curl -L -o node-macos-arm64.tar.gz \
      "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-darwin-arm64.tar.gz"
    echo "  ✓ 下载完成: node-macos-arm64.tar.gz"

    echo "  - macOS x64"
    curl -L -o node-macos-x64.tar.gz \
      "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-darwin-x64.tar.gz"
    echo "  ✓ 下载完成: node-macos-x64.tar.gz"
    ;;
  linux-x86_64)
    echo "  - Linux x64"
    curl -L -o node-linux-x64.tar.gz \
      "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-x64.tar.gz"
    echo "  ✓ 下载完成: node-linux-x64.tar.gz"
    ;;
  *)
    echo "  ⚠️  未知平台: $OS-$ARCH"
    echo "  请手动下载 Node.js"
    ;;
esac

cd ..
echo ""

# 准备 Git 资源（Windows 使用 git-portable.zip）
echo "📦 下载 Git 资源..."
cd git

case "$OS-$ARCH" in
  msys_nt-*|mingw*|cygwin_nt-*|windows-*)
    GIT_FILE="git-portable.zip"
    echo "  - Windows x64 (portable)"
    if command -v git >/dev/null 2>&1; then
      GIT_EXE="$(command -v git)"
      GIT_BIN_DIR="$(dirname "$GIT_EXE")"
      GIT_ROOT="$(dirname "$GIT_BIN_DIR")"
      if [ ! -f "$GIT_FILE" ]; then
        (cd "$GIT_ROOT" && zip -r "$OLDPWD/$GIT_FILE" . >/dev/null)
        echo "  ✓ 已打包当前系统 Git: $GIT_FILE"
      else
        echo "  ✓ 已存在: $GIT_FILE（跳过）"
      fi
    else
      echo "  ⚠️  未检测到系统 Git，无法自动生成 $GIT_FILE"
      echo "  请手动放置 Git for Windows 的 .zip 到 src-tauri/resources/git/"
    fi
    ;;
  darwin-*|linux-*)
    echo "  - $OS-$ARCH"
    echo "  ✓ 当前平台默认使用系统 Git（离线 Git 资源主要用于 Windows）"
    ;;
  *)
    echo "  ⚠️  未知平台: $OS-$ARCH，跳过 Git 资源下载"
    ;;
esac

cd ..
echo ""

# 下载 OpenClaw（离线安装包，不需要 Git）
echo "📦 下载 OpenClaw（离线安装，无需 Git）..."
cd openclaw

if command -v npm &> /dev/null; then
  echo "  使用 npm pack 打包..."
  rm -f *.tgz

  # 强制清除缓存并从 registry 获取最新版本
  npm cache clean --force 2>/dev/null || true
  npm pack "${OPENCLAW_PACKAGE}@latest" --prefer-online

  # npm pack openclaw 生成 openclaw-<version>.tgz，重命名为统一文件名
  for file in openclaw-*.tgz; do
    if [ -f "$file" ]; then
      mv "$file" openclaw.tgz
      echo "  ✓ 已保存为: openclaw.tgz"
      break
    fi
  done
else
  echo "  ⚠️  npm 未安装，跳过 OpenClaw 下载"
  echo "  请手动运行: npm pack $OPENCLAW_PACKAGE"
fi

cd ..
echo ""

# 显示下载的文件
echo "=========================================="
echo "  已下载的资源："
echo "=========================================="
echo ""
echo "Node.js:"
ls -lh nodejs/ 2>/dev/null || echo "  (无)"
echo ""
echo "OpenClaw:"
ls -lh openclaw/ 2>/dev/null || echo "  (无)"
echo ""
echo "Git (Windows):"
ls -lh git/ 2>/dev/null || echo "  (无)"
echo ""

echo "✅ 完成！"
echo ""
echo "💡 提示："
echo "  - OpenClaw 离线包安装时不需要 Git，更可靠"
echo "  - 开发模式不需要下载所有平台的资源"
echo "  - 生产构建时确保目标平台的资源已下载"
echo "  - 可以在 CI/CD 中运行此脚本自动下载"
echo ""
echo "📦 打包体积影响："
echo "  - Node.js (每个平台): ~40-50MB"
echo "  - Git portable (Windows): ~45-65MB"
echo "  - OpenClaw .tgz: ~10-20MB"
echo "  - 总计（含离线 Git）: ~90-125MB"
