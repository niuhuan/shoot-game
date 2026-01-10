# PowerShell 构建脚本 (Windows)

param(
    [Parameter(Position=0)]
    [string]$Command = "help"
)

$ErrorActionPreference = "Stop"

Write-Host "🎮 几何射击 - 构建脚本" -ForegroundColor Cyan
Write-Host "========================" -ForegroundColor Cyan

$FontUrl = "https://github.com/notofonts/noto-cjk/raw/refs/heads/main/Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf"
$FontPath = Join-Path "assets" "NotoSansCJKsc-Regular.otf"
$FontFullPath = Join-Path "assets" "NotoSansCJKsc-Regular.full.otf"

function Test-Command($cmdname) {
    return [bool](Get-Command -Name $cmdname -ErrorAction SilentlyContinue)
}

function Ensure-Font {
    if (-not (Test-Path "assets")) {
        New-Item -ItemType Directory -Path "assets" | Out-Null
    }

    if (Test-Path $FontPath) {
        $info = Get-Item $FontPath
        # 如果已经是小体积子集字体，直接用；否则尝试重建子集
        if ($info.Length -gt 0 -and $info.Length -le 6000000) { return }

        if ((-not (Test-Path $FontFullPath)) -or ((Get-Item $FontFullPath).Length -le 0)) {
            Copy-Item -Force $FontPath $FontFullPath
        }
    }

    if (-not (Test-Path $FontFullPath) -or ((Get-Item $FontFullPath).Length -le 0)) {
        Write-Host "📥 字体缺失，正在下载: $FontFullPath" -ForegroundColor Yellow
        $tmp = "$FontFullPath.download"

        try {
            Invoke-WebRequest -Uri $FontUrl -OutFile $tmp -UseBasicParsing
        } catch {
            Write-Host "错误: 字体下载失败: $($_.Exception.Message)" -ForegroundColor Red
            Write-Host "请手动下载并放到 $FontFullPath" -ForegroundColor Red
            exit 1
        }

        if (-not (Test-Path $tmp)) {
            Write-Host "错误: 字体下载失败（未生成文件）" -ForegroundColor Red
            exit 1
        }

        $len = (Get-Item $tmp).Length
        if ($len -le 0) {
            Remove-Item -Force $tmp -ErrorAction SilentlyContinue
            Write-Host "错误: 字体下载失败（文件为空）" -ForegroundColor Red
            exit 1
        }

        Move-Item -Force $tmp $FontFullPath
        Write-Host "✓ 字体下载完成" -ForegroundColor Green
    }

    # 尝试生成子集字体（没有 fontTools 就退化为直接复制）
    if (Test-Command "python") {
        try {
            # 缺少 fontTools 时尝试安装（失败则回退）
            $hasFontTools = $false
            try { python -c "import fontTools.subset" | Out-Null; $hasFontTools = $true } catch { $hasFontTools = $false }
            if (-not $hasFontTools) {
                try { python -m pip install --user -q fonttools | Out-Null } catch {}
            }
            python tools/subset_font.py --input $FontFullPath --output $FontPath --roots "src" --roots "web" | Out-Null
        } catch {
            Copy-Item -Force $FontFullPath $FontPath
        }
    } else {
        Copy-Item -Force $FontFullPath $FontPath
    }

    if (Test-Path $FontPath) {
        $len = (Get-Item $FontPath).Length
        Write-Host ("✓ 字体就绪: {0} ({1} bytes)" -f $FontPath, $len) -ForegroundColor Green
    }
}

function Install-Deps {
    Write-Host "📦 检查依赖..." -ForegroundColor Blue
    
    if (-not (Test-Command "cargo")) {
        Write-Host "错误: 未找到 cargo，请先安装 Rust" -ForegroundColor Red
        exit 1
    }
    
    # 检查 wasm32 目标
    $targets = rustup target list --installed
    if ($targets -notcontains "wasm32-unknown-unknown") {
        Write-Host "安装 wasm32-unknown-unknown 目标..." -ForegroundColor Yellow
        rustup target add wasm32-unknown-unknown
    }
    
    # 检查 wasm-bindgen-cli
    if (-not (Test-Command "wasm-bindgen")) {
        Write-Host "安装 wasm-bindgen-cli..." -ForegroundColor Yellow
        cargo install wasm-bindgen-cli
    }

    Ensure-Font
    
    Write-Host "✓ 依赖检查完成" -ForegroundColor Green
}

function Build-Native {
    Write-Host "🔨 构建原生版本..." -ForegroundColor Blue
    Ensure-Font
    cargo build --release
    Write-Host "✓ 原生版本构建完成" -ForegroundColor Green
}

function Build-Wasm {
    Write-Host "🔨 构建 WASM 版本..." -ForegroundColor Blue
    Ensure-Font
    
    # 构建 WASM
    cargo build --release --target wasm32-unknown-unknown --no-default-features --features web
    
    # 创建输出目录
    if (-not (Test-Path "dist")) {
        New-Item -ItemType Directory -Path "dist" | Out-Null
    }
    
    # 生成绑定
    Write-Host "📎 生成 wasm-bindgen 绑定..." -ForegroundColor Blue
    wasm-bindgen `
        --out-dir dist `
        --target web `
        --no-typescript `
        target/wasm32-unknown-unknown/release/shoot.wasm
    
    # 复制 web 资源
    Write-Host "📁 复制 web 资源..." -ForegroundColor Blue
    Copy-Item "web/index.html" -Destination "dist/"
    Copy-Item "web/style.css" -Destination "dist/"
    Copy-Item -Recurse "assets" -Destination "dist/" -Force
    $fullInDist = Join-Path "dist" "assets" "NotoSansCJKsc-Regular.full.otf"
    if (Test-Path $fullInDist) {
        Remove-Item -Force $fullInDist
    }
    
    Write-Host "✓ WASM 版本构建完成" -ForegroundColor Green
    Write-Host "输出目录: dist/" -ForegroundColor Blue
}

function Start-Server {
    Write-Host "🌐 启动本地服务器..." -ForegroundColor Blue
    Write-Host "访问 http://localhost:4000" -ForegroundColor Green
    Write-Host "按 Ctrl+C 停止服务器" -ForegroundColor Yellow
    
    if (Test-Command "basic-http-server") {
        basic-http-server dist -a 0.0.0.0:4000
    } elseif (Test-Command "python") {
        Set-Location dist
        python -m http.server 4000
    } else {
        Write-Host "错误: 未找到 HTTP 服务器" -ForegroundColor Red
        exit 1
    }
}

function Start-Native {
    Write-Host "🎮 运行原生版本..." -ForegroundColor Blue
    Ensure-Font
    cargo run --release
}

function Clear-Build {
    Write-Host "🧹 清理构建文件..." -ForegroundColor Blue
    cargo clean
    if (Test-Path "dist") {
        Remove-Item -Recurse -Force "dist"
    }
    Write-Host "✓ 清理完成" -ForegroundColor Green
}

function Show-Help {
    Write-Host @"
用法: .\build.ps1 [命令]

命令:
  deps      安装/检查依赖
  native    构建原生版本
  wasm      构建 WASM 版本
  serve     启动本地服务器测试 WASM
  run       运行原生版本
  all       构建所有版本
  clean     清理构建文件
  help      显示帮助

示例:
  .\build.ps1 deps      # 安装依赖
  .\build.ps1 wasm      # 构建 WASM
  .\build.ps1 serve     # 本地测试
"@
}

# 切换到脚本目录
Set-Location $PSScriptRoot

switch ($Command.ToLower()) {
    "deps" { Install-Deps }
    "native" { Build-Native }
    "wasm" { Install-Deps; Build-Wasm }
    "serve" {
        if (-not (Test-Path "dist")) {
            Write-Host "dist 目录不存在，先构建 WASM..." -ForegroundColor Yellow
            Build-Wasm
        }
        Start-Server
    }
    "run" { Start-Native }
    "all" {
        Install-Deps
        Build-Native
        Build-Wasm
        Write-Host "✓ 所有构建完成" -ForegroundColor Green
    }
    "clean" { Clear-Build }
    "help" { Show-Help }
    default {
        Write-Host "未知命令: $Command" -ForegroundColor Red
        Show-Help
        exit 1
    }
}
