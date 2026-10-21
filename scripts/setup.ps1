<#
  Richtet die Rust-Entwicklungsumgebung ohne Admin-Rechte ein:
  portables MinGW (w64devkit) im Benutzerordner + Eintrag im Benutzer-PATH.
  Der Rust-Toolchain selbst kommt per rust-toolchain.toml beim ersten cargo-Aufruf.
#>
$ErrorActionPreference = 'Stop'

$version = '2.10.1'
$root = Join-Path $env:USERPROFILE 'tools'
$kit = Join-Path $root 'w64devkit'
$bin = Join-Path $kit 'bin'

if (-not (Test-Path (Join-Path $bin 'gcc.exe'))) {
    New-Item -ItemType Directory -Force $root | Out-Null
    $sfx = Join-Path $env:TEMP "w64devkit-$version.7z.exe"
    # --ssl-no-revoke: der Firmen-Proxy bietet keine Sperrlisten-Pruefung
    curl.exe -fSL --ssl-no-revoke -o $sfx "https://github.com/skeeto/w64devkit/releases/download/v$version/w64devkit-x64-$version.7z.exe"
    & $sfx "-o$root" -y | Out-Null
    Remove-Item $sfx
}

# Rust linkt gegen libgcc_eh; w64devkit hat den Unwinder bereits in libgcc.a.
$gccLib = Get-ChildItem (Join-Path $kit 'lib\gcc\x86_64-w64-mingw32') -Directory | Select-Object -First 1
$eh = Join-Path $gccLib.FullName 'libgcc_eh.a'
if (-not (Test-Path $eh)) { & (Join-Path $bin 'ar.exe') rcs $eh }

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($userPath -split ';') -notcontains $bin) {
    [Environment]::SetEnvironmentVariable('Path', "$userPath;$bin", 'User')
    Write-Host "PATH ergaenzt: $bin  -> VS Code komplett neu starten."
}

# Rechnerspezifisch statt im Repo: "stable" aus rust-toolchain.toml loest auf diesem Rechner zu GNU auf.
rustup set default-host x86_64-pc-windows-gnu
rustup toolchain install stable --profile minimal

# Firmen-Proxy bietet keine Sperrlisten-Pruefung (schannel: CRYPT_E_NO_REVOCATION_CHECK)
[Environment]::SetEnvironmentVariable('CARGO_HTTP_CHECK_REVOKE', 'false', 'User')

Write-Host 'Fertig. VS Code komplett neu starten, danach: cargo run -p klipper-touch-slint'
