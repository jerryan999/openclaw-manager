# OpenClaw Manager 应用内更新（Updater）

本应用使用 Tauri 的 [updater 插件](https://v2.tauri.app/plugin/updater/)，通过 **GitHub Releases** 实现「检测 → 下载 → 安装」的自动更新。

## 用户侧

- 应用启动后约 3.5 秒会检查是否有新版本（请求 `latest.json`）。
- 若有新版本，顶部会显示**橙色横幅**「发现新版本 OpenClaw Manager x.y.z」，可点击「立即更新」下载并安装，安装完成后自动重启。

## 维护者：发布前必做

### 1. 生成签名密钥（仅需一次）

更新必须经**私钥签名**，客户端用**公钥**校验，无法关闭。

```bash
npm run tauri signer generate -- -w ~/.tauri/openclaw-manager.key
```

会生成两个文件（示例路径）：

- `~/.tauri/openclaw-manager.key` — **私钥**，妥善保管、勿提交到仓库。
- `~/.tauri/openclaw-manager.key.pub` — **公钥**，需写入 `tauri.conf.json`。

### 2. 在 tauri.conf.json 中填入公钥

打开 `src-tauri/tauri.conf.json`，将 `plugins.updater.pubkey` 的占位符替换为**公钥文件中的完整内容**（不是文件路径）：

```json
"plugins": {
  "updater": {
    "pubkey": "CONTENT_OF_.key.pub_FILE_PASTED_HERE",
    "endpoints": ["https://github.com/miaoxworld/openclaw-manager/releases/latest/download/latest.json"],
    "windows": { "installMode": "passive" }
  }
}
```

若仓库不是 `miaoxworld/openclaw-manager`，请同时修改 `endpoints` 中的 URL。

### 3. 在 GitHub 仓库中配置私钥 Secret

1. 仓库 → **Settings** → **Secrets and variables** → **Actions**
2. 新建 **Repository secret**：
   - **Name**: `TAURI_SIGNING_PRIVATE_KEY`
   - **Value**: 私钥文件 `openclaw-manager.key` 的**完整内容**，或填写私钥文件路径（如 `~/.tauri/openclaw-manager.key`，需保证 CI 环境可访问）

推送 **tag**（如 `v0.0.30`）时，GitHub Actions 会使用该 secret 构建并签名，生成 `.sig` 与 macOS 的 `.app.tar.gz`，并生成 `latest.json` 随 Release 一起发布。

## 发布流程简述

1. 本地更新版本号：`npm run version 0.0.30`
2. 提交并推送，打 tag：`git tag v0.0.30 && git push origin v0.0.30`
3. CI 构建 macOS / Windows 安装包并生成签名与 `latest.json`，创建 GitHub Release 并上传所有产物。
4. 用户打开已安装的旧版应用时，会拉取 `latest.json`，发现新版本后即可在应用内一键更新。

## 产物说明

- **macOS**：除 `.dmg` 外，会生成 `*.app.tar.gz` 与 `*.app.tar.gz.sig`，供 updater 使用。
- **Windows**：除 NSIS 的 `.exe` 外，会生成同名的 `.exe.sig`。
- **latest.json**：由 `scripts/generate-updater-latest.cjs` 在 CI 的 release 阶段生成，包含 `version`、`pub_date`、各平台的 `url` 与 `signature`，供客户端检查更新。

## 故障排查

- **应用内提示「无法获取更新」**：多为网络问题或 `endpoints` 中的 URL 无法访问；若为自建仓库，请确认该 URL 在浏览器中可打开。
- **检查更新时报错**：若尚未配置 `pubkey` 或配置错误，插件会校验失败；请确认公钥已正确粘贴到 `tauri.conf.json`。
- **CI 构建失败**：若在 tag 推送时失败，常见原因为未配置 `TAURI_SIGNING_PRIVATE_KEY` 或私钥内容/路径错误。
