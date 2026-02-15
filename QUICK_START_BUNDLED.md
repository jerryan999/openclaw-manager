# 🚀 快速开始：打包部署模式

本指南帮助你快速将 Node.js 和 OpenClaw 打包到应用中。

## ⚡ 快速步骤

### 1️⃣ 下载依赖（约 2 分钟）

**macOS/Linux:**
```bash
cd src-tauri/resources
./download-resources.sh
```

**Windows:**
```powershell
cd src-tauri\resources
.\download-resources.ps1
```

### 2️⃣ 构建应用（约 5-10 分钟）

```bash
npm run tauri:build
```

### 3️⃣ 测试

打开构建好的应用：
- **macOS**: `src-tauri/target/release/bundle/dmg/` 中的 `.dmg` 文件
- **Windows**: `src-tauri/target/release/bundle/msi/` 中的 `.msi` 文件  
- **Linux**: `src-tauri/target/release/bundle/appimage/` 中的 `.AppImage` 文件

## 📊 效果对比

| 模式 | 应用大小 | 安装时间 | 网络要求 |
|------|---------|---------|---------|
| **在线模式** | 1-2 MB | 2-5 分钟 | ❌ 需要 |
| **打包模式** | 50-100 MB | 10-30 秒 | ✅ 不需要 |

## 🎯 使用场景

### 推荐打包模式：
- ✅ 面向终端用户发布
- ✅ 网络环境不稳定
- ✅ 需要离线安装
- ✅ 追求极致的用户体验

### 保持在线模式：
- ✅ 内部开发测试
- ✅ 网络环境良好
- ✅ 需要控制应用大小
- ✅ 依赖频繁更新

## 🔄 切换模式

### 启用打包模式（默认）
保持 `src-tauri/tauri.conf.json` 中的资源配置即可。

### 禁用打包模式
注释掉 `src-tauri/tauri.conf.json` 中的资源配置：

```json
{
  "bundle": {
    "resources": {
      // "nodejs/*": "./resources/nodejs/*",
      // "openclaw/*": "./resources/openclaw/*"
    }
  }
}
```

或者简单删除 `src-tauri/resources/nodejs` 和 `src-tauri/resources/openclaw` 目录中的文件。

## 📝 注意事项

1. **首次下载**：资源文件较大（约 50-100MB），首次下载需要一些时间
2. **Git 管理**：资源文件已添加到 `.gitignore`，不会提交到代码仓库
3. **CI/CD**：在 CI 流程中需要运行下载脚本
4. **平台特定**：每个平台只需要下载对应平台的 Node.js

## 🆘 故障排查

### 下载失败
```bash
# 检查网络连接
curl -I https://nodejs.org/dist/

# 手动下载并放到对应目录
cd src-tauri/resources/nodejs
curl -L -O https://nodejs.org/dist/v22.12.0/node-v22.12.0-darwin-arm64.tar.gz
```

### 构建失败
```bash
# 检查资源文件是否存在
ls -lh src-tauri/resources/nodejs/
ls -lh src-tauri/resources/openclaw/

# 查看构建日志
npm run tauri:build 2>&1 | tee build.log
```

### 应用启动失败
检查应用日志：
```bash
# macOS
tail -f ~/Library/Logs/com.openclaw.manager/main.log

# Windows  
type %APPDATA%\com.openclaw.manager\logs\main.log

# Linux
tail -f ~/.local/share/com.openclaw.manager/logs/main.log
```

## 📚 详细文档

完整实现细节请参考：[docs/BUNDLED_DEPLOYMENT.md](docs/BUNDLED_DEPLOYMENT.md)
