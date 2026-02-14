# QQ 渠道功能更新日志

## 2026-02-14 - 添加 QQ 渠道支持

### ✨ 新功能

#### 1. QQ 渠道配置
- ✅ 添加 QQ 渠道到渠道列表
- ✅ 支持配置 App ID、App Secret、Token
- ✅ 支持私聊策略（配对模式/开放模式/禁用）
- ✅ 支持群组策略（白名单/开放/禁用）
- ✅ 青色主题图标，清晰易识别

#### 2. QQ 插件管理
- ✅ 自动检测 QQ 插件安装状态
- ✅ 一键安装 QQ 插件功能
- ✅ 显示插件版本信息
- ✅ 提供手动安装指引

#### 3. 用户体验优化
- ✅ 插件状态实时检查
- ✅ 安装进度提示
- ✅ NapCat 配置提醒
- ✅ 详细的错误提示和解决方案

### 📝 技术实现

#### 后端 (Rust)
**文件**: `src-tauri/src/commands/config.rs`
- 新增 `QQPluginStatus` 结构体
- 新增 `check_qq_plugin()` 命令 - 检查 QQ 插件状态
- 新增 `install_qq_plugin()` 命令 - 安装 QQ 插件
- 在渠道类型列表中添加 `qq` 类型

**文件**: `src-tauri/src/main.rs`
- 注册 `check_qq_plugin` 和 `install_qq_plugin` Tauri 命令

#### 前端 (React + TypeScript)
**文件**: `src/components/Channels/index.tsx`
- 新增 `QQPluginStatus` 接口定义
- 新增 QQ 插件状态管理 (useState)
- 新增 `checkQQPlugin()` 函数 - 检查插件状态
- 新增 `handleInstallQQPlugin()` 函数 - 处理插件安装
- 在 `channelInfo` 中添加 QQ 渠道配置
- 添加 QQ 插件状态提示 UI 组件
- 选择 QQ 渠道时自动检查插件状态

#### 文档
- 新增 `docs/QQ_CHANNEL_SETUP.md` - QQ 渠道配置指南
- 更新 `README.md` - 添加插件说明和 QQ 渠道介绍
- 新增 `CHANGELOG_QQ.md` - 本更新日志

### 🔍 渠道类型对比

#### 内置渠道（7个）
无需额外安装插件，直接配置即可使用：
1. Telegram
2. Discord
3. Slack
4. WhatsApp
5. iMessage
6. 微信 (WeChat)
7. 钉钉 (DingTalk)

#### 插件渠道（2个）
需要通过 `openclaw plugins install` 安装：
1. **飞书 (Feishu)** - `@m1heng-clawd/feishu`
2. **QQ** - `@openclaw/qq`

### 📋 QQ 渠道配置字段

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `appId` | text | ✅ | QQ 机器人 App ID |
| `appSecret` | password | ✅ | QQ 机器人 App Secret |
| `token` | password | ❌ | QQ 机器人 Token（用于验证） |
| `dmPolicy` | select | ❌ | 私聊策略：配对模式/开放模式/禁用 |
| `groupPolicy` | select | ❌ | 群组策略：白名单/开放/禁用 |

### 🏗️ QQ 渠道架构

```
┌─────────────┐
│  QQ 客户端   │
└──────┬──────┘
       │
┌──────▼──────────────────────┐
│  NapCat (OneBot v11 服务器)  │
└──────┬──────────────────────┘
       │ WebSocket
┌──────▼───────────────┐
│  OpenClaw QQ 插件     │
└──────┬───────────────┘
       │
┌──────▼─────────────┐
│  OpenClaw Gateway  │
└──────┬─────────────┘
       │
┌──────▼────────┐
│   AI Agent    │
└───────────────┘
```

### 📦 安装方式

#### 方式一：UI 安装（推荐）
1. 打开 OpenClaw Manager
2. 进入"消息渠道"页面
3. 选择"QQ"渠道
4. 点击"一键安装插件"

#### 方式二：命令行安装
```bash
openclaw plugins install @openclaw/qq
```

### ⚠️ 注意事项

1. **NapCat 必须单独安装**
   - QQ 插件只是 OpenClaw 与 QQ 的桥梁
   - 实际的 QQ 连接需要 NapCat 作为 OneBot v11 服务器

2. **插件包名可能不同**
   - 本实现假设插件包名为 `@openclaw/qq`
   - 实际包名请参考 OpenClaw 官方文档或插件注册表

3. **配置后需重启**
   - 插件配置更改后需要重启 OpenClaw Gateway

4. **安全性**
   - 插件在 Gateway 进程内运行
   - 仅安装信任的插件

### 🐛 故障排查

#### 插件未找到
```bash
# 检查插件列表
openclaw plugins list

# 手动安装
openclaw plugins install @openclaw/qq

# 检查插件目录
ls ~/.openclaw/extensions/
```

#### NapCat 连接问题
- 确认 NapCat 正在运行
- 检查 WebSocket 地址和端口
- 验证防火墙设置

#### QQ 登录失败
- 确保 NapCat 已成功登录 QQ
- 检查 QQ 账号权限
- 查看 NapCat 日志

### 📚 参考资料

- [OpenClaw 官方插件文档](https://docs.clawd.bot/plugins)
- [NapCat 官方文档](https://napneko.github.io/)
- [OneBot v11 协议规范](https://github.com/botuniverse/onebot-11)
- [QQ 开放平台](https://q.qq.com/)

### 🔄 代码变更统计

- **新增文件**: 2 个（配置文档 + 更新日志）
- **修改文件**: 4 个
  - `src-tauri/src/commands/config.rs` (+76 行)
  - `src-tauri/src/main.rs` (+2 行)
  - `src/components/Channels/index.tsx` (+78 行)
  - `README.md` (+15 行)

### ✅ 测试清单

- [ ] QQ 渠道在渠道列表中显示
- [ ] 点击 QQ 渠道后自动检查插件状态
- [ ] 插件未安装时显示警告提示
- [ ] 一键安装按钮功能正常
- [ ] 安装成功后显示绿色状态
- [ ] 配置字段正确保存
- [ ] 快速测试功能可用
- [ ] 清空配置功能正常

---

**作者**: OpenClaw Manager Team  
**日期**: 2026-02-14  
**版本**: v1.0.0-qq-support
