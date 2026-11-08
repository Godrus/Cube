# P-WORLD (experimental spike, not for production): edit the world in a real Godot window, close the window the way
# a user would (WM_CLOSE -> the client autosaves), start a NEW process and check that every edit came back.
# Three separate OS processes: write (edit + close), verify (load at start), corrupt (damaged save is refused).
# The input in the write phase is injected by the test script (synthetic, not a person).
#   scripts/test-world-persistence.ps1 [-Godot <path>] [-OutDir <dir>]
param(
    [string]$Godot = $env:GODOT,
    [string]$OutDir = ''
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$client = Join-Path $root 'godot-client'

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

if (-not $OutDir) { $OutDir = Join-Path ([IO.Path]::GetTempPath()) ("pworld-persist-" + [guid]::NewGuid().ToString('N').Substring(0, 8)) }
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$exe = Find-Godot
$save = Join-Path $OutDir 'world.wst0'
Write-Host "godot: $exe"
Write-Host "dir:   $OutDir"
$failed = $false

function Start-Phase([string]$phase) {
    $args = @('--path', "`"$client`"", 'world_edit.tscn', '--', "--phase=$phase", "--dir=`"$OutDir`"", "--save=`"$save`"", "--shot=`"$OutDir`"")
    return Start-Process -FilePath $exe -ArgumentList $args -PassThru
}

function Show-Result([string]$phase) {
    $f = Join-Path $OutDir "$phase.result"
    if (Test-Path $f) { Get-Content $f | ForEach-Object { Write-Host "  [$phase] $_" } } else { Write-Host "  [$phase] no result file" }
}

# --- process 1: edit, then close the window like a user -------------------------------------------------------
Write-Host '== process 1: write (edit the world, then close the window)'
$p1 = Start-Phase 'write'
$ready = Join-Path $OutDir 'ready.txt'
$t0 = Get-Date
while (-not (Test-Path $ready) -and -not $p1.HasExited -and ((Get-Date) - $t0).TotalSeconds -lt 120) { Start-Sleep -Milliseconds 300 }
if (-not (Test-Path $ready)) {
    Write-Host 'FAIL write phase did not finish its edits'
    if (-not $p1.HasExited) { $p1.Kill() }
    Show-Result 'write'; exit 1
}
if (Test-Path $save) { Write-Host 'FAIL a save file existed before the close (the test would prove nothing)'; $failed = $true }
Start-Sleep -Milliseconds 500
[void]$p1.CloseMainWindow()                       # the same WM_CLOSE as the window's close button
if (-not $p1.WaitForExit(20000)) { Write-Host 'FAIL window did not close'; $p1.Kill(); $failed = $true }
Show-Result 'write'
if (-not (Test-Path $save)) { Write-Host 'FAIL closing the window did not write the save file'; exit 1 }
Write-Host ("save file after close: {0} bytes" -f (Get-Item $save).Length)

# --- process 2: a new process loads the save --------------------------------------------------------------------
Write-Host '== process 2: verify (new process, loads the save at start)'
$p2 = Start-Phase 'verify'
if (-not $p2.WaitForExit(90000)) { $p2.Kill(); Write-Host 'FAIL verify timed out'; $failed = $true }
Show-Result 'verify'
if ($p2.ExitCode -ne 0) { Write-Host "FAIL verify exit code $($p2.ExitCode)"; $failed = $true }

# --- process 3: a damaged save is refused -----------------------------------------------------------------------
Write-Host '== process 3: corrupt (one byte of the save flipped)'
$bytes = [IO.File]::ReadAllBytes($save)
$bytes[[int]($bytes.Length / 2)] = $bytes[[int]($bytes.Length / 2)] -bxor 0xFF
[IO.File]::WriteAllBytes($save, $bytes)
$p3 = Start-Phase 'corrupt'
if (-not $p3.WaitForExit(90000)) { $p3.Kill(); Write-Host 'FAIL corrupt timed out'; $failed = $true }
Show-Result 'corrupt'
if ($p3.ExitCode -ne 0) { Write-Host "FAIL corrupt exit code $($p3.ExitCode)"; $failed = $true }

if ($failed) { Write-Host 'RESULT: FAIL'; exit 1 }
Write-Host 'RESULT: PASS (edit -> close window -> new process -> every edit came back; damaged save refused)'
