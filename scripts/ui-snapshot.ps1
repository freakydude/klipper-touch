<#
  Rendert alle Seiten der Slint-UI ohne Fenster als PNG gegen einen frisch gestarteten Fake-Moonraker.
  Aufruf (Repo-Wurzel): powershell -File scripts\ui-snapshot.ps1 [-OutDir <ordner>] [-Profile voron] [-Size 800x480]
  Exit-Code 0 bei Erfolg; die PNG-Pfade stehen in der Ausgabe.
#>
param(
    [string]$OutDir = (Join-Path $env:TEMP 'kt-ui-snapshots'),
    [ValidateSet('basic', 'voron', 'happy-hare')][string]$Profile = 'voron',
    [string]$Size = '800x480',
    [int]$Port = 7125
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Push-Location $root
try {
    $env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' + [Environment]::GetEnvironmentVariable('Path', 'User')
    cargo build -q -p fake-moonraker -p klipper-touch --bins
    if ($LASTEXITCODE -ne 0) { throw 'build failed' }

    Get-Process fake-moonraker, ui-snapshot -ErrorAction SilentlyContinue | Stop-Process -Force
    if (Test-Path $OutDir) { Remove-Item -Recurse -Force $OutDir }
    $fake = Start-Process -FilePath 'target\debug\fake-moonraker.exe' -ArgumentList '--port', $Port, '--time-scale', '20', '--profile', $Profile -PassThru -WindowStyle Hidden
    try {
        Start-Sleep -Milliseconds 800
        & 'target\debug\ui-snapshot.exe' --moonraker "http://127.0.0.1:$Port" --out $OutDir --size $Size
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
