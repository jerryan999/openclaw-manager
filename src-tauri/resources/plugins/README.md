# 插件资源

本目录用于存放打包进应用的插件安装包。

## QQ 插件 (qqbot.tgz)

安装 QQ 渠道时会从 `plugins/qqbot.tgz` 解压到 `~/.openclaw/extensions/qqbot` 并执行 `npm install --prod`。

**准备 qqbot.tgz：**

```bash
# 从 npm 打包 @sliverp/qqbot
mkdir -p src-tauri/resources/plugins
cd src-tauri/resources/plugins
npm pack @sliverp/qqbot@latest
# 将生成的 sliverp-qqbot-*.tgz 重命名为 qqbot.tgz
mv sliverp-qqbot-*.tgz qqbot.tgz
```

或从源码打包：

```bash
git clone https://github.com/sliverp/qqbot.git
cd qqbot
npm pack
# 将生成的 sliverp-qqbot-*.tgz 复制到 resources/plugins/ 并重命名为 qqbot.tgz
```

构建应用前请确保此处存在 `qqbot.tgz`，否则安装 QQ 插件时会提示「未找到打包的 QQ 插件」。
