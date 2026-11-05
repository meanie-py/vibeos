# Real Windows sockets/processes; isolated temporary files, no bench or firewall mutations.
param([Parameter(Mandatory=$true)][string]$Binary)
$ErrorActionPreference = 'Stop'
$directory = Join-Path ([IO.Path]::GetTempPath()) ('vibeos-bench-fixture-' + [guid]::NewGuid())
$children = @()
New-Item -ItemType Directory -Path $directory | Out-Null
try {
    $exe = Join-Path $directory 'bench-infra.exe'
    Copy-Item -LiteralPath $Binary -Destination $exe
    $image = Join-Path $directory 'image'
    [IO.File]::WriteAllBytes($image, [Text.Encoding]::ASCII.GetBytes('0123456789'))
    foreach ($service in @('http', 'tftp')) {
        if ($service -eq 'http') {
            $probe = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
            $probe.Start(); $port = $probe.LocalEndpoint.Port; $probe.Stop()
        } else {
            $probe = [Net.Sockets.UdpClient]::new(0)
            $port = $probe.Client.LocalEndPoint.Port; $probe.Dispose()
        }
        $child = Start-Process -FilePath $exe -ArgumentList @($service, ('"' + $directory + '"'), '127.0.0.1', $port) -PassThru -WindowStyle Hidden -RedirectStandardOutput (Join-Path $directory "$service.log") -RedirectStandardError (Join-Path $directory "$service.err")
        $child.Refresh()
        $identity = Get-CimInstance Win32_Process -Filter "ProcessId=$($child.Id)"
        if (!$identity -or $identity.ExecutablePath -ne $exe) { throw 'wrong child identity' }
        $children += @{pid=$child.Id; creation=$identity.CreationDate; exe=$exe}
        if ($service -eq 'http') {
            $deadline = [DateTime]::UtcNow.AddSeconds(10)
            do {
                try {
                    $request = [Net.HttpWebRequest]::Create("http://127.0.0.1:$port/image")
                    $request.Timeout = 1000; $request.AddRange(2, 4)
                    $response = $request.GetResponse()
                    break
                } catch {
                    if ([DateTime]::UtcNow -ge $deadline) { throw }
                    Start-Sleep -Milliseconds 50
                }
            } while ($true)
            try {
                $reader = [IO.StreamReader]::new($response.GetResponseStream())
                if ([int]$response.StatusCode -ne 206 -or $reader.ReadToEnd() -ne '234') { throw 'HTTP range mismatch' }
                $reader.Dispose()
            } finally { $response.Dispose() }
        } else {
            $udp = [Net.Sockets.UdpClient]::new(0)
            try {
                $udp.Client.ReceiveTimeout = 500
                $packet = [byte[]](0,1) + [Text.Encoding]::ASCII.GetBytes("image`0octet`0blksize`08`0tsize`00`0")
                $deadline = [DateTime]::UtcNow.AddSeconds(10)
                do {
                    [void]$udp.Send($packet, $packet.Length, '127.0.0.1', $port)
                    $peer = [Net.IPEndPoint]::new([Net.IPAddress]::Any, 0)
                    try { $reply = $udp.Receive([ref]$peer); break } catch { if ([DateTime]::UtcNow -ge $deadline) { throw } }
                } while ($true)
                if ($reply[0] -ne 0 -or $reply[1] -ne 6 -or $peer.Port -eq $port) { throw 'TFTP OACK mismatch' }
                $ack = [byte[]](0,4,0,0); [void]$udp.Send($ack, 4, $peer)
                $reply = $udp.Receive([ref]$peer)
                if ($reply[1] -ne 3 -or $reply[3] -ne 1 -or [Text.Encoding]::ASCII.GetString($reply,4,$reply.Length-4) -ne '01234567') { throw 'TFTP block mismatch' }
                $ack = [byte[]](0,4,0,1); [void]$udp.Send($ack, 4, $peer)
                $reply = $udp.Receive([ref]$peer)
                if ($reply[3] -ne 2 -or [Text.Encoding]::ASCII.GetString($reply,4,$reply.Length-4) -ne '89') { throw 'TFTP final block mismatch' }
                $ack = [byte[]](0,4,0,2); [void]$udp.Send($ack, 4, $peer)
            } finally { $udp.Dispose() }
        }
    }
    Write-Output 'WINDOWS_BENCH_SERVICES_OK'
} finally {
    foreach ($owned in $children) {
        $live = Get-CimInstance Win32_Process -Filter "ProcessId=$($owned.pid)"
        if ($live) {
            if ($live.CreationDate -ne $owned.creation -or $live.ExecutablePath -ne $owned.exe) { throw 'refusing changed child identity' }
            Stop-Process -Id $owned.pid
        }
    }
    Remove-Item -LiteralPath $directory -Recurse -Force
}
