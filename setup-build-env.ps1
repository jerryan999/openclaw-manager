# OpenClaw Manager 构建环境一键配置脚本
# 版本: 1.0
# 用途: 检查并配置构建环境

$ErrorActionPreference = 'Continue'

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  🛠️  OpenClaw Manager 构建环境配置" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 1. 检查 Node.js
Write-Host "📦 检查 Node.js..." -ForegroundColor Yellow
try {
    $nodeVersion = node --version 2>$null
    if ($nodeVersion) {
        Write-Host "  ✅ Node.js: $nodeVersion" -ForegroundColor Green
    } else {
        throw "Not found"
    }
} catch {
    Write-Host "  ❌ Node.js 未安装" -ForegroundColor Red
    Write-Host "     请访问: https://nodejs.org/" -ForegroundColor White
    $needsNodeJs = $true
}

# 2. 检查 npm
Write-Host "📦 检查 npm..." -ForegroundColor Yellow
try {
    $npmVersion = npm --version 2>$null
    if ($npmVersion) {
        Write-Host "  ✅ npm: v$npmVersion" -ForegroundColor Green
    } else {
        throw "Not found"
    }
} catch {
    Write-Host "  ❌ npm 未安装" -ForegroundColor Red
}

# 3. 检查 Rust
Write-Host "🦀 检查 Rust..." -ForegroundColor Yellow
try {
    $rustVersion = rustc --version 2>$null
    if ($rustVersion) {
        Write-Host "  ✅ Rust: $rustVersion" -ForegroundColor Green
        $hasRust = $true
    } else {
        throw "Not found"
    }
} catch {
    Write-Host "  ❌ Rust 未安装" -ForegroundColor Red
    $needsRust = $true
}

# 4. 检查 Cargo
Write-Host "📦 检查 Cargo..." -ForegroundColor Yellow
try {
    $cargoVersion = cargo --version 2>$null
    if ($cargoVersion) {
        Write-Host "  ✅ Cargo: $cargoVersion" -ForegroundColor Green
    } else {
        throw "Not found"
    }
} catch {
    Write-Host "  ❌ Cargo 未安装" -ForegroundColor Red
}

# 5. 检查 C++ Build Tools
Write-Host "🔧 检查 C++ Build Tools..." -ForegroundColor Yellow
$clPath = where.exe cl 2>$null
if ($clPath) {
    Write-Host "  ✅ C++ Build Tools 已安装" -ForegroundColor Green
    $hasCppTools = $true
} else {
    Write-Host "  ⚠️  C++ Build Tools 未检测到" -ForegroundColor Yellow
    Write-Host "     (某些 Rust 依赖编译时可能需要)" -ForegroundColor Gray
    $needsCppTools = $true
}

# 6. 检查资源文件
Write-Host "📦 检查资源文件..." -ForegroundColor Yellow

$nodeResourcePath = "src-tauri\resources\nodejs\node-windows-x64.zip"
if (Test-Path $nodeResourcePath) {
    $size = [math]::Round((Get-Item $nodeResourcePath).Length / 1MB, 2)
    Write-Host "  ✅ Node.js 资源: $size MB" -ForegroundColor Green
    $hasNodeResource = $true
} else {
    Write-Host "  ❌ Node.js 资源未下载" -ForegroundColor Red
    $needsResources = $true
}

$openclawResourcePath = "src-tauri\resources\openclaw\openclaw.tgz"
if (Test-Path $openclawResourcePath) {
    $size = [math]::Round((Get-Item $openclawResourcePath).Length / 1MB, 2)
    Write-Host "  ✅ OpenClaw 资源: $size MB" -ForegroundColor Green
    $hasOpenclawResource = $true
} else {
    Write-Host "  ❌ OpenClaw 资源未下载" -ForegroundColor Red
    $needsResources = $true
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  📊 环境检查总结" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 总结
$allReady = -not ($needsNodeJs -or $needsRust -or $needsResources)

if ($allReady) {
    Write-Host "✅ 所有环境已准备就绪！" -ForegroundColor Green
    if (-not $hasCppTools) {
        Write-Host "⚠️  建议安装 C++ Build Tools (可选)" -ForegroundColor Yellow
    }
    Write-Host ""
    Write-Host "🚀 下一步:" -ForegroundColor Cyan
    Write-Host "   npm run tauri:build" -ForegroundColor White
} else {
    Write-Host "❌ 需要安装以下工具:" -ForegroundColor Red
    Write-Host ""
    
    if ($needsNodeJs) {
        Write-Host "  1️⃣  Node.js (必需)" -ForegroundColor Yellow
        Write-Host "     https://nodejs.org/" -ForegroundColor Gray
        Write-Host ""
    }
    
    if ($needsRust) {
        Write-Host "  2️⃣  Rust (必需)" -ForegroundColor Yellow
        Write-Host "     https://rustup.rs/" -ForegroundColor Gray
        Write-Host ""
        Write-Host "     或运行: " -ForegroundColor Gray
        Write-Host "     Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile 'rustup-init.exe'" -ForegroundColor Cyan
        Write-Host "     .\rustup-init.exe" -ForegroundColor Cyan
        Write-Host ""
    }
    
    if ($needsCppTools) {
        Write-Host "  3️⃣  C++ Build Tools (推荐)" -ForegroundColor Yellow
        Write-Host "     https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Gray
        Write-Host ""
    }
    
    if ($needsResources) {
        Write-Host "  4️⃣  下载资源文件 (必需)" -ForegroundColor Yellow
        Write-Host "     cd src-tauri\resources" -ForegroundColor Cyan
        Write-Host "     .\download-resources.ps1" -ForegroundColor Cyan
        Write-Host ""
    }
    
    Write-Host "📝 详细说明: BUILD_ENVIRONMENT_SETUP.md" -ForegroundColor Gray
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
