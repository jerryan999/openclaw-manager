# 📦 离线安装方案

## 概述

为了解决 Windows 用户安装 OpenClaw 时需要 Git 的问题，我们实现了**离线安装方案**。

### ✨ 优势对比

| 方案 | Git 依赖 | 网络要求 | 安装速度 | 可靠性 | 包体积 |
|------|---------|---------|---------|--------|--------|
| **离线安装** | ❌ 不需要 | ❌ 不需要 | ⚡ 快 | ✅ 高 | ~10-20MB |
| 在线安装 | ⚠️ Windows需要 | ✅ 需要 | 🐢 慢 | ⚠️ 中 | 0MB |

## 🚀 使用方法

### 方案 1：自动构建（推荐，CI/CD）

在 GitHub Actions 构建时自动下载并打包：

```yaml
# .github/workflows/build.yml
- name: Download offline packages
  run: |
    cd src-tauri/resources
    ./download-resources.sh
```

### 方案 2：手动打包

```bash
# 进入资源目录
cd src-tauri/resources

# 运行下载脚本
./download-resources.sh

# 或者手动下载 OpenClaw 包
cd openclaw
npm pack openclaw@latest
mv openclaw-*.tgz openclaw.tgz
```

### 方案 3：跳过离线包（保持在线安装）

如果不需要离线包，直接构建即可。程序会自动切换到在线安装模式（Windows 需要 Git）。

## 📝 工作原理

### 安装流程

```
启动安装
    ↓
检查 openclaw.tgz 是否存在？
    ├─ 是 → 离线安装（npm install -g ./openclaw.tgz）
    │         ✓ 不需要 Git
    │         ✓ 不需要网络
    │         ✓ 更快更可靠
    │
    └─ 否 → 在线安装（npm install -g openclaw@latest）
              ⚠️ Windows 需要 Git
              ⚠️ 需要网络连接
```

### 代码实现

离线包检测逻辑（`src-tauri/src/commands/installer.rs`）：

```rust
fn get_bundled_openclaw_package() -> Option<String> {
    let resource_paths = vec![
        "resources/openclaw/openclaw.tgz",
        "../resources/openclaw/openclaw.tgz",
        "openclaw.tgz",
    ];
    
    for path in resource_paths {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    
    None
}
```

## 🎯 最佳实践

### 开发环境

开发时不需要下载离线包，直接使用在线安装即可：

```bash
# 正常开发流程
npm install
npm run tauri:dev
```

### 生产构建

#### 选项 A：完整离线包（推荐）

```bash
# 1. 下载离线包
cd src-tauri/resources
./download-resources.sh

# 2. 构建应用
cd ../..
npm run tauri:build

# 结果：
# - Windows 用户无需 Git
# - 安装速度快
# - 包体积增加 ~15MB
```

#### 选项 B：轻量在线版

```bash
# 直接构建，不下载离线包
npm run tauri:build

# 结果：
# - Windows 用户需要先安装 Git
# - 安装需要网络
# - 包体积最小
```

## 📊 包体积对比

```
不含离线包：
  - macOS .dmg: ~10MB
  - Windows .msi: ~8MB

含离线包：
  - macOS .dmg: ~25MB (+15MB)
  - Windows .msi: ~23MB (+15MB)

含离线包 + Node.js：
  - macOS .dmg: ~75MB (+65MB)
  - Windows .msi: ~70MB (+62MB)
```

## 🔧 故障排查

### 离线包未被识别

检查文件路径和权限：

```bash
# 检查文件是否存在
ls -la src-tauri/resources/openclaw/openclaw.tgz

# 确认文件不是空的
du -h src-tauri/resources/openclaw/openclaw.tgz

# 应该显示 ~10-20MB
```

### 下载脚本失败

```bash
# 检查 npm 是否可用
npm --version

# 手动打包
cd src-tauri/resources/openclaw
npm pack openclaw@latest --verbose
```

### Windows 仍然要求 Git

这说明离线包未被正确打包，检查：

1. 文件是否存在于构建产物中
2. 文件路径是否正确
3. 查看应用日志确认检测逻辑

## 💡 进阶：打包 Node.js

如果想让用户连 Node.js 都不用安装，可以打包 Node.js：

```bash
# 下载 Node.js 预编译版本
cd src-tauri/resources/nodejs

# macOS ARM64
curl -L -o node-macos-arm64.tar.gz \
  https://nodejs.org/dist/v22.12.0/node-v22.12.0-darwin-arm64.tar.gz

# Windows x64
curl -L -o node-windows-x64.zip \
  https://nodejs.org/dist/v22.12.0/node-v22.12.0-win-x64.zip
```

⚠️ 注意：打包 Node.js 会增加 ~50MB 体积，仅在必要时使用。

## 🔧 故障排除

### 报错：Cannot find package 'xxx'（如 ajv）

**原因**：Windows 上 `npm install -g <tgz> --prefix` 有时不会把包的依赖正确装到 prefix 的 `node_modules`，导致运行时缺包。

**当前 CI 做法**：在全局安装 openclaw.tgz 后，再在已安装的包目录内执行 `npm install --omit=dev`，把 openclaw 的**全部生产依赖**装进 `node_modules/openclaw/node_modules/`，避免漏装任意依赖。

若你用的是旧版安装包仍报缺包，可重新安装新构建的版本；或有网络时在 PowerShell 手动补装（示例为 ajv）：

```powershell
$rt = "$env:LOCALAPPDATA\OpenClawManager\runtime"
& "$rt\node\npm.cmd" install ajv --prefix "$rt\npm-global" --no-audit --loglevel=error
```

## 📚 相关文档

- [资源打包说明](../src-tauri/resources/README.md)
- [下载脚本](../src-tauri/resources/download-resources.sh)
- [安装器实现](../src-tauri/src/commands/installer.rs)

## 🎉 总结

使用离线安装方案：
- ✅ 解决了 Windows Git 依赖问题
- ✅ 提升了安装成功率和速度
- ✅ 包体积增加可控（~15MB）
- ✅ 开发体验不受影响（可选）

**推荐在生产构建时启用离线包，给用户最佳体验！**
