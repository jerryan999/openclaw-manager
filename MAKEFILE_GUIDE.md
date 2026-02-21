# Makefile 使用指南

OpenClaw Manager 项目的 Makefile 提供了便捷的构建和开发命令。

---

## 🚀 快速开始

### 完整构建流程（一键完成）

```bash
make quickstart
```

这会自动执行：
1. 安装 npm 依赖
2. 下载打包资源（Node.js + OpenClaw）
3. 构建完全离线版应用

---

## 📋 常用命令

### 查看帮助

```bash
make help
```

显示所有可用命令。

### 检查环境

```bash
make check
```

检查构建所需的环境：
- Node.js
- npm
- Rust
- Cargo
- 资源文件

### 显示项目信息

```bash
make info
```

显示项目版本和资源下载状态。

---

## 🔧 开发命令

### 安装依赖

```bash
make install
```

安装项目的 npm 依赖。

### 开发模式运行

```bash
make dev
```

启动 Tauri 开发服务器，支持热重载。

### 运行测试

```bash
make test
```

运行前端和后端的测试套件。

---

## 🏗️ 构建命令

### 构建完全离线版

```bash
make build
```

构建包含 Node.js + OpenClaw 的完全离线安装包。

**输出位置**: `src-tauri/target/release/bundle/msi/*.msi`

**首次构建时间**: 6-8 分钟  
**后续构建时间**: 2-3 分钟

### 仅构建前端

```bash
make build-frontend
```

只构建 React 前端（使用 Vite）。

### 仅构建后端（调试模式）

```bash
make build-backend
```

只构建 Rust 后端（调试模式，编译速度快）。

### 仅构建后端（发布模式）

```bash
make build-backend-release
```

只构建 Rust 后端（发布模式，优化性能）。

---

## 📦 资源管理

### 下载打包资源

```bash
make resources
```

下载构建离线版所需的资源：
- Node.js Windows x64 (~33 MB)
- OpenClaw 离线包 (~17 MB)

资源保存在 `src-tauri/resources/` 目录。

### 清理资源文件

```bash
make clean-resources
```

删除已下载的资源文件。

---

## 🗑️ 清理命令

### 清理构建产物

```bash
make clean
```

删除：
- `dist/` - 前端构建产物
- `src-tauri/target/` - Rust 构建产物
- `node_modules/.cache/` - npm 缓存

### 清理所有文件

```bash
make clean-all
```

除了构建产物，还会删除：
- 下载的资源文件
- `node_modules/` 目录

---

## 📊 实用工具

### 查看构建产物大小

```bash
make size
```

显示 MSI 安装包和 EXE 可执行文件的大小。

### 打开安装包目录

```bash
make open-bundle
```

在文件资源管理器中打开 `bundle/` 目录。

---

## 🎯 发布命令

### 准备发布

```bash
make release
```

构建应用并显示发布清单：
1. 测试 MSI 安装包
2. 创建 GitHub Release
3. 上传安装包
4. 编写发布说明

---

## 🔍 手动构建命令

如果你需要手动运行构建命令（不通过 Makefile），可以使用：

```bash
make manual-build
```

这会显示完整的手动构建命令。

或者直接运行：

```powershell
# 设置环境变量
set PATH=%USERPROFILE%\.cargo\bin;%PATH%

# 构建应用
npm run tauri:build
```

---

## 📝 命令速查表

| 命令 | 说明 | 用途 |
|------|------|------|
| `make help` | 显示帮助 | 查看所有命令 |
| `make check` | 环境检查 | 验证构建环境 |
| `make install` | 安装依赖 | 安装 npm 包 |
| `make resources` | 下载资源 | 下载 Node.js + OpenClaw |
| `make dev` | 开发模式 | 启动开发服务器 |
| `make build` | 构建应用 | 生成离线安装包 |
| `make test` | 运行测试 | 测试前后端代码 |
| `make clean` | 清理构建 | 删除构建产物 |
| `make info` | 项目信息 | 查看版本和状态 |
| `make quickstart` | 快速开始 | 一键完成所有步骤 |

---

## 🎓 使用示例

### 场景 1: 首次克隆项目

```bash
# 1. 安装依赖
make install

# 2. 下载资源
make resources

# 3. 开发模式运行
make dev
```

### 场景 2: 构建发布版本

```bash
# 检查环境
make check

# 构建
make build

# 打开安装包目录
make open-bundle
```

### 场景 3: 清理后重新构建

```bash
# 清理所有构建产物
make clean

# 重新构建
make build
```

### 场景 4: 完全重置项目

```bash
# 清理所有（包括依赖和资源）
make clean-all

# 重新开始
make quickstart
```

---

## ⚡ 性能优化提示

### 加速构建

1. **首次构建后**，Rust 依赖会被缓存，后续构建会快很多
2. **开发时**使用 `make dev`，支持热重载
3. **测试修改**时只构建前端或后端：`make build-frontend` 或 `make build-backend`

### 减少磁盘占用

- 构建完成后运行 `make clean` 清理临时文件
- 不需要离线打包时删除资源文件：`make clean-resources`

---

## 🐛 故障排查

### 命令找不到

**错误**: `'make' is not recognized`

**解决**: 
- 安装 GNU Make for Windows
- 或使用 Git Bash / WSL

### Rust 相关错误

**错误**: `cargo not found`

**解决**:
```bash
# 检查 Rust 是否安装
rustc --version

# 如果未安装，运行
make check
```

### 资源下载失败

**错误**: 资源文件下载失败

**解决**:
```bash
# 手动运行下载脚本
cd src-tauri/resources
.\download-resources.ps1
```

---

## 📚 相关文档

- [README.md](README.md) - 项目说明
- [BUILD_ENVIRONMENT_SETUP.md](BUILD_ENVIRONMENT_SETUP.md) - 环境配置详细指南
- [VERIFICATION_REPORT.md](VERIFICATION_REPORT.md) - 离线打包验证报告
- [OFFLINE.md](OFFLINE.md) - 离线版说明

---

## 💡 提示

- 使用 `make help` 随时查看命令列表
- 使用 `make check` 在构建前验证环境
- 使用 `make info` 查看当前项目状态
- 首次构建使用 `make quickstart` 最方便

---

**最后更新**: 2026-02-15  
**Makefile 版本**: 1.0
