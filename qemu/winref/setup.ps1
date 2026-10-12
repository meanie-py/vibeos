# First-logon configuration of the Windows reference VM. Runs once, from the answer-file
# CD, as the autologon administrator; ends by shutting down, which is how the launcher
# knows the base image is finished. Every step is reported on COM1 AND in
# C:\winref\setup.log, so a failed install says which step broke instead of leaving a
# silent VM (the first attempt hung with nothing on COM1 and nothing on disk to read).
$ErrorActionPreference = 'Continue'
New-Item -ItemType Directory -Force C:\winref | Out-Null
$log = 'C:\winref\setup.log'
$port = $null
try {
    $port = New-Object System.IO.Ports.SerialPort COM1, 115200, None, 8, One
    $port.Open()
} catch { $port = $null }
function Say([string]$m) {
    Add-Content -Path $log -Value "WINREF-SETUP: $m"
    if ($port) { try { $port.WriteLine("WINREF-SETUP: $m") } catch {} }
}
Say "ports=$([System.IO.Ports.SerialPort]::GetPortNames() -join ',') com1=$([bool]$port)"

$src = Split-Path -Parent $MyInvocation.MyCommand.Path
Copy-Item (Join-Path $src '*') C:\winref -Recurse -Force
Say 'copied runner to C:\winref'

# Test drivers (kmtests, new kernel-API tests) must load. On x64 this admits TEST-SIGNED
# drivers, not unsigned ones: a payload signs its own (kmtest-sweep.ps1). Requires Secure
# Boot off, which the launcher's non-secure firmware guarantees.
bcdedit /set testsigning on | Out-Null
Say "testsigning: exit=$LASTEXITCODE"

# A run is boot -> test -> shutdown; nothing may sleep or hibernate in between.
powercfg /change standby-timeout-ac 0
powercfg /change monitor-timeout-ac 0
powercfg /hibernate off
Say 'power: no sleep, no hibernate'

# No network device exists, but keep the update services from spinning on boot anyway.
foreach ($svc in 'wuauserv', 'UsoSvc') {
    sc.exe config $svc start= disabled | Out-Null
    Say "service ${svc}: disabled (exit=$LASTEXITCODE)"
}

# Test binaries are unsigned and freshly built; real-time scanning can quarantine one and
# turn a conformance result into a Defender result. Exclude the payload CD letters and the
# runner's working directory. (Tamper protection may refuse; that is reported, not fatal.)
try {
    Add-MpPreference -ExclusionPath 'C:\winref', 'D:\', 'E:\', 'F:\', 'G:\' -ErrorAction Stop
    Say 'defender: exclusions added'
} catch { Say "defender: exclusions FAILED: $_" }

# The runner: an ELEVATED task at the autologon user's logon (a Run key would get the
# UAC-filtered token, and some tests need admin). UAC itself stays at its default so
# Windows behaves as users have it. NOT `schtasks /create /ru winref`: without /rp that
# prompts for the password on stdin and waits forever, which is how the first install hung.
# An Interactive-logon principal needs no stored password.
try {
    $action = New-ScheduledTaskAction -Execute 'powershell.exe' `
        -Argument '-NoProfile -ExecutionPolicy Bypass -File C:\winref\boot.ps1'
    $trigger = New-ScheduledTaskTrigger -AtLogOn -User 'winref'
    $principal = New-ScheduledTaskPrincipal -UserId 'winref' -LogonType Interactive -RunLevel Highest
    # No 72 h limit (it would kill a long soak), and never skip a start over battery state.
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries `
        -ExecutionTimeLimit ([TimeSpan]::Zero)
    Register-ScheduledTask -TaskName 'winref-boot' -Action $action -Trigger $trigger `
        -Principal $principal -Settings $settings -Force -ErrorAction Stop | Out-Null
    # Verify Task Scheduler actually KNOWS it: on the first base the XML file existed in
    # System32\Tasks and Get-ScheduledTask said "cannot find the file specified".
    $known = [bool](Get-ScheduledTask -TaskName 'winref-boot' -ErrorAction SilentlyContinue)
    Say "runner task: registered (known=$known)"
} catch { Say "runner task: FAILED: $_" }

$v = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
Say "done build=$($v.CurrentBuild).$($v.UBR) edition=$($v.EditionID) display=$($v.DisplayVersion)"
if ($port) { $port.Close() }
Stop-Computer -Force
