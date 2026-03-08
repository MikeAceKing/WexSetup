$ErrorActionPreference = "Stop"

$bundleRoot = Join-Path $PSScriptRoot "..\src-tauri\target\release\bundle"
$resolvedBundleRoot = Resolve-Path $bundleRoot

$nsisInstaller = Get-ChildItem -Path (Join-Path $resolvedBundleRoot "nsis") -Filter *.exe -ErrorAction SilentlyContinue |
  Sort-Object LastWriteTime -Descending |
  Select-Object -First 1

$msiInstaller = Get-ChildItem -Path (Join-Path $resolvedBundleRoot "msi") -Filter *.msi -ErrorAction SilentlyContinue |
  Sort-Object LastWriteTime -Descending |
  Select-Object -First 1

if ($nsisInstaller) {
  Write-Host "Launching NSIS installer: $($nsisInstaller.FullName)"
  Start-Process -FilePath $nsisInstaller.FullName -Wait
  exit 0
}

if ($msiInstaller) {
  Write-Host "Launching MSI installer: $($msiInstaller.FullName)"
  Start-Process "msiexec.exe" -ArgumentList "/i `"$($msiInstaller.FullName)`"" -Wait
  exit 0
}

throw "No Windows installer found. Run 'npm run build:windows' first."
