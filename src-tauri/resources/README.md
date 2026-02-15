# 打包资源说明

此目录用于存放需要打包进应用的资源文件。

## 目录结构

```
resources/
├── nodejs/           # Node.js 预编译文件
│   ├── macos-arm64/  # macOS Apple Silicon
│   ├── macos-x64/    # macOS Intel
│   ├── windows-x64/  # Windows 64位
│   └── linux-x64/    # Linux 64位
└── openclaw/         # OpenClaw npm 包
    └── package.tgz   # 打包的 npm 包
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

### 2. OpenClaw npm 包

```bash
# 下载 OpenClaw 包
cd src-tauri/resources/openclaw
npm pack @jerryan999/openclaw-zh

# 或者手动下载所有依赖
npm install --global-style --no-save @jerryan999/openclaw-zh
```

## 注意事项

1. 这些文件会被打包进最终的应用程序中
2. 每个平台只需要包含对应平台的 Node.js 版本
3. 开发时可以先不下载这些文件，使用在线安装方式
4. 发布前需要确保所有资源文件都已准备好
