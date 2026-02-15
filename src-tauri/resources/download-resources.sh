#!/bin/bash
set -e

# 下载打包资源的脚本
# 用于 CI/CD 或本地开发

NODE_VERSION="22.12.0"
OPENCLAW_PACKAGE="@jerryan999/openclaw-zh"

echo "=========================================="
echo "  下载打包资源"
echo "=========================================="
echo ""

# 创建目录
mkdir -p nodejs
mkdir -p openclaw

# 检测当前平台
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

echo "当前平台: $OS-$ARCH"
echo ""

# 下载 Node.js
echo "📦 下载 Node.js v${NODE_VERSION}..."
cd nodejs

case "$OS-$ARCH" in
  darwin-arm64)
    echo "  - macOS ARM64"
    curl -L -o node-macos-arm64.tar.gz \
      "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-darwin-arm64.tar.gz"
    echo "  ✓ 下载完成: node-macos-arm64.tar.gz"
    ;;
  darwin-x86_64)
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

# 下载 OpenClaw
echo "📦 下载 OpenClaw..."
cd openclaw

if command -v npm &> /dev/null; then
  echo "  使用 npm pack..."
  rm -f *.tgz
  npm pack "$OPENCLAW_PACKAGE"
  echo "  ✓ 下载完成: $(ls -1 *.tgz | head -1)"
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

echo "✅ 完成！"
echo ""
echo "提示："
echo "  - 开发模式不需要下载所有平台的资源"
echo "  - 生产构建时确保目标平台的资源已下载"
echo "  - 可以在 CI/CD 中运行此脚本自动下载"
