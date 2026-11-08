# P-WORLD (experimental spike, not for production): build the adapter and open the world window.
#   scripts/run-world.ps1                       seed 12345, radius 1
#   scripts/run-world.ps1 -Seed 777 -Radius 4   chosen seed and loaded radius (0..8)
# Godot is taken from -Godot, then $env:GODOT, then PATH, then the winget install location.
# Controls: WASD / arrows walk (relative to the camera), Space jumps, right mouse button drag orbits the camera,
# mouse wheel zooms, "Jump 64" teleports 64 voxels along +x (the character waits for the chunk, then falls).
param(
    [long]$Seed = 12345,
    [ValidateRange(0, 8)][int]$Radius = 1,
    [ValidateSet('faces', 'greedy', 'cubes')][string]$Renderer = 'faces',
    [string]$Godot = $env:GODOT,
    [switch]$NoBuild
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot

function Find-Godot {
    if ($Godot -and (Test-Path $Godot)) { return $Godot }
    $c = Get-Command godot -ErrorAction SilentlyContinue
    if ($c) { return $c.Source }
    $w = Join-Path $env:LOCALAPPDATA 'Microsoft\WinGet\Packages'
    if (Test-Path $w) {
        $f = Get-ChildItem -Path $w -Recurse -Filter 'Godot_v*_win64.exe' -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($f) { return $f.FullName }
    }
    throw 'Godot executable not found: pass -Godot <path> or set $env:GODOT'
}

if (-not $NoBuild) {
    Push-Location $root
    try { cargo build -p cube-godot-adapter; if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' } } finally { Pop-Location }
}
$exe = Find-Godot
Write-Host "godot: $exe"
& $exe --path (Join-Path $root 'godot-client') world.tscn -- "--seed=$Seed" "--radius=$Radius" "--renderer=$Renderer"
