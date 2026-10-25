<#
  Renders the pages of the Slint UI headless as PNG against a freshly started fake Moonraker.
  Usage (repo root): powershell -File scripts\ui-snapshot.ps1 [-OutDir <folder>] [-Profile voron] [-Size 800x480]
                     [-Scenes pages,network,printing,shutdown,idle|all]
  Without -Scenes all scenes except the slow 'idle' run. Exit code 0 on success; the PNG paths are printed.
#>
param(
    [string]$OutDir = (Join-Path $env:TEMP 'kt-ui-snapshots'),
    [ValidateSet('basic', 'voron', 'happy-hare')][string]$Profile = 'voron',
    [string]$Size = '800x480',
    [string]$Scenes = '',
    [int]$Port = 7125
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Push-Location $root
try {
    $env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' + [Environment]::GetEnvironmentVariable('Path', 'User')
    cargo build -q -p fake-moonraker -p ui-snapshot
    if ($LASTEXITCODE -ne 0) { throw 'build failed' }

    Get-Process fake-moonraker, ui-snapshot -ErrorAction SilentlyContinue | Stop-Process -Force
    if (Test-Path $OutDir) { Remove-Item -Recurse -Force $OutDir }
    $fake = Start-Process -FilePath 'target\debug\fake-moonraker.exe' -ArgumentList '--port', $Port, '--time-scale', '20', '--profile', $Profile -PassThru -WindowStyle Hidden
    try {
        Start-Sleep -Milliseconds 800
        $arguments = @('--moonraker', "http://127.0.0.1:$Port", '--out', $OutDir, '--size', $Size)
        if ($Scenes) { $arguments += @('--scenes', $Scenes) }
        & 'target\debug\ui-snapshot.exe' @arguments
        $code = $LASTEXITCODE
    }
    finally {
        Stop-Process -Id $fake.Id -Force -ErrorAction SilentlyContinue
    }
    exit $code
}
finally {
    Pop-Location
}
