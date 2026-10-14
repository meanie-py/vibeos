param([string]$PortName = 'COM5', [switch]$EnterSetup)
$ErrorActionPreference = 'Stop'
$runDir = Join-Path $PSScriptRoot ('serial-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $runDir | Out-Null
$serial = [System.IO.Ports.SerialPort]::new($PortName,115200,[System.IO.Ports.Parity]::None,8,[System.IO.Ports.StopBits]::One)
$serial.Handshake = [System.IO.Ports.Handshake]::None
$serial.DtrEnable = $false
$serial.RtsEnable = $false
$serial.ReadTimeout = 250
$stream = $null
$times = $null
try {
    $serial.Open()
    $stream = [System.IO.File]::Open((Join-Path $runDir 'serial.bin'),[System.IO.FileMode]::CreateNew,[System.IO.FileAccess]::Write,[System.IO.FileShare]::Read)
    # serial.times: one '<byte offset> <unix ms>' line per chunk read, so a silence
    # in the log can be measured afterwards (serial_times.py). serial.bin stays raw.
    $times = [System.IO.StreamWriter]::new([System.IO.File]::Open((Join-Path $runDir 'serial.times'),[System.IO.FileMode]::CreateNew,[System.IO.FileAccess]::Write,[System.IO.FileShare]::Read))
    $offset = [int64]0
    [ordered]@{ status='LISTENING'; port=$PortName; baud=115200; format='8N1'; handshake='None'; pid=$PID; started=(Get-Date).ToString('o'); directory=$runDir } |
        ConvertTo-Json | Set-Content (Join-Path $PSScriptRoot 'serial-current.json')
    Write-Output "LISTENING $PortName 115200 8N1 -> $runDir"
    if ($EnterSetup) { $serial.Write([byte[]]@(27), 0, 1) }
    $buffer = New-Object byte[] 4096
    while ($true) {
        $commandPath = Join-Path $PSScriptRoot 'serial-send.bin'
        if (Test-Path -LiteralPath $commandPath) {
            # The writer may still hold the file. Losing the whole capture to a
            # sharing violation cost a live session on 2026-09-20 - skip this
            # poll and pick the file up on the next one instead of dying.
            try {
                $commandBytes = [System.IO.File]::ReadAllBytes($commandPath)
                $serial.Write($commandBytes,0,$commandBytes.Length)
                Remove-Item -LiteralPath $commandPath -ErrorAction Stop
            } catch {
                $commandBytes = $null
            }
        }
        try { $count = $serial.Read($buffer,0,$buffer.Length) }
        catch [System.TimeoutException] { continue }
        if ($count -gt 0) {
            $times.WriteLine(('{0} {1}' -f $offset, [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()))
            $times.Flush()
            $stream.Write($buffer,0,$count)
            $stream.Flush()
            $offset += $count
        }
    }
} catch {
    [ordered]@{ status='FAILED'; error=$_.ToString(); directory=$runDir } |
        ConvertTo-Json | Set-Content (Join-Path $PSScriptRoot 'serial-current.json')
    throw
} finally {
    if ($stream) { $stream.Dispose() }
    if ($times) { $times.Dispose() }
    $serial.Dispose()
}
