# PowerShell 脚本：下载打包资源
# 用于 Windows 平台的 CI/CD 或本地开发

$ErrorActionPreference = 'Stop'

$NODE_VERSION = "22.12.0"
$OPENCLAW_PACKAGE = "@jerryan999/openclaw-zh"

Write-Host "=========================================="
Write-Host "  下载打包资源"
Write-Host "=========================================="
Write-Host ""

# 创建目录
New-Item -ItemType Directory -Force -Path "nodejs" | Out-Null
New-Item -ItemType Directory -Force -Path "openclaw" | Out-Null

# 下载 Node.js for Windows
Write-Host "📦 下载 Node.js v$NODE_VERSION..."
Set-Location "nodejs"

$nodeUrl = "https://nodejs.org/dist/v$NODE_VERSION/node-v$NODE_VERSION-win-x64.zip"
$nodeFile = "node-windows-x64.zip"

Write-Host "  - Windows x64"
Write-Host "  从 $nodeUrl 下载..."

try {
    Invoke-WebRequest -Uri $nodeUrl -OutFile $nodeFile -UseBasicParsing
    Write-Host "  ✓ 下载完成: $nodeFile"
} catch {
    Write-Host "  ✗ 下载失败: $_"
}

Set-Location ".."
Write-Host ""

# 下载 OpenClaw（离线安装，无需 Git）
Write-Host "📦 下载 OpenClaw（离线安装，无需 Git）..."
Set-Location "openclaw"

if (Get-Command npm -ErrorAction SilentlyContinue) {
    Write-Host "  使用 npm pack 打包..."
    Remove-Item "*.tgz" -ErrorAction SilentlyContinue
    npm pack $OPENCLAW_PACKAGE
    
    # 重命名为统一的文件名
    $tgzFiles = Get-ChildItem "jerryan999-openclaw-zh-*.tgz"
    if ($tgzFiles.Count -gt 0) {
        $tgzFile = $tgzFiles[0]
        Move-Item -Path $tgzFile.Name -Destination "openclaw-zh.tgz" -Force
        Write-Host "  ✓ 已保存为: openclaw-zh.tgz"
    }
} else {
    Write-Host "  ⚠️  npm 未安装，跳过 OpenClaw 下载"
    Write-Host "  请手动运行: npm pack $OPENCLAW_PACKAGE"
}

Set-Location ".."
Write-Host ""

# 显示下载的文件
Write-Host "=========================================="
Write-Host "  已下载的资源："
Write-Host "=========================================="
Write-Host ""
Write-Host "Node.js:"
Get-ChildItem "nodejs" -ErrorAction SilentlyContinue | ForEach-Object {
    Write-Host "  $($_.Name) - $([math]::Round($_.Length / 1MB, 2)) MB"
}
Write-Host ""
Write-Host "OpenClaw:"
Get-ChildItem "openclaw" -ErrorAction SilentlyContinue | ForEach-Object {
    Write-Host "  $($_.Name) - $([math]::Round($_.Length / 1MB, 2)) MB"
}
Write-Host ""

Write-Host "✅ 完成！"
Write-Host ""
Write-Host "💡 提示："
Write-Host "  - OpenClaw 离线包安装时不需要 Git，更可靠"
Write-Host "  - 开发模式不需要下载所有平台的资源"
Write-Host "  - 生产构建时确保目标平台的资源已下载"
Write-Host "  - 可以在 CI/CD 中运行此脚本自动下载"
Write-Host ""
Write-Host "📦 打包体积影响："
Write-Host "  - Node.js (Windows): ~40-50MB"
Write-Host "  - OpenClaw .tgz: ~10-20MB"
Write-Host "  - 总计: ~50-70MB"
