# Per-boot runner of the Windows reference VM (`vibeos-cli winref run`). Finds `run.cmd`
# at the root of a payload CD, runs it elevated from a writable copy in C:\winref\payload as the working
# directory, streams every output line to COM1, reports the exit code, and powers off.
# The launcher reads COM1; the protocol is the WINREF: lines below and nothing else.
$ErrorActionPreference = 'Continue'
# Also to C:\winref\boot.log, so `vibeos-cli winref logs` can say what a silent run did.
$log = 'C:\winref\boot.log'
$port = $null
try {
    $port = New-Object System.IO.Ports.SerialPort COM1, 115200, None, 8, One
    $port.Open()
} catch { $port = $null }
function Say([string]$m) {
    Add-Content -Path $log -Value "$(Get-Date -Format o) $m"
    if ($port) { try { $port.WriteLine($m) } catch {} }
}

$v = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
Say "WINREF: ready build=$($v.CurrentBuild).$($v.UBR) edition=$($v.EditionID)"

$run = Get-PSDrive -PSProvider FileSystem |
    ForEach-Object { Join-Path $_.Root 'run.cmd' } |
    Where-Object { Test-Path $_ } |
    Select-Object -First 1
if (-not $run) {
    Say 'WINREF: no payload'
} else {
    # Run from a WRITABLE copy: tests write beside their own .exe (version.c's manifest
    # tests got ERROR_ACCESS_DENIED on the CD and then measured an unmanifested process).
    $payload = 'C:\winref\payload'
    Remove-Item -Recurse -Force $payload -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force $payload | Out-Null
    Copy-Item -Path (Join-Path (Split-Path $run) '*') -Destination $payload -Recurse -Force
    Get-ChildItem $payload -Recurse -File | ForEach-Object { $_.IsReadOnly = $false }
    $run = Join-Path $payload 'run.cmd'
    Say "WINREF: running $run"
    Push-Location $payload
    & cmd.exe /c "`"$run`" 2>&1" | ForEach-Object { Say "OUT: $_" }
    $code = $LASTEXITCODE
    Pop-Location
    Say "WINREF: exit=$code"
}
Say 'WINREF: poweroff'
if ($port) { $port.Close() }
Stop-Computer -Force
