# 📋 打包检查清单

## 离线包打包确认

### ✅ 配置检查

#### 1. Tauri 资源配置
**文件**: `src-tauri/tauri.conf.json`

```json
"resources": [
  "./resources/nodejs/*",
  "./resources/openclaw/*"   // ← 这会包含 openclaw-zh.tgz
]
```

✅ **已配置**

---

#### 2. CI/CD 下载脚本
**文件**: `.github/workflows/build.yml`

**macOS**:
```yaml
- name: Download bundled resources (macOS/Linux)
  if: matrix.name == 'macos'
  run: |
    cd src-tauri/resources
    chmod +x download-resources.sh
    ./download-resources.sh
```

**Windows**:
```yaml
- name: Download bundled resources (Windows)
  if: matrix.name == 'windows'
  run: |
    cd src-tauri/resources
    ./download-resources.ps1
  shell: pwsh
```

✅ **已配置**

---

#### 3. 下载脚本功能
**文件**: 
- `src-tauri/resources/download-resources.sh` (macOS/Linux)
- `src-tauri/resources/download-resources.ps1` (Windows)

**功能**:
- ✅ 下载 `@jerryan999/openclaw-zh` 包
- ✅ 重命名为统一文件名 `openclaw-zh.tgz`
- ✅ 保存到 `src-tauri/resources/openclaw/` 目录

---

#### 4. 代码检测逻辑
**文件**: `src-tauri/src/commands/installer.rs`

```rust
fn get_bundled_openclaw_package() -> Option<String> {
    let resource_paths = vec![
        "resources/openclaw/openclaw-zh.tgz",
        "../resources/openclaw/openclaw-zh.tgz",
        "openclaw-zh.tgz",
    ];
    
    for path in resource_paths {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    
    None
}
```

✅ **已实现**

---

## 🔍 验证方法

### 本地验证

#### 1. 下载离线包
```bash
cd src-tauri/resources
./download-resources.sh  # macOS/Linux
# 或
./download-resources.ps1  # Windows
```

#### 2. 检查文件
```bash
ls -lh src-tauri/resources/openclaw/openclaw-zh.tgz
```

**预期输出**: 
```
-rw-r--r--  1 user  staff   15M  openclaw-zh.tgz
```

#### 3. 构建测试
```bash
npm run tauri build
```

#### 4. 检查构建产物
```bash
# macOS
ls -lh src-tauri/target/release/bundle/macos/*.app/Contents/Resources/

# Windows
dir src-tauri\target\release\bundle\msi\*.msi
```

---

### CI/CD 验证

#### 1. 查看构建日志
进入 GitHub Actions → 最新的构建 → 展开 "Download bundled resources" 步骤

**预期输出**:
```
📦 下载 OpenClaw（离线安装，无需 Git）...
  使用 npm pack 打包...
  ✓ 已保存为: openclaw-zh.tgz
```

#### 2. 下载构建产物
在 GitHub Actions 的 Artifacts 中下载：
- `openclaw-manager-macos`
- `openclaw-manager-windows`

#### 3. 解压检查
```bash
# macOS .dmg
hdiutil mount OpenClaw-Manager.dmg
ls -la /Volumes/OpenClaw\ Manager/

# Windows .msi (需要工具)
# 或直接安装后检查程序目录
```

---

## 🎯 打包状态总结

### 当前状态（v0.0.12+）

| 项目 | 状态 | 说明 |
|------|------|------|
| **Tauri 资源配置** | ✅ | 会打包 resources/openclaw/* |
| **CI/CD 脚本** | ✅ | 自动下载离线包 |
| **下载脚本 (sh)** | ✅ | 重命名为统一文件名 |
| **下载脚本 (ps1)** | ✅ | 重命名为统一文件名 |
| **代码检测** | ✅ | 自动识别离线包 |
| **UI 显示** | ✅ | 显示离线/在线状态 |

### 预期效果

✅ **有离线包的版本**:
- Windows 用户：无需 Git，只需 Node.js
- macOS 用户：无需网络，直接安装
- 包体积：增加 ~15MB

✅ **没有离线包的版本**:
- Windows 用户：需要 Git + Node.js
- 所有用户：需要网络连接
- 包体积：不增加

---

## 🚀 发布流程

### 标准发布（含离线包）

```bash
# 1. 修改版本号
# 编辑 Makefile: TAG := 0.0.12

# 2. 发布
make release

# 3. GitHub Actions 自动：
#    - 下载离线包
#    - 构建应用
#    - 打包所有资源
#    - 创建 Release

# 4. 等待 10-15 分钟
# 5. 下载测试
```

### 快速发布（不含离线包）

如果需要快速发布，可以跳过下载步骤：

1. 在 `.github/workflows/build.yml` 中注释掉下载步骤
2. 构建会更快，但用户需要 Git 和网络

---

## 📝 常见问题

### Q: 如何确认离线包已打包？

**A**: 检查三个地方：
1. CI 日志显示 "✓ 已保存为: openclaw-zh.tgz"
2. 安装包体积比之前版本大 ~15MB
3. Windows 用户安装时不再要求 Git

### Q: 离线包在应用中的位置？

**A**: 
- macOS: `OpenClaw Manager.app/Contents/Resources/openclaw-zh.tgz`
- Windows: `C:\Program Files\OpenClaw Manager\resources\openclaw-zh.tgz`

### Q: 如果离线包下载失败怎么办？

**A**: CI 会继续构建，生成不含离线包的版本，用户需要 Git 在线安装。

### Q: 可以手动添加离线包吗？

**A**: 可以，在 `src-tauri/resources/openclaw/` 目录下放置 `openclaw-zh.tgz` 文件后重新构建。

---

## ✅ 最终确认

在发布前确认：

- [ ] `src-tauri/resources/openclaw/openclaw-zh.tgz` 存在（本地测试）
- [ ] CI 日志显示成功下载离线包
- [ ] 构建产物体积符合预期（~25MB macOS, ~23MB Windows）
- [ ] 测试安装不要求 Git（Windows）
- [ ] 验证离线安装功能正常

---

**更新时间**: 2026-02-15  
**当前版本**: v0.0.11+  
**下一版本**: v0.0.12（将包含完整离线包）
