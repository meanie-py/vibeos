# Per-boot runner of the Windows reference VM (`vibeos-cli winref run`). Finds `run.cmd`
# at the root of a payload CD, runs it elevated with C:\winref\out as the working
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
    Say "WINREF: running $run"
    $out = 'C:\winref\out'
    New-Item -ItemType Directory -Force $out | Out-Null
    Push-Location $out
    & cmd.exe /c "`"$run`" 2>&1" | ForEach-Object { Say "OUT: $_" }
    $code = $LASTEXITCODE
    Pop-Location
    Say "WINREF: exit=$code"
}
Say 'WINREF: poweroff'
if ($port) { $port.Close() }
Stop-Computer -Force
