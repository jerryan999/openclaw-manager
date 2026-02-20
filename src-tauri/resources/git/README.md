# Git 离线资源（仅 Windows）

本目录用于存放 Windows 64 位 MinGit/Portable Git 的 **.zip** 包，供应用在无系统 Git 时使用。

## 如何添加

在项目根目录的 `src-tauri/resources` 下执行：

```powershell
.\download-resources.ps1
```

会将 MinGit 下载为 `git-windows-x64.zip`。

或从 [Git for Windows Releases](https://github.com/git-for-windows/git/releases) 下载 **MinGit-*-64-bit.zip**，放入本目录并命名为以下之一：

- `git-windows-x64.zip`
- `git-portable.zip`
- `PortableGit.zip`

## 说明

- 应用仅支持 **.zip** 格式（不支持 .7z）。
- 不放入任何文件时，应用会使用系统 PATH 中的 Git（若存在）。
