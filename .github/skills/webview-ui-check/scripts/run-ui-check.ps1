<#
  Startet Fake-Moonraker und Tauri-App (mit Debug-Port), fuehrt ui-check.mjs aus und raeumt auf.
  Aufruf (aus dem Repo-Wurzelordner): powershell -File .github\skills\webview-ui-check\scripts\run-ui-check.ps1 [-OutDir <ordner>] [-Viewports 480x320,800x480]
  Voraussetzung: src-tauri\target\debug\klipper-touch.exe (npm run tauri build -- --debug --no-bundle) und fake-moonraker\target\debug\fake-moonraker.exe (cargo build).
#>
param(
    [string]$OutDir = (Join-Path $env:TEMP 'ui-check'),
    [string]$Viewports = '480x320,800x480'
)
$ErrorActionPreference = 'Stop'

$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')).Path
$app = Join-Path $root 'src-tauri\target\debug\klipper-touch.exe'
$fake = Join-Path $root 'fake-moonraker\target\debug\fake-moonraker.exe'
foreach ($binary in $app, $fake) {
    if (-not (Test-Path $binary)) { throw "Fehlt: $binary (zuerst bauen)" }
}

Get-Process klipper-touch, fake-moonraker -ErrorAction SilentlyContinue | Stop-Process -Force
$fakeProcess = Start-Process $fake -PassThru -WindowStyle Hidden
Start-Sleep 1

$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'
$appProcess = Start-Process $app -ArgumentList '-m', 'http://127.0.0.1:7125/', '-w', 'ws://127.0.0.1:7125/websocket' -PassThru
Remove-Item Env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS

try {
    Start-Sleep 12
    node (Join-Path $PSScriptRoot 'ui-check.mjs') $OutDir $Viewports
    Write-Host "Screenshots: $OutDir"
}
finally {
    Stop-Process -Id $appProcess.Id, $fakeProcess.Id -Force -ErrorAction SilentlyContinue
}
