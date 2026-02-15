# Build script with proper Visual Studio environment
# This script loads VS environment and runs the build

param(
    [ValidateSet("dev", "build", "check")]
    [string]$Command = "build"
)

$ErrorActionPreference = "Continue"

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  OpenClaw Manager Build (with VS Env)" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Find Visual Studio installation
Write-Host "Looking for Visual Studio..." -ForegroundColor Yellow

$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"

if (Test-Path $vswhere) {
    Write-Host "Using vswhere to locate VS..." -ForegroundColor Gray
    
    $vsInstallPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    
    if ($vsInstallPath) {
        Write-Host "✅ Found Visual Studio at: $vsInstallPath" -ForegroundColor Green
        
        $vcvarsPath = Join-Path $vsInstallPath "VC\Auxiliary\Build\vcvars64.bat"
        
        if (Test-Path $vcvarsPath) {
            Write-Host "✅ Found vcvars64.bat" -ForegroundColor Green
            Write-Host ""
            Write-Host "Loading Visual Studio environment..." -ForegroundColor Yellow
            
            # Load environment from vcvars64.bat
            cmd /c "`"$vcvarsPath`" && set" | ForEach-Object {
                if ($_ -match '^([^=]+)=(.*)$') {
                    $name = $matches[1]
                    $value = $matches[2]
                    Set-Item -Path "env:$name" -Value $value
                }
            }
            
            Write-Host "✅ Environment loaded!" -ForegroundColor Green
            
            # Verify link.exe is available
            $linkExe = where.exe link.exe 2>$null
            if ($linkExe) {
                Write-Host "✅ link.exe found: $($linkExe[0])" -ForegroundColor Green
            } else {
                Write-Host "⚠️  link.exe still not in PATH" -ForegroundColor Yellow
            }
        }
    }
}

# Add Cargo to PATH
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

Write-Host ""
Write-Host "Running: make $Command" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Run the make command
& make $Command

$exitCode = $LASTEXITCODE

Write-Host ""
if ($exitCode -eq 0) {
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  ✅ Success!" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Cyan
} else {
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  ❌ Failed (Exit code: $exitCode)" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Cyan
}
Write-Host ""

exit $exitCode
