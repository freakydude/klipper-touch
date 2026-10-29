<#
  Renders the pages of the Slint UI headless as PNG against a freshly started fake Moonraker.
  Usage (repo root): powershell -File scripts\ui-snapshot.ps1 [-OutDir <folder>] [-Profile voron] [-Size 800x480]
                     [-NoFit] [-Scenes pages,network,printing,shutdown,idle,login|all]
  -NoFit: the window of the given size without scaling the layout to 800x480 (as a desktop window; at least 800x480).
          Without it the layout is fitted to the size like the app does in fullscreen.
  Without -Scenes all scenes except the slow 'idle' and 'login' run; with 'login' the fake wants a login (user test,
  password secret) and the other chosen scenes run logged in. Exit code 0 on success; the PNG paths are printed.
#>
param(
    [string]$OutDir = (Join-Path $env:TEMP 'kt-ui-snapshots'),
    [ValidateSet('basic', 'voron', 'happy-hare', 'ercf')][string]$Profile = 'voron',
    [string]$Size = '800x480',
    [string]$Scenes = '',
    [switch]$NoFit,
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
    $fakeArguments = @('--port', $Port, '--time-scale', '20', '--profile', $Profile)
    if (",$Scenes," -like '*,login,*') { $fakeArguments += @('--auth', 'test:secret') }
    $fake = Start-Process -FilePath 'target\debug\fake-moonraker.exe' -ArgumentList $fakeArguments -PassThru -WindowStyle Hidden
    try {
        Start-Sleep -Milliseconds 800
        $arguments = @('--moonraker', "http://127.0.0.1:$Port", '--out', $OutDir, '--size', $Size)
        if ($Scenes) { $arguments += @('--scenes', $Scenes) }
        if ($NoFit) { $arguments += '--no-fit' }
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
