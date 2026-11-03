$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

cargo build --release
New-Item -ItemType Directory -Force -Path dist | Out-Null
Copy-Item -Force "target\release\keystats.exe" "dist\keystats.exe"
Write-Host "已生成 dist\keystats.exe"
