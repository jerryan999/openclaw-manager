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

# 下载 OpenClaw（离线安装包，不需要 Git）
echo "📦 下载 OpenClaw（离线安装，无需 Git）..."
cd openclaw

if command -v npm &> /dev/null; then
  echo "  使用 npm pack 打包..."
  rm -f *.tgz
  # 强制清除缓存并从 registry 获取最新版本
  npm cache clean --force 2>/dev/null || true
  npm pack "$OPENCLAW_PACKAGE@latest" --prefer-online
  
  # 重命名为统一的文件名
  for file in jerryan999-openclaw-zh-*.tgz; do
    if [ -f "$file" ]; then
      mv "$file" openclaw-zh.tgz
      echo "  ✓ 已保存为: openclaw-zh.tgz"
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
echo "  - OpenClaw .tgz: ~10-20MB"
echo "  - 总计（单平台）: ~50-70MB"
