# Git 离线资源（仅 Windows）

本目录用于存放 Windows 64 位 MinGit/Portable Git 的 **.zip** 包，供应用在无系统 Git 时使用。

## 如何添加

在项目根目录的 `src-tauri/resources` 下执行：

```powershell
.\download-resources.ps1
```

会将 MinGit 下载为 `git-windows-x64.zip`。

或从 [Git for Windows Releases](https://github.com/git-for-windows/git/releases) 下载 **MinGit-*-64-bit.zip**，放入本目录：

- **推荐命名**：`git-windows-x64.zip`（与 download 脚本一致）
- 也可不重命名：本目录下任意 **.zip** 均会被识别为 Git 包（如 `MinGit-2.53.0-64-bit.zip`）

## 说明

- 应用仅支持 **.zip** 格式（不支持 .7z）；需为**压缩包文件**，不要放解压后的文件夹。
- 不放入任何文件时，应用会使用系统 PATH 中的 Git（若存在）。
