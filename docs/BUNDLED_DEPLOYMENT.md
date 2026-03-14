# 打包部署指南

本文档说明如何将 Node.js 和 OpenClaw 依赖打包到应用程序中，实现离线安装。

## 🎯 优势

- ✅ **无需网络下载**：用户安装时不需要联网
- ✅ **安装速度快**：本地提取比网络下载快得多
- ✅ **版本一致性**：确保所有用户使用相同版本的依赖
- ✅ **离线支持**：支持完全离线环境使用

## 📦 打包大小对比

- **当前**：1-2 MB（仅应用代码）
- **打包后**：50-100 MB（包含 Node.js + OpenClaw）
  - Node.js：~30-50 MB
  - OpenClaw：~10-20 MB

## 🚀 实施步骤

### 1. 下载 Node.js 预编译文件

为需要支持的平台下载 Node.js v22：

```bash
cd src-tauri/resources/nodejs

# macOS ARM64 (Apple Silicon)
curl -L -o node-macos-arm64.tar.gz \
  https://nodejs.org/dist/v22.16.0/node-v22.16.0-darwin-arm64.tar.gz

# macOS x64 (Intel)
curl -L -o node-macos-x64.tar.gz \
  https://nodejs.org/dist/v22.16.0/node-v22.16.0-darwin-x64.tar.gz

# Windows x64
curl -L -o node-windows-x64.zip \
  https://nodejs.org/dist/v22.16.0/node-v22.16.0-win-x64.zip

# Linux x64
curl -L -o node-linux-x64.tar.gz \
  https://nodejs.org/dist/v22.16.0/node-v22.16.0-linux-x64.tar.gz
```

### 2. 打包 OpenClaw npm 包

```bash
cd src-tauri/resources/openclaw

# 方法一：直接打包（推荐）
npm pack openclaw@latest
mv openclaw-*.tgz openclaw.tgz

# 方法二：下载包及所有依赖
npm install --global-style --no-save openclaw@latest
tar -czf openclaw.tgz node_modules/openclaw
```

### 3. 修改安装逻辑

在 `installer.rs` 中，已经实现了优先使用打包资源的逻辑：

```rust
// 1. 检查是否有打包的 Node.js
if utils::bundled::has_bundled_nodejs(&app_handle) {
    // 使用打包的 Node.js
    install_from_bundled_nodejs(&app_handle).await?;
} else {
    // 回退到在线安装
    install_nodejs_online().await?;
}

// 2. 检查是否有打包的 OpenClaw
if utils::bundled::has_bundled_openclaw(&app_handle) {
    // 使用打包的 OpenClaw
    install_from_bundled_openclaw(&app_handle).await?;
} else {
    // 回退到在线安装
    install_openclaw_online().await?;
}
```

### 4. 构建应用

```bash
# 开发构建（不需要资源）
npm run tauri:dev

# 生产构建（需要准备好资源文件）
npm run tauri:build
```

## 📁 目录结构

```
src-tauri/
├── resources/
│   ├── nodejs/
│   │   ├── node-macos-arm64.tar.gz    # 46 MB
│   │   ├── node-macos-x64.tar.gz      # 47 MB
│   │   ├── node-windows-x64.zip       # 28 MB
│   │   └── node-linux-x64.tar.gz      # 45 MB
│   └── openclaw/
│       └── openclaw.tgz  # ~15 MB（官方 openclaw@latest）
├── src/
│   └── utils/
│       └── bundled.rs                 # 资源提取逻辑
└── tauri.conf.json                    # 配置资源打包
```

## 🔄 更新依赖版本

### 更新 Node.js

1. 访问 https://nodejs.org/dist/
2. 选择新版本（如 v22.13.0）
3. 重新下载对应平台的文件
4. 替换 `src-tauri/resources/nodejs/` 中的文件

### 更新 OpenClaw

```bash
cd src-tauri/resources/openclaw
rm -f openclaw.tgz openclaw-*.tgz
npm pack openclaw@latest
mv openclaw-*.tgz openclaw.tgz
```

## 🎛️ 配置选项

可以通过修改 `tauri.conf.json` 来控制是否打包资源：

```json
{
  "bundle": {
    "resources": {
      // 注释掉以下行即可禁用资源打包
      "nodejs/*": "./resources/nodejs/*",
      "openclaw/*": "./resources/openclaw/*"
    }
  }
}
```

## 🧪 测试

### 测试打包资源

```bash
# 1. 构建应用
npm run tauri:build

# 2. 安装构建的应用
# macOS: 打开 src-tauri/target/release/bundle/dmg/*.dmg
# Windows: 运行 src-tauri/target/release/bundle/msi/*.msi
# Linux: 运行 src-tauri/target/release/bundle/appimage/*.AppImage

# 3. 断开网络连接

# 4. 打开应用，测试是否能正常安装 Node.js 和 OpenClaw
```

### 测试在线安装（回退模式）

```bash
# 1. 删除资源文件
rm -rf src-tauri/resources/nodejs/*
rm -rf src-tauri/resources/openclaw/*

# 2. 重新构建
npm run tauri:build

# 3. 应用会自动回退到在线安装模式
```

## 📝 注意事项

1. **Git 忽略**：大文件不应提交到 Git，在 `.gitignore` 中添加：
   ```
   src-tauri/resources/nodejs/*.tar.gz
   src-tauri/resources/nodejs/*.zip
   src-tauri/resources/openclaw/*.tgz
   ```

2. **CI/CD**：在 CI 流程中自动下载资源：
   ```yaml
   - name: Download bundled resources
     run: |
       cd src-tauri/resources
       ./download-resources.sh
   ```

3. **多平台构建**：每个平台只需要打包对应平台的 Node.js：
   - macOS 构建机：只下载 macOS 版本
   - Windows 构建机：只下载 Windows 版本
   - Linux 构建机：只下载 Linux 版本

4. **安装位置**：
   - macOS/Linux: `~/.openclaw-manager/nodejs/`
   - Windows: `%USERPROFILE%\.openclaw-manager\nodejs\`

## 🔧 故障排查

### 资源未找到

检查构建后的应用包中是否包含资源：

```bash
# macOS
ls -lh "/Applications/OpenClaw Manager.app/Contents/Resources/"

# Windows
dir "C:\Program Files\OpenClaw Manager\resources"

# Linux
ls -lh "/opt/openclaw-manager/resources/"
```

### 提取失败

查看应用日志：

```bash
# macOS
tail -f ~/Library/Logs/com.openclaw.manager/main.log

# Windows
type %APPDATA%\com.openclaw.manager\logs\main.log

# Linux
tail -f ~/.local/share/com.openclaw.manager/logs/main.log
```

## 🚀 发布流程

1. ✅ 下载最新的 Node.js 和 OpenClaw
2. ✅ 本地测试打包和安装
3. ✅ 在 CI/CD 中自动下载资源
4. ✅ 构建各平台的安装包
5. ✅ 上传到 GitHub Releases

## 📚 相关文档

- [Tauri Bundle Configuration](https://tauri.app/v2/reference/config/#bundleconfig)
- [Node.js Downloads](https://nodejs.org/dist/)
- [npm pack documentation](https://docs.npmjs.com/cli/v8/commands/npm-pack)
