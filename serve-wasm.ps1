# Build guitar-notes for wasm32 and serve www/ with Python's http.server.
# Commands mirror .github/workflows/deploy.yml
#
# Usage:
#   .\serve-wasm.ps1
#   .\serve-wasm.ps1 -Port 8080
#   .\serve-wasm.ps1 -SkipOpt   # skip wasm-opt if Binaryen is not installed
#   .\serve-wasm.ps1 -ServeOnly # skip build, only start the server

param(
    [int] $Port = 8000,
    [switch] $SkipOpt,
    [switch] $ServeOnly
)

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

function Require-Command([string] $Name) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command not found: $Name"
    }
}

if (-not $ServeOnly) {
    Require-Command cargo
    Require-Command wasm-bindgen

    Write-Host "==> Building WASM (wasm-release / wasm32-unknown-unknown)" -ForegroundColor Cyan
    # rustflags for getrandom live in .cargo/config.toml [target.wasm32-unknown-unknown]
    # Do not set $env:RUSTFLAGS — it would stick in the shell and break native cargo check.
    cargo build --profile wasm-release --target wasm32-unknown-unknown
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $wasmDir = "target/wasm32-unknown-unknown/wasm-release"
    $wasmIn = Join-Path $wasmDir "guitar-notes.wasm"
    $wasmOpt = Join-Path $wasmDir "guitar-notes.opt.wasm"

    if (-not $SkipOpt) {
        if (Get-Command wasm-opt -ErrorAction SilentlyContinue) {
            Write-Host "==> Optimizing WASM (wasm-opt -Oz)" -ForegroundColor Cyan
            & wasm-opt -Oz --output $wasmOpt $wasmIn
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
            Move-Item -Force $wasmOpt $wasmIn
        }
        else {
            Write-Host "wasm-opt not found; skipping optimize (pass -SkipOpt to silence this)." -ForegroundColor Yellow
        }
    }
    else {
        Write-Host "==> Skipping wasm-opt" -ForegroundColor Yellow
    }

    Write-Host "==> wasm-bindgen -> www/" -ForegroundColor Cyan
    wasm-bindgen `
        --out-name guitar_notes `
        --out-dir www `
        --target web `
        $wasmIn
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    New-Item -ItemType File -Force -Path "www/.nojekyll" | Out-Null
    Write-Host "==> Build done" -ForegroundColor Green
}

$python = $null
foreach ($candidate in @("python", "py")) {
    if (Get-Command $candidate -ErrorAction SilentlyContinue) {
        $python = $candidate
        break
    }
}
if (-not $python) {
    throw "Python not found. Install Python or add it to PATH."
}

$url = "http://127.0.0.1:$Port/"
Write-Host "==> Serving www/ at $url (Ctrl+C to stop)" -ForegroundColor Cyan

# Open browser shortly after server starts (best-effort).
Start-Job -ScriptBlock {
    param($Url)
    Start-Sleep -Seconds 1
    Start-Process $Url
} -ArgumentList $url | Out-Null

Push-Location www
try {
    if ($python -eq "py") {
        & py -3 -m http.server $Port
    }
    else {
        & python -m http.server $Port
    }
}
finally {
    Pop-Location
}
