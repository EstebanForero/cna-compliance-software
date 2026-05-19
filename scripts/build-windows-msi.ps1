$ErrorActionPreference = "Stop"

Write-Host "Checking toolchain..."
bun --version | Out-Host
rustc --version | Out-Host

Write-Host "Installing dependencies..."
bun install

Write-Host "Running Rust tests..."
Push-Location src-tauri
cargo test
Pop-Location

Write-Host "Building Windows MSI..."
bun run build:windows:msi

Write-Host ""
Write-Host "MSI output:"
Get-ChildItem -Recurse -Path "src-tauri\target\release\bundle\msi" -Filter "*.msi" |
  Select-Object -ExpandProperty FullName
