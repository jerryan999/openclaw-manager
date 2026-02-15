# 打包资源说明

此目录用于存放需要打包进应用的资源文件。

## 目录结构

```
resources/
├── nodejs/           # Node.js 预编译文件（可选）
│   ├── macos-arm64/  # macOS Apple Silicon
│   ├── macos-x64/    # macOS Intel
│   ├── windows-x64/  # Windows 64位
│   └── linux-x64/    # Linux 64位
├── git/              # Portable Git（仅 Windows，可选）
│   └── git-portable-windows-x64.7z
└── openclaw/         # OpenClaw npm 包（离线安装）
    └── openclaw-zh.tgz   # 打包的 npm 包
```

## 下载资源

### 1. Node.js 二进制文件

从 https://nodejs.org/dist/ 下载对应平台的预编译版本：

```bash
# macOS ARM64 (Apple Silicon)
curl -o src-tauri/resources/nodejs/node-macos-arm64.tar.gz \
  https://nodejs.org/dist/v22.12.0/node-v22.12.0-darwin-arm64.tar.gz

# macOS x64 (Intel)
curl -o src-tauri/resources/nodejs/node-macos-x64.tar.gz \
  https://nodejs.org/dist/v22.12.0/node-v22.12.0-darwin-x64.tar.gz

# Windows x64
curl -o src-tauri/resources/nodejs/node-windows-x64.zip \
  https://nodejs.org/dist/v22.12.0/node-v22.12.0-win-x64.zip

# Linux x64
curl -o src-tauri/resources/nodejs/node-linux-x64.tar.gz \
  https://nodejs.org/dist/v22.12.0/node-v22.12.0-linux-x64.tar.gz
```

### 2. OpenClaw npm 包（离线安装，推荐）

```bash
# 创建目录
mkdir -p src-tauri/resources/openclaw

# 下载并打包 OpenClaw（不需要 Git）
cd src-tauri/resources/openclaw
npm pack @jerryan999/openclaw-zh

# 生成的文件类似：jerryan999-openclaw-zh-1.0.0.tgz
# 重命名为统一的名字
mv jerryan999-openclaw-zh-*.tgz openclaw-zh.tgz
```

### 3. Portable Git（可选，仅 Windows）

如果不想要求用户安装 Git，可以打包 Portable Git：

```bash
# 下载 Portable Git for Windows
# 访问 https://github.com/git-for-windows/git/releases
# 下载 PortableGit-<version>-64-bit.7z (~50MB)

mkdir -p src-tauri/resources/git
# 将下载的文件放到 src-tauri/resources/git/git-portable-windows-x64.7z
```

**注意**：实际上使用离线 .tgz 安装方式后，**不再需要 Git**，这是更推荐的方案。

## 注意事项

1. 这些文件会被打包进最终的应用程序中
2. 每个平台只需要包含对应平台的 Node.js 版本
3. 开发时可以先不下载这些文件，使用在线安装方式
4. 发布前需要确保所有资源文件都已准备好
