# Development script with Visual Studio environment
# Automatically loads VS Build Tools environment before running make commands

param(
    [string]$Command = "dev"
)

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  OpenClaw Manager - VS Environment" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Find Visual Studio Build Tools
$vsPaths = @(
    "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
    "C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
    "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat",
    "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat"
)

$vsPath = $null
foreach ($path in $vsPaths) {
    if (Test-Path $path) {
        $vsPath = $path
        break
    }
}

if (-not $vsPath) {
    Write-Host "❌ Visual Studio Build Tools not found!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please install from:" -ForegroundColor Yellow
    Write-Host "  https://visualstudio.microsoft.com/downloads/" -ForegroundColor White
    Write-Host ""
    Write-Host "Or use:" -ForegroundColor Yellow
    Write-Host "  Developer PowerShell for VS 2022" -ForegroundColor White
    Write-Host ""
    exit 1
}

Write-Host "✅ Found VS Build Tools" -ForegroundColor Green
Write-Host "   $vsPath" -ForegroundColor Gray
Write-Host ""
Write-Host "Loading environment..." -ForegroundColor Yellow

# Load VS environment variables
$envVars = cmd /c "`"$vsPath`" && set" | Out-String
$envVars -split "`r`n" | ForEach-Object {
    if ($_ -match '^([^=]+)=(.*)') {
        $name = $matches[1]
        $value = $matches[2]
        [System.Environment]::SetEnvironmentVariable($name, $value, 'Process')
    }
}

Write-Host "✅ Environment loaded!" -ForegroundColor Green
Write-Host ""

# Verify link.exe is available
$linkExe = where.exe link.exe 2>$null
if ($linkExe) {
    Write-Host "✅ link.exe found: $linkExe" -ForegroundColor Green
} else {
    Write-Host "⚠️  link.exe not found in PATH" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Running: make $Command" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Run the make command
& make $Command

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Done!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
