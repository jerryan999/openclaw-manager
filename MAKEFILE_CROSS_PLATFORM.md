# Makefile 跨平台支持说明

OpenClaw Manager 的 Makefile 已经升级为跨平台版本，同时支持 **macOS** 和 **Windows**。

---

## 🎯 支持的平台

### ✅ Windows
- Windows 10/11
- PowerShell / CMD
- Visual Studio Build Tools

### ✅ macOS
- macOS 10.15+
- Apple Silicon (ARM64)
- Intel (x64)
- Xcode Command Line Tools

### 🔄 Linux (基础支持)
- Ubuntu / Debian
- Fedora / CentOS
- Arch Linux

---

## 🔍 自动平台检测

Makefile 会自动检测当前操作系统并使用相应的命令：

```makefile
ifeq ($(OS),Windows_NT)
    DETECTED_OS := Windows
else
    DETECTED_OS := $(shell uname -s)
    ifeq ($(DETECTED_OS),Darwin)
        DETECTED_OS := macOS
    endif
endif
```

---

## 📦 平台差异

### 资源文件

#### Windows
- `nodejs/node-windows-x64.zip` (33 MB)
- `openclaw/openclaw.tgz` (17 MB)

#### macOS
- `nodejs/node-macos-arm64.tar.gz` (ARM64, 40 MB)
- `nodejs/node-macos-x64.tar.gz` (x64, 42 MB)
- `openclaw/openclaw.tgz` (17 MB)

#### Linux
- `nodejs/node-linux-x64.tar.gz` (44 MB)
- `openclaw/openclaw.tgz` (17 MB)

### 构建产物

#### Windows
- `src-tauri/target/release/bundle/msi/*.msi` (~71 MB)
- `src-tauri/target/release/bundle/nsis/*.exe`

#### macOS
- `src-tauri/target/release/bundle/dmg/*.dmg` (~70 MB)
- `src-tauri/target/release/bundle/macos/*.app`

#### Linux
- `src-tauri/target/release/bundle/appimage/*.AppImage`
- `src-tauri/target/release/bundle/deb/*.deb`

---

## 🚀 使用方法

### 所有平台通用命令

```bash
# 查看帮助（显示当前平台）
make help

# 检查环境
make check

# 显示项目信息
make info

# 下载资源（自动选择平台）
make resources

# 安装依赖
make install

# 开发模式
make dev

# 构建应用
make build

# 清理
make clean
```

---

## 🔧 平台特定行为

### Windows 环境

**环境变量**:
```makefile
CARGO_BIN = $(USERPROFILE)\.cargo\bin
PATH = $(CARGO_BIN);%PATH%
```

**命令示例**:
```bash
make build
# 内部执行: set PATH=%USERPROFILE%\.cargo\bin;%PATH% && npm run tauri:build
```

**资源下载**:
```bash
make resources
# 执行: powershell -ExecutionPolicy Bypass -File .\download-resources.ps1
```

### macOS/Linux 环境

**环境变量**:
```makefile
CARGO_BIN = $(HOME)/.cargo/bin
PATH = $(CARGO_BIN):$PATH
```

**命令示例**:
```bash
make build
# 内部执行: export PATH="$HOME/.cargo/bin:$PATH" && npm run tauri:build
```

**资源下载**:
```bash
make resources
# 执行: bash ./download-resources.sh
```

---

## 📋 完整命令对照表

| 功能 | Windows | macOS/Linux | 统一命令 |
|------|---------|-------------|---------|
| 设置PATH | `set PATH=...` | `export PATH=...` | 自动处理 |
| 删除文件 | `del /q` | `rm -f` | `make clean` |
| 删除目录 | `rmdir /s /q` | `rm -rf` | `make clean` |
| 打开目录 | `explorer` | `open` (macOS) / `xdg-open` (Linux) | `make open-bundle` |
| 下载资源 | PowerShell 脚本 | Bash 脚本 | `make resources` |
| 检查文件 | `if exist` | `test -f` | `make check` |

---

## 🧪 测试跨平台 Makefile

### Windows 测试

```powershell
# 查看平台信息
make help
# 输出: Platform: Windows

# 检查环境
make check

# 查看项目信息
make info
# 显示 Windows 特定的资源状态
```

### macOS 测试

```bash
# 查看平台信息
make help
# 输出: Platform: macOS

# 检查环境
make check

# 查看项目信息
make info
# 显示 macOS 特定的资源状态（ARM64 + x64）
```

---

## 🎯 构建示例

### Windows 构建流程

```bash
# 1. 检查环境
make check

# 2. 下载 Windows 资源
make resources
# 下载: node-windows-x64.zip

# 3. 构建
make build
# 输出: src-tauri/target/release/bundle/msi/*.msi

# 4. 打开安装包目录
make open-bundle
# 使用 explorer 打开
```

### macOS 构建流程

```bash
# 1. 检查环境
make check

# 2. 下载 macOS 资源
make resources
# 下载: node-macos-arm64.tar.gz + node-macos-x64.tar.gz

# 3. 构建
make build
# 输出: src-tauri/target/release/bundle/dmg/*.dmg

# 4. 打开安装包目录
make open-bundle
# 使用 open 打开
```

---

## 💡 高级功能

### 条件编译

Makefile 使用条件语句处理平台差异：

```makefile
ifeq ($(DETECTED_OS),Windows)
    # Windows 特定命令
    @set PATH=$(CARGO_BIN);%PATH% && npm run tauri:build
else
    # Unix 特定命令
    @export PATH="$(CARGO_BIN):$$PATH" && npm run tauri:build
endif
```

### 资源检查

不同平台检查不同的资源文件：

```makefile
ifeq ($(DETECTED_OS),Windows)
    @if exist "$(RESOURCES_DIR)\nodejs\node-windows-x64.zip" ...
else ifeq ($(DETECTED_OS),macOS)
    @test -f "$(RESOURCES_DIR)/nodejs/node-macos-arm64.tar.gz" ...
    @test -f "$(RESOURCES_DIR)/nodejs/node-macos-x64.tar.gz" ...
else
    @test -f "$(RESOURCES_DIR)/nodejs/node-linux-x64.tar.gz" ...
endif
```

---

## 🐛 故障排查

### Windows 问题

**问题**: `make` 命令找不到

**解决**:
1. 安装 [Make for Windows](http://gnuwin32.sourceforge.net/packages/make.htm)
2. 或使用 Git Bash
3. 或使用 WSL

**问题**: PowerShell 脚本执行策略

**解决**:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### macOS 问题

**问题**: 权限被拒绝

**解决**:
```bash
chmod +x src-tauri/resources/download-resources.sh
```

**问题**: xcode-select 未安装

**解决**:
```bash
xcode-select --install
```

### 通用问题

**问题**: Cargo 命令找不到

**解决**:
```bash
# 检查 Rust 安装
rustc --version

# 手动设置 PATH
export PATH="$HOME/.cargo/bin:$PATH"  # macOS/Linux
set PATH=%USERPROFILE%\.cargo\bin;%PATH%  # Windows
```

---

## 📚 参考资料

### Makefile 语法

- [GNU Make 文档](https://www.gnu.org/software/make/manual/)
- [跨平台 Makefile 最佳实践](https://makefiletutorial.com/)

### 平台特定文档

- **Windows**: [BUILD_ENVIRONMENT_SETUP.md](BUILD_ENVIRONMENT_SETUP.md)
- **macOS**: Tauri 官方文档
- **通用**: [MAKEFILE_GUIDE.md](MAKEFILE_GUIDE.md)

---

## ✅ 验证清单

### Windows
- [ ] `make help` 显示 "Platform: Windows"
- [ ] `make check` 检测 Windows 资源
- [ ] `make resources` 下载 Windows 资源
- [ ] `make build` 生成 .msi 文件

### macOS
- [ ] `make help` 显示 "Platform: macOS"
- [ ] `make check` 检测 ARM64 + x64 资源
- [ ] `make resources` 下载 macOS 资源
- [ ] `make build` 生成 .dmg 文件

---

## 🎉 总结

跨平台 Makefile 的优势：

✅ **统一命令** - 所有平台使用相同的命令  
✅ **自动检测** - 自动识别当前操作系统  
✅ **智能处理** - 根据平台选择正确的工具  
✅ **易于维护** - 单一文件管理多平台  
✅ **开发友好** - 简化跨平台开发流程  

---

**最后更新**: 2026-02-15  
**Makefile 版本**: 2.0 (跨平台版)  
**支持平台**: Windows, macOS, Linux
