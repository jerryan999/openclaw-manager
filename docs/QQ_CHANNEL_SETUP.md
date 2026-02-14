# QQ 渠道配置指南

## 概述

QQ 渠道使用 [**@sliverp/qqbot**](https://github.com/sliverp/qqbot) 插件，通过 **QQ 开放平台** 的官方长连接事件订阅接收消息与事件，无需公网 Webhook，稳定安全。

## 插件与配置参数

- **插件包名**: `@sliverp/qqbot`
- **渠道 ID**: `qqbot`
- **配置参数**（写入 `channels.qqbot`）：

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `enabled` | boolean | - | 是否启用，保存时自动为 `true` |
| `appId` | string | ✅ | QQ 开放平台机器人的 AppID |
| `clientSecret` | string | ✅ | QQ 开放平台机器人的 AppSecret（注意字段名为 clientSecret） |

插件 [openclaw.plugin.json](https://github.com/sliverp/qqbot/blob/main/openclaw.plugin.json) 中 `configSchema` 为空对象，实际配置按上述字段写在 `channels.qqbot` 下即可。

## 渠道插件对比

### 内置渠道（无需额外插件）
- ✅ Telegram、Discord、Slack、WhatsApp、iMessage、微信、钉钉

### 需要安装插件的渠道
- 🔌 **飞书**: `@m1heng-clawd/feishu`
- 🔌 **QQ**: `@sliverp/qqbot`

## 配置步骤

### 1. 安装 QQ 插件

#### 方式一：使用 OpenClaw Manager（推荐）
1. 打开 OpenClaw Manager → 消息渠道
2. 选择「QQ」渠道
3. 点击「一键安装插件」，等待完成

#### 方式二：命令行安装
```bash
openclaw plugins install @sliverp/qqbot@latest
```

或从源码安装：
```bash
git clone https://github.com/sliverp/qqbot.git && cd qqbot
openclaw plugins install .
```

### 2. 在 QQ 开放平台创建机器人

1. 打开 [QQ 开放平台](https://q.qq.com/)，使用**新账号**注册（个人 QQ 不能直接登录）。
2. 完成实名与管理员设置后，在「QQ 机器人」页面创建机器人。
3. 在机器人管理页获取 **AppID** 和 **AppSecret**，妥善保存（AppSecret 仅首次或重置时可见）。
4. 在「开发管理」→「沙箱配置」中配置私聊：选择「在消息列表中配置」，添加成员后，用对应成员的 QQ 扫码添加机器人。  
   ⚠️ 当前 QQ 开放平台仅支持私聊，不支持 QQ 群。

### 3. 在 OpenClaw 中配置

#### 方式一：OpenClaw Manager（推荐）
1. 消息渠道 → 选择 QQ
2. 填写 **App ID**（对应 appId）
3. 填写 **App Secret**（对应 clientSecret）
4. 保存配置

#### 方式二：命令行
```bash
openclaw channels add --channel qqbot --token "AppID:AppSecret"
```

#### 方式三：编辑配置文件
编辑 `~/.openclaw/openclaw.json`：

```json
{
  "channels": {
    "qqbot": {
      "enabled": true,
      "appId": "你的 AppID",
      "clientSecret": "你的 AppSecret"
    }
  }
}
```

### 4. 启动与测试

1. 启动网关：`openclaw gateway`
2. 在 QQ 中与机器人私聊测试

## 架构说明

```
QQ 客户端
    ↓ 官方长连接事件订阅
QQ 开放平台
    ↓
@sliverp/qqbot 插件
    ↓
OpenClaw Gateway
    ↓
AI Agent
```

无需 NapCat 或 OneBot v11 服务器。

## 故障排查

### 插件未找到
- 执行：`openclaw plugins install @sliverp/qqbot@latest`
- 检查：`openclaw plugins list` 是否包含 qqbot
- 修改配置后重启：`openclaw gateway`

### 机器人无响应 / 提示「去火星了」
- 确认 `channels.qqbot` 中 `appId`、`clientSecret` 正确（字段名必须是 `clientSecret`）
- 确认已在 QQ 开放平台沙箱中添加成员并用该成员 QQ 扫码添加机器人

### 升级插件
```bash
openclaw plugins upgrade @sliverp/qqbot@latest
```

## 参考链接

- [@sliverp/qqbot 仓库](https://github.com/sliverp/qqbot)
- [OpenClaw 插件文档](https://docs.clawd.bot/plugins)
- [QQ 开放平台](https://q.qq.com/)

## 注意事项

1. QQ 渠道依赖 `@sliverp/qqbot` 插件，需先安装。
2. 配置字段为 `appId` 与 `clientSecret`（不是 appSecret）。
3. 修改插件或渠道配置后需重启 OpenClaw Gateway。
