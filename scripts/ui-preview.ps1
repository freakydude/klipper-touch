<#
  Renders the UI previews in crates\app\ui\preview with slint-viewer, without building Rust code.
  Usage (repo root): powershell -File scripts\ui-preview.ps1 [-Name print-printing,temperature] [-OutDir <folder>]
  Downloads the slint-viewer matching the Slint version in Cargo.lock to %USERPROFILE%\tools on first use.
  Exit code 0 on success; the PNG paths are printed.
#>
param(
    [string[]]$Name = @(),
    [string]$OutDir = (Join-Path $env:TEMP 'kt-ui-preview')
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$previewDir = Join-Path $root 'crates\app\ui\preview'

function Get-SlintVersion {
    $lock = Get-Content (Join-Path $root 'Cargo.lock') -Raw
    if ($lock -notmatch '(?m)^name = "slint"\r?\nversion = "([^"]+)"') { throw 'slint not found in Cargo.lock' }
    $Matches[1]
}

function Get-SlintViewer {
    $onPath = Get-Command slint-viewer -ErrorAction SilentlyContinue
    if ($onPath) { return $onPath.Source }
    $version = Get-SlintVersion
    $dir = Join-Path $env:USERPROFILE "tools\slint-viewer-$version"
    $exe = Join-Path $dir 'slint-viewer.exe'
    if (-not (Test-Path $exe)) {
        $zip = Join-Path $env:TEMP "slint-viewer-$version.zip"
        # --ssl-no-revoke: the corporate proxy offers no revocation check
        curl.exe -fsSL --ssl-no-revoke -o $zip "https://github.com/slint-ui/slint/releases/download/v$version/slint-viewer-windows-x86_64.zip"
        if ($LASTEXITCODE -ne 0) { throw "download of slint-viewer $version failed" }
        Expand-Archive -Force $zip $dir
        Remove-Item $zip
    }
    $exe
}

$viewer = Get-SlintViewer
$previews = Get-ChildItem $previewDir -Filter '*.slint' | Where-Object { $_.BaseName -ne 'sample' }
if ($Name.Count -gt 0) { $previews = $previews | Where-Object { $Name -contains $_.BaseName } }
if (-not $previews) { throw "no preview found in $previewDir" }

New-Item -ItemType Directory -Force $OutDir | Out-Null
$failed = 0
foreach ($preview in $previews) {
    $png = Join-Path $OutDir "$($preview.BaseName).png"
    & $viewer --screenshot $png $preview.FullName
    if ($LASTEXITCODE -eq 0) { $png } else { $failed++; Write-Warning "$($preview.Name) failed" }
}
exit $failed
