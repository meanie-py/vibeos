# Maintenance of the BASE image (`vibeos-cli winref maint qemu/winref/maint-runner.ps1`):
# records why the runner task never fired, then fixes it in place instead of reinstalling.
#   1. evidence: the task's last result, and the power source Windows believes it is on;
#   2. the task stops refusing to start "on batteries" (a QEMU VM has no AC adapter);
#   3. the newer boot.ps1 from this CD (it also logs to C:\winref\boot.log);
#   4. proof: start the task by hand and wait for its first line in boot.log.
$ErrorActionPreference = 'Continue'
$port = New-Object System.IO.Ports.SerialPort COM1, 115200, None, 8, One
$port.Open()
function Say([string]$m) { $port.WriteLine("WINREF-MAINT: $m") }

$info = Get-ScheduledTaskInfo -TaskName 'winref-boot'
Say "before: lastRun=$($info.LastRunTime) lastResult=$($info.LastTaskResult) missed=$($info.NumberOfMissedRuns)"
Add-Type -AssemblyName System.Windows.Forms
$ps = [System.Windows.Forms.SystemInformation]::PowerStatus
Say "power: line=$($ps.PowerLineStatus) battery=$($ps.BatteryChargeStatus)"

$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries `
    -ExecutionTimeLimit ([TimeSpan]::Zero)
Set-ScheduledTask -TaskName 'winref-boot' -Settings $settings | Out-Null
Say "task settings: exit=$?"

$src = Split-Path -Parent $MyInvocation.MyCommand.Path
Copy-Item (Join-Path $src 'boot.ps1') C:\winref\boot.ps1 -Force
Say "boot.ps1 updated: $((Get-Item C:\winref\boot.ps1).Length) bytes"

# boot.ps1 opens COM1 itself, so release it first; it reports straight to the console.
$port.Close()
Remove-Item C:\winref\boot.log -ErrorAction SilentlyContinue
Start-ScheduledTask -TaskName 'winref-boot'
$t0 = Get-Date
while (-not (Test-Path C:\winref\boot.log) -and ((Get-Date) - $t0).TotalSeconds -lt 300) {
    Start-Sleep -Seconds 5
}
# boot.ps1 finds no run.cmd on this CD, says so, and powers the VM off: that IS the proof.
if (-not (Test-Path C:\winref\boot.log)) {
    $port.Open()
    Say 'FAILED: the runner task did not start within 300 s of Start-ScheduledTask'
    $port.Close()
    Stop-Computer -Force
}
