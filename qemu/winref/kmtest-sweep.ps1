# The kmtest sweep on real Windows (`vibeos-cli winref kmtest`): the reference column for
# the conformance ratchet. Runs from the payload's run.cmd, elevated, with every kmtest
# binary beside it, and prints the SAME markers the guest sweep (tools/payloads/kmtest/
# sweep.c) sends to serial, so tools/conformance/parse_kmtest_serial.py reads both:
#
#   [KMTEST-SWEEP] launch <Test>          [KMTEST-SWEEP] exit <Test> code=N
#   [KMTEST-SWEEP] TIMEOUT <Test>         [KMTEST-SWEEP] done total=...
#   *** BUGCHECK: 0x...                   (kmtest_.exe adds [KMTEST:BEGIN/END ...])
#
# A test that bugchecks Windows reboots the VM, and the runner starts this script again.
# C:\winref\kmtest-progress.txt survives that (the payload copy does not), so the rerun
# reports the crash against the test that was in flight, skips everything already done,
# and carries on: one crashing test cannot cost the rest of the column.
#
# The VM can also be LOST without Windows crashing: QEMU's WHPX on this host can die under
# a kernel-heavy test (`Unexpected VP exit code 4`), and the launcher then boots the same
# overlay again. Windows logs no bugcheck for that, so the in-flight test is run again,
# once; lost twice, it is reported as VM-LOST, which grades as NOT RUN, never as a result.
# -Bin is the payload's copy of the guest's reactos\bin: kmtest_.exe, every driver, the list.
param([Parameter(Mandatory)][string]$Bin, [int]$TimeoutSec = 180)
$ErrorActionPreference = 'Continue'
$progress = 'C:\winref\kmtest-progress.txt'
# Every progress line goes to the DISK before the next test starts. Add-Content only
# reaches the cache, and a bugcheck discards the cache: the first crash of the column lost
# `done ObSymbolicLink` and `inflight ObType`, and blamed ObSymbolicLink for ObType's 0x3B.
function Mark([string]$line) {
    $fs = New-Object System.IO.FileStream($progress, 'Append', 'Write', 'Read', 4096, 'WriteThrough')
    $b = [System.Text.Encoding]::UTF8.GetBytes("$line`r`n")
    $fs.Write($b, 0, $b.Length)
    $fs.Flush($true)
    $fs.Close()
}

$done = @{}
$inflight = $null
$tries = @{}
if (Test-Path $progress) {
    foreach ($ln in Get-Content $progress) {
        $kind, $name, $when = $ln -split ' ', 3
        if ($kind -eq 'done') { $done[$name] = $true; if ($name -eq $inflight) { $inflight = $null } }
        elseif ($kind -eq 'inflight') { $inflight = $name; $since = [datetime]$when; $tries[$name] = 1 + [int]$tries[$name] }
    }
}
if ($inflight) {
    # Windows records a bugcheck as event 1001 ("The bugcheck was: 0x0000003b (...)") once
    # it is back up. Only one logged AFTER this test started is this test's.
    $ev = Get-WinEvent -FilterHashtable @{ LogName = 'System'; Id = 1001; StartTime = $since } -MaxEvents 5 -ErrorAction SilentlyContinue |
        Where-Object { $_.Message -match 'bugcheck was: (0x[0-9a-fA-F]+)' } | Select-Object -First 1
    if ($ev -and $ev.Message -match 'bugcheck was: (0x[0-9a-fA-F]+)') {
        $code = '0x{0:X}' -f [Convert]::ToUInt64($Matches[1], 16)
        Write-Output "[KMTEST-SWEEP] launch $inflight"
        Write-Output "*** BUGCHECK: $code (Windows rebooted under $inflight)"
        Mark "done $inflight"
        $done[$inflight] = $true
    } elseif ($tries[$inflight] -ge 2) {
        Write-Output "[KMTEST-SWEEP] VM-LOST $inflight (the VM died under it twice, no bugcheck logged)"
        Mark "done $inflight"
        $done[$inflight] = $true
    } else {
        Write-Output "[KMTEST-SWEEP] RERUN $inflight (the VM died under it, no bugcheck logged)"
    }
}

# Test signing. testsigning is on in the base image, but on x64 that admits TEST-SIGNED
# drivers, not unsigned ones: sign every .sys with a throwaway self-signed certificate
# made here (the overlay is discarded after the run), trusted as a root and a publisher.
$cert = Get-ChildItem Cert:\LocalMachine\My | Where-Object Subject -eq 'CN=vibeos winref kmtest' | Select-Object -First 1
if (-not $cert) {
    $cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject 'CN=vibeos winref kmtest' `
        -CertStoreLocation Cert:\LocalMachine\My -HashAlgorithm SHA256 -NotAfter (Get-Date).AddYears(5)
    foreach ($store in 'Root', 'TrustedPublisher') {
        $s = New-Object System.Security.Cryptography.X509Certificates.X509Store $store, 'LocalMachine'
        $s.Open('ReadWrite'); $s.Add($cert); $s.Close()
    }
}
$bad = 0
$drivers = Get-ChildItem $Bin -Recurse -Filter *.sys
foreach ($sys in $drivers) {
    $sig = Set-AuthenticodeSignature -FilePath $sys.FullName -Certificate $cert -HashAlgorithm SHA256
    if ($sig.Status -ne 'Valid') { $bad++; Write-Output "[KMTEST-SWEEP] SIGN FAILED $($sys.Name): $($sig.Status) $($sig.StatusMessage)" }
}
Write-Output "[KMTEST-SWEEP] signed $($drivers.Count - $bad)/$($drivers.Count) drivers"
$bcd = (bcdedit /enum '{current}' | Select-String testsigning) -replace '\s+', ' '
Write-Output "[KMTEST-SWEEP] $bcd"

$tests = Get-Content (Join-Path $Bin 'kmtest-tests.txt') | Where-Object { $_ -and -not $_.StartsWith('#') }
$exe = Join-Path $Bin 'kmtest_.exe'
$out = Join-Path $env:TEMP 'kmtest-out.txt'
$total = 0; $launched = 0; $timeouts = 0
foreach ($t in $tests) {
    $total++
    if ($done[$t]) { continue }
    Mark "inflight $t $((Get-Date).ToString('o'))"
    Write-Output "[KMTEST-SWEEP] launch $t"
    $launched++
    $p = Start-Process -FilePath $exe -ArgumentList $t -WorkingDirectory $Bin -NoNewWindow -PassThru `
        -RedirectStandardOutput $out -RedirectStandardError "$out.err"
    # Without the handle cached now, a Start-Process object reports ExitCode as $null.
    $null = $p.Handle
    $timedOut = -not $p.WaitForExit($TimeoutSec * 1000)
    if ($timedOut) {
        $timeouts++
        Write-Output "[KMTEST-SWEEP] TIMEOUT $t (${TimeoutSec}s) - terminating"
        Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
        $p.WaitForExit(10000) | Out-Null
    }
    Get-Content $out -ErrorAction SilentlyContinue
    Get-Content "$out.err" -ErrorAction SilentlyContinue
    if (-not $timedOut) {
        # As the guest sweep prints it: unsigned, so an NTSTATUS exit reads 3221225477.
        $code = [BitConverter]::ToUInt32([BitConverter]::GetBytes([int]$p.ExitCode), 0)
        Write-Output "[KMTEST-SWEEP] exit $t code=$code"
    }
    Mark "done $t"
}
Write-Output "[KMTEST-SWEEP] done total=$total launched=$launched failstart=0 timeouts=$timeouts"
