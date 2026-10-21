<#
  Sets up the Rust development environment without admin rights:
  portable MinGW (w64devkit) in the user folder + entry in the user PATH.
  The Rust toolchain itself is installed via rust-toolchain.toml on the first cargo call.
#>
$ErrorActionPreference = 'Stop'

$version = '2.10.1'
$root = Join-Path $env:USERPROFILE 'tools'
$kit = Join-Path $root 'w64devkit'
$bin = Join-Path $kit 'bin'

if (-not (Test-Path (Join-Path $bin 'gcc.exe'))) {
    New-Item -ItemType Directory -Force $root | Out-Null
    $sfx = Join-Path $env:TEMP "w64devkit-$version.7z.exe"
    # --ssl-no-revoke: the corporate proxy offers no revocation check
    curl.exe -fSL --ssl-no-revoke -o $sfx "https://github.com/skeeto/w64devkit/releases/download/v$version/w64devkit-x64-$version.7z.exe"
    & $sfx "-o$root" -y | Out-Null
    Remove-Item $sfx
}

# Rust links against libgcc_eh; w64devkit already has the unwinder in libgcc.a.
$gccLib = Get-ChildItem (Join-Path $kit 'lib\gcc\x86_64-w64-mingw32') -Directory | Select-Object -First 1
$eh = Join-Path $gccLib.FullName 'libgcc_eh.a'
if (-not (Test-Path $eh)) { & (Join-Path $bin 'ar.exe') rcs $eh }

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($userPath -split ';') -notcontains $bin) {
    [Environment]::SetEnvironmentVariable('Path', "$userPath;$bin", 'User')
    Write-Host "PATH extended: $bin  -> fully restart VS Code."
}

# Machine-specific instead of in the repo: "stable" from rust-toolchain.toml resolves to GNU on this machine.
rustup set default-host x86_64-pc-windows-gnu
rustup toolchain install stable --profile minimal

# Corporate proxy offers no revocation check (schannel: CRYPT_E_NO_REVOCATION_CHECK)
[Environment]::SetEnvironmentVariable('CARGO_HTTP_CHECK_REVOKE', 'false', 'User')

Write-Host 'Done. Fully restart VS Code, then: cargo run -p klipper-touch'
