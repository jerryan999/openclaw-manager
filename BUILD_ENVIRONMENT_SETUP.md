# 🛠️ 构建环境配置指南

**用于构建 OpenClaw Manager 完全离线版**

---

## 📋 环境需求

### 必需工具

| 工具 | 版本要求 | 用途 | 状态 |
|------|---------|------|------|
| **Node.js** | >= 18.0 | 前端构建 | ✅ 已安装 (v24.14.0) |
| **npm** | >= 8.0 | 包管理 | ✅ 已安装 (v10.9.2) |
| **Rust** | >= 1.70 | Tauri 后端编译 | ❌ **未安装** |
| **Cargo** | 随 Rust 安装 | Rust 包管理 | ❌ **未安装** |

### Windows 额外需求

- **Microsoft C++ Build Tools** (用于编译 Rust 依赖)
- **WebView2** (运行时，通常 Windows 11 已内置)

---

## 🚀 快速安装 (Windows)

### 方法一：使用 rustup (推荐)

**1. 下载并安装 Rust**

访问: https://rustup.rs/

或直接运行:
```powershell
# 下载 rustup-init.exe
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"

# 运行安装程序
.\rustup-init.exe
```

**2. 选择默认安装**
- 按提示操作，选择 `1) Proceed with installation (default)`
- 安装过程会自动下载 Rust、Cargo 和必要的工具链

**3. 重启终端**
```powershell
# 关闭当前 PowerShell 窗口，重新打开
```

**4. 验证安装**
```powershell
rustc --version   # 应显示版本号
cargo --version   # 应显示版本号
```

### 方法二：使用 Winget (Windows 11)

```powershell
winget install -e --id Rustlang.Rustup
```

---

## 🔧 安装 Microsoft C++ Build Tools

Rust 编译某些依赖时需要 C++ 编译器。

### 选项 A：安装 Visual Studio Build Tools (推荐)

1. 访问: https://visualstudio.microsoft.com/visual-cpp-build-tools/
2. 下载 "Visual Studio Build Tools"
3. 安装时选择：
   - ✅ "Desktop development with C++"
   - ✅ "C++ build tools" 核心组件

### 选项 B：使用 Winget

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools"
```

---

## ✅ 验证环境

运行以下命令检查所有工具是否正确安装：

```powershell
# Node.js
node --version
npm --version

# Rust
rustc --version
cargo --version

# C++ 编译器 (可选，检查 cl.exe 是否在 PATH 中)
where cl
```

**预期输出**:
```
v24.14.0
10.9.2
rustc 1.xx.x
cargo 1.xx.x
C:\Program Files\Microsoft Visual Studio\...\cl.exe
```

---

## 🏗️ 构建流程

环境配置完成后，按以下步骤构建：

### 步骤 1: 确保资源文件已下载

```powershell
# 检查资源
ls src-tauri/resources/nodejs
ls src-tauri/resources/openclaw
```

应该看到:
- `nodejs/node-windows-x64.zip` (33.26 MB)
- `openclaw/openclaw.tgz` (16.56 MB)

如果没有，运行:
```powershell
cd src-tauri/resources
.\download-resources.ps1
cd ../..
```

### 步骤 2: 构建应用

```powershell
npm run tauri:build
```

**预计耗时**: 5-10 分钟（首次构建）

### 步骤 3: 查找构建产物

构建完成后，安装包位于:
```
src-tauri/target/release/bundle/
├── msi/
│   └── OpenClaw Manager_0.0.18_x64_zh-CN.msi  (~71 MB)
└── nsis/
    └── OpenClaw Manager_0.0.18_x64-setup.exe  (~71 MB)
```

---

## 📊 构建时间参考

| 阶段 | 首次构建 | 增量构建 |
|------|---------|---------|
| 前端编译 (TypeScript + Vite) | 30-60秒 | 10-20秒 |
| Rust 依赖编译 | 3-5分钟 | 0秒 (缓存) |
| Tauri 核心编译 | 2-3分钟 | 1分钟 |
| 资源打包 | 30秒 | 30秒 |
| **总计** | **6-9分钟** | **2-3分钟** |

---

## 🐛 常见问题

### ❌ 错误: `linker 'link.exe' not found`

**原因**: 缺少 C++ 编译工具

**解决方案**: 安装 Visual Studio Build Tools (见上文)

### ❌ 错误: `cargo metadata` command failed

**原因**: Rust 未安装或未加入 PATH

**解决方案**:
1. 安装 Rust (见上文)
2. 重启终端
3. 验证: `cargo --version`

### ❌ 错误: `WebView2Loader.dll not found`

**原因**: 缺少 WebView2 运行时

**解决方案**:
```powershell
# 下载并安装 WebView2
Invoke-WebRequest -Uri "https://go.microsoft.com/fwlink/p/?LinkId=2124703" -OutFile "WebView2Setup.exe"
.\WebView2Setup.exe
```

### ⚠️ 警告: `target was built with panic=unwind`

**影响**: 无，可以忽略

### 🐌 构建速度慢

**优化建议**:
1. 首次构建一定慢（需要编译所有依赖）
2. 后续构建会快很多（利用缓存）
3. 使用 SSD 硬盘
4. 关闭杀毒软件实时扫描（构建时）

---

## 🚦 快速检查清单

在开始构建前，确保：

- [ ] ✅ Node.js >= 18.0 已安装
- [ ] ✅ Rust >= 1.70 已安装  **← 当前缺失**
- [ ] ✅ C++ Build Tools 已安装 **← 当前缺失**
- [ ] ✅ 资源文件已下载 (Node.js + OpenClaw)
- [ ] ✅ 网络连接良好（首次构建需下载依赖）
- [ ] ✅ 磁盘空间充足（至少 5GB）

---

## 📱 一键安装脚本 (推荐)

将以下内容保存为 `setup-build-env.ps1`：

```powershell
# OpenClaw Manager 构建环境一键配置脚本
Write-Host "🛠️  配置构建环境..." -ForegroundColor Cyan

# 1. 检查 Node.js
Write-Host "`n📦 检查 Node.js..." -ForegroundColor Yellow
try {
    $nodeVersion = node --version
    Write-Host "  ✅ Node.js: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Host "  ❌ Node.js 未安装" -ForegroundColor Red
    Write-Host "     请访问: https://nodejs.org/" -ForegroundColor White
    exit 1
}

# 2. 检查 Rust
Write-Host "`n🦀 检查 Rust..." -ForegroundColor Yellow
try {
    $rustVersion = rustc --version
    Write-Host "  ✅ Rust: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "  ⚠️  Rust 未安装，正在安装..." -ForegroundColor Yellow
    
    # 下载 rustup
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"
    
    # 安装 Rust (静默模式)
    .\rustup-init.exe -y
    
    # 刷新环境变量
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
    
    # 验证
    rustc --version
    Write-Host "  ✅ Rust 安装成功！" -ForegroundColor Green
    
    # 清理
    Remove-Item "rustup-init.exe"
}

# 3. 检查 C++ Build Tools
Write-Host "`n🔧 检查 C++ Build Tools..." -ForegroundColor Yellow
$clPath = where.exe cl 2>$null
if ($clPath) {
    Write-Host "  ✅ C++ Build Tools 已安装" -ForegroundColor Green
} else {
    Write-Host "  ⚠️  C++ Build Tools 未检测到" -ForegroundColor Yellow
    Write-Host "     请安装 Visual Studio Build Tools:" -ForegroundColor White
    Write-Host "     https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor White
}

# 4. 检查资源文件
Write-Host "`n📦 检查资源文件..." -ForegroundColor Yellow
if (Test-Path "src-tauri/resources/nodejs/node-windows-x64.zip") {
    Write-Host "  ✅ Node.js 资源已下载" -ForegroundColor Green
} else {
    Write-Host "  ❌ Node.js 资源未下载" -ForegroundColor Red
}

if (Test-Path "src-tauri/resources/openclaw/openclaw.tgz") {
    Write-Host "  ✅ OpenClaw 资源已下载" -ForegroundColor Green
} else {
    Write-Host "  ❌ OpenClaw 资源未下载" -ForegroundColor Red
}

Write-Host "`n✨ 环境检查完成！" -ForegroundColor Green
Write-Host "`n📝 下一步:" -ForegroundColor Cyan
Write-Host "  1. 如果提示缺少工具，请按照上述链接安装" -ForegroundColor White
Write-Host "  2. 重启 PowerShell" -ForegroundColor White
Write-Host "  3. 运行: npm run tauri:build" -ForegroundColor White
```

使用方法:
```powershell
.\setup-build-env.ps1
```

---

## 📚 相关文档

- [Rust 官方安装指南](https://www.rust-lang.org/tools/install)
- [Tauri 环境配置](https://tauri.app/v1/guides/getting-started/prerequisites)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

---

## ✅ 配置完成后

环境配置完成后，运行：

```powershell
npm run tauri:build
```

构建成功后，你会得到一个 **~71 MB** 的完全离线安装包！

---

**最后更新**: 2026-02-15  
**适用版本**: OpenClaw Manager v0.0.18
