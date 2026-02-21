# PowerShell 脚本：下载打包资源
# 用于 Windows 平台的 CI/CD 或本地开发

$ErrorActionPreference = "Stop"

$NODE_VERSION = "22.12.0"
$OPENCLAW_PACKAGE = "openclaw"

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

# 准备 Git for Windows 离线包（仅使用 portable Git）
Write-Host "📦 准备 Git (Windows 64-bit, portable 优先)..."
Set-Location "git"
$portableFile = "git-portable.zip"
if (-not (Test-Path $portableFile)) {
    $gitCmd = Get-Command git -ErrorAction SilentlyContinue
    if ($gitCmd) {
        try {
            Write-Host "  发现系统 Git，正在打包为 $portableFile ..."
            $gitExe = $gitCmd.Source
            $gitRoot = Split-Path (Split-Path $gitExe -Parent) -Parent
            Compress-Archive -Path "$gitRoot\*" -DestinationPath $portableFile -Force
            Write-Host "  ✓ 已生成: $portableFile"
        } catch {
            Write-Host "  ✗ 打包系统 Git 失败: $_"
        }
    } else {
        Write-Host "  ⚠️  当前系统未检测到 Git，无法自动生成 $portableFile"
        Write-Host "  请手动放置 Git for Windows 的便携版 zip 到 src-tauri/resources/git/ 并命名为 $portableFile"
    }
}

if (Test-Path $portableFile) {
    Write-Host "  ✓ 已存在: $portableFile（跳过）"
}
Set-Location ".."
Write-Host ""

# 下载 OpenClaw（离线安装，无需 Git）
Write-Host "📦 下载 OpenClaw（离线安装，无需 Git）..."
Set-Location "openclaw"

if (Get-Command npm -ErrorAction SilentlyContinue) {
    Write-Host "  使用 npm pack 打包..."
    Remove-Item "*.tgz" -ErrorAction SilentlyContinue
    # 校验缓存并从 registry 获取最新版本（npm 5+ 推荐用 verify 替代 clean）
    npm cache verify
    npm pack "$($OPENCLAW_PACKAGE)@latest" --prefer-online
    
    # npm pack openclaw 生成 openclaw-<version>.tgz，重命名为统一文件名
    $tgzFiles = Get-ChildItem "openclaw-*.tgz"
    if ($tgzFiles.Count -gt 0) {
        $tgzFile = $tgzFiles[0]
        Move-Item -Path $tgzFile.Name -Destination "openclaw.tgz" -Force
        Write-Host "  ✓ 已保存为: openclaw.tgz"
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
    $mb = [math]::Round($_.Length / 1MB, 2); Write-Host ("  " + $_.Name + " - " + $mb + " MB")
}
Write-Host ""
Write-Host "OpenClaw:"
Get-ChildItem "openclaw" -ErrorAction SilentlyContinue | ForEach-Object {
    $mb = [math]::Round($_.Length / 1MB, 2); Write-Host ("  " + $_.Name + " - " + $mb + " MB")
}
Write-Host ""
Write-Host "Git (Windows):"
Get-ChildItem "git" -ErrorAction SilentlyContinue | ForEach-Object {
    $mb = [math]::Round($_.Length / 1MB, 2); Write-Host ("  " + $_.Name + " - " + $mb + " MB")
}
Write-Host ""

Write-Host "Done."
Write-Host ""
