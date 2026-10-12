# Maintenance of the BASE image (`vibeos-cli winref maint qemu/winref/maint-runner.ps1`):
# (re)registers the runner task and installs the current boot.ps1, instead of reinstalling.
#
# Runs as a payload of the runner itself (already elevated; COM1 is the runner's, so this
# writes to stdout and the runner relays it). Under `--keys` it is typed into Win+R instead,
# for a base whose runner does not work at all, which is how the first base was repaired:
# its task's XML was in System32\Tasks but Task Scheduler did not know it ("The system
# cannot find the file specified"), so nothing ever triggered it.
$ErrorActionPreference = 'Continue'
$port = $null
if (-not $env:WINREF_RUNNER) {  # set by the maintenance CD's run.cmd
    # Typed in via --keys: no runner is relaying stdout, so report on COM1 directly.
    $port = New-Object System.IO.Ports.SerialPort COM1, 115200, None, 8, One
    $port.Open()
}
function Say([string]$m) {
    if ($port) { $port.WriteLine("WINREF-MAINT: $m") } else { Write-Output "WINREF-MAINT: $m" }
}

$known = [bool](Get-ScheduledTask -TaskName 'winref-boot' -ErrorAction SilentlyContinue)
Say "before: task known to Task Scheduler=$known"

try {
    $action = New-ScheduledTaskAction -Execute 'powershell.exe' `
        -Argument '-NoProfile -ExecutionPolicy Bypass -File C:\winref\boot.ps1'
    $trigger = New-ScheduledTaskTrigger -AtLogOn -User 'winref'
    $principal = New-ScheduledTaskPrincipal -UserId 'winref' -LogonType Interactive -RunLevel Highest
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries `
        -ExecutionTimeLimit ([TimeSpan]::Zero)
    Register-ScheduledTask -TaskName 'winref-boot' -Action $action -Trigger $trigger `
        -Principal $principal -Settings $settings -Force -ErrorAction Stop | Out-Null
    $known = [bool](Get-ScheduledTask -TaskName 'winref-boot' -ErrorAction SilentlyContinue)
    Say "registered: task known to Task Scheduler=$known"
} catch { Say "register FAILED: $_" }

# PowerShell reads a script whole before running it, so replacing the running boot.ps1 is
# safe; the new one takes effect at the next boot.
$src = Split-Path -Parent $MyInvocation.MyCommand.Path
Copy-Item (Join-Path $src 'boot.ps1') C:\winref\boot.ps1 -Force
(Get-Item C:\winref\boot.ps1).IsReadOnly = $false
Say "boot.ps1 installed: $((Get-Item C:\winref\boot.ps1).Length) bytes"

Say 'done'
if ($port) {
    # --keys mode: nothing else will power the VM off.
    $port.Close()
    Stop-Computer -Force
}
