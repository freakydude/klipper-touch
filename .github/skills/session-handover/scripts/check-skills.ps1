<#
  Prueft die Skills unter .github\skills auf formale Fehler und tote Verweise.
  Aufruf (Repo-Wurzel): powershell -File .github\skills\session-handover\scripts\check-skills.ps1
  Meldet: Frontmatter fehlt, name != Ordner, description fehlt oder > 1024 Zeichen, relative Links (./...) ohne Ziel,
  Dateipfade in Backticks, die es nicht gibt. Inhaltliche Aktualitaet (Fakten, Befehle) muss weiterhin von Hand geprueft werden.
#>
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')).Path
$skills = Get-ChildItem (Join-Path $root '.github\skills') -Directory
$warnings = 0

# Relative Pfade (z. B. tests/websocket.rs) in den Projektordnern suchen; target wird bewusst nicht durchsucht.
$searchRoots = 'docs', 'scripts', '.github', 'assets', 'crates', 'fake-moonraker\src', 'fake-moonraker\tests', '.vscode' |
ForEach-Object { Join-Path $root $_ } | Where-Object { Test-Path $_ }
$knownFiles = @($searchRoots | ForEach-Object { Get-ChildItem $_ -Recurse -File } | ForEach-Object { $_.FullName })

function Exists($token, $skillDir) {
    $normalized = $token.Replace('/', '\')
    foreach ($candidate in (Join-Path $root $normalized), (Join-Path $skillDir $normalized)) {
        if (Test-Path $candidate) { return $true }
    }
    return [bool]($knownFiles | Where-Object { $_.EndsWith('\' + $normalized) } | Select-Object -First 1)
}

function Warn($skill, $message) {
    Write-Host "WARN  [$skill] $message" -ForegroundColor Yellow
    $script:warnings++
}

foreach ($skill in $skills) {
    $file = Join-Path $skill.FullName 'SKILL.md'
    if (-not (Test-Path $file)) { Warn $skill.Name 'SKILL.md fehlt'; continue }
    $text = Get-Content $file -Raw -Encoding UTF8

    if ($text -notmatch '(?s)^---\r?\n(.*?)\r?\n---') { Warn $skill.Name 'Frontmatter fehlt'; continue }
    $front = $Matches[1]
    if ($front -notmatch '(?m)^name:\s*(.+?)\s*$') { Warn $skill.Name 'name fehlt' }
    elseif ($Matches[1].Trim("'", '"') -ne $skill.Name) { Warn $skill.Name "name '$($Matches[1])' passt nicht zum Ordner" }
    if ($front -notmatch '(?m)^description:\s*(.+?)\s*$') { Warn $skill.Name 'description fehlt' }
    elseif ($Matches[1].Length -gt 1024) { Warn $skill.Name "description zu lang ($($Matches[1].Length) Zeichen)" }

    foreach ($link in [regex]::Matches($text, '\]\((\./[^)]+)\)') | ForEach-Object { $_.Groups[1].Value }) {
        if (-not (Test-Path (Join-Path $skill.FullName $link))) { Warn $skill.Name "Link ohne Ziel: $link" }
    }

    $pathPattern = '^[\w.\- \\/]+\.(md|ps1|mjs|rs|toml|slint|svg|html|json|drawio|yml|cfg|sh)$'
    foreach ($token in [regex]::Matches($text, '`([^`\r\n]+)`') | ForEach-Object { $_.Groups[1].Value } | Sort-Object -Unique) {
        if ($token -match $pathPattern -and $token -notmatch '^\.\.?[\\/]' -and $token -notmatch '\s-' -and $token -match '[\\/]') {
            if (-not (Exists $token $skill.FullName)) { Warn $skill.Name "Pfad existiert nicht: $token" }
        }
    }
}

Write-Host ("{0} Skills geprueft, {1} Warnungen." -f $skills.Count, $warnings)
exit ([int]($warnings -gt 0))
