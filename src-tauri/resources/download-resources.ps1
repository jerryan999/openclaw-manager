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
New-Item -ItemType Directory -Force -Path "git" | Out-Null

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

# 下载 MinGit for Windows（可选，用于离线 Git）
Write-Host "📦 下载 MinGit (Windows 64-bit)..."
Set-Location "git"
$GIT_VERSION = "2.53.0"
$gitUrl = "https://github.com/git-for-windows/git/releases/download/v$GIT_VERSION.windows.1/MinGit-$GIT_VERSION-64-bit.zip"
$gitFile = "git-windows-x64.zip"
if (-not (Test-Path $gitFile)) {
    try {
        Write-Host "  从 $gitUrl 下载..."
        Invoke-WebRequest -Uri $gitUrl -OutFile $gitFile -UseBasicParsing
        Write-Host "  ✓ 下载完成: $gitFile"
    } catch {
        Write-Host "  ✗ 下载失败: $_"
        Write-Host "  可手动从 https://github.com/git-for-windows/git/releases 下载 MinGit-*-64-bit.zip 并重命名为 $gitFile"
    }
} else {
    Write-Host "  ✓ 已存在: $gitFile（跳过）"
}
Set-Location ".."
Write-Host ""

# 下载 OpenClaw（离线安装，无需 Git）
Write-Host "📦 下载 OpenClaw（离线安装，无需 Git）..."
Set-Location "openclaw"

if (Get-Command npm -ErrorAction SilentlyContinue) {
    Write-Host "  使用 npm pack 打包..."
    Remove-Item "*.tgz" -ErrorAction SilentlyContinue
    # 强制清除缓存并从 registry 获取最新版本
    npm cache clean --force 2>$null
    npm pack "$($OPENCLAW_PACKAGE)@latest" --prefer-online
    
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
Write-Host "Git (Windows):"
Get-ChildItem "git" -ErrorAction SilentlyContinue | ForEach-Object {
    Write-Host "  $($_.Name) - $([math]::Round($_.Length / 1MB, 2)) MB"
}
Write-Host ""

Write-Host "Done."
Write-Host ""
Write-Host "Tips:"
Write-Host "  - OpenClaw offline install does not require Git"
Write-Host "  - For full offline: put Git zip at resources/git/git-windows-x64.zip"
Write-Host "  - Dev mode: not all platform resources are required"
Write-Host "  - Production: ensure target platform resources are downloaded"
Write-Host "  - Can run this script in CI/CD"
Write-Host ""
Write-Host "Size: Node 40-50MB, OpenClaw 10-20MB, MinGit 10-15MB"
