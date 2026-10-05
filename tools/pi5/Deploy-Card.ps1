# Write a tools/pi5/make-card.sh tree onto the Pi 5 bench card's FAT partition.
#
#   powershell -File Deploy-Card.ps1 -Src <tree, Windows path> -Drive F -PartUuid 8b744144
#
# Refuses anything that is not plainly that card: a USB disk that is neither boot
# nor system, at most 256 GB, whose MBR id is the PARTUUID the tree's cmdline.txt
# boots Linux from, with a FAT32 volume labelled bootfs (a fresh Pi OS card) or
# VIBEOS_RPI5 (this bench). Only the FAT volume is touched; Linux's root survives.
# Every file is read back and checked against the tree's card.sha256.
param(
    [Parameter(Mandatory)][string]$Src,
    [Parameter(Mandatory)][string]$Drive,
    [Parameter(Mandatory)][string]$PartUuid
)
$ErrorActionPreference = 'Stop'

$vol = Get-Volume -DriveLetter $Drive
$part = Get-Partition -DriveLetter $Drive
$disk = Get-Disk -Number $part.DiskNumber
$id = '{0:x8}' -f $disk.Signature
$problems = @()
if ($disk.BusType -ne 'USB') { $problems += "bus $($disk.BusType)" }
if ($disk.IsBoot -or $disk.IsSystem) { $problems += 'boot/system disk' }
if ($disk.Size -gt 256GB) { $problems += "size $($disk.Size)" }
if ($disk.PartitionStyle -ne 'MBR' -or $id -ne $PartUuid) { $problems += "MBR id $id, expected $PartUuid" }
if ($vol.FileSystem -ne 'FAT32') { $problems += "filesystem $($vol.FileSystem)" }
if ($vol.FileSystemLabel -notin @('bootfs', 'VIBEOS_RPI5')) { $problems += "label $($vol.FileSystemLabel)" }
if ($problems) { throw "not the Pi 5 bench card (disk $($disk.Number)): $($problems -join '; ')" }

$root = "${Drive}:\"
Get-ChildItem $root -Force | Where-Object Name -ne 'System Volume Information' | Remove-Item -Recurse -Force
Copy-Item -Path (Join-Path $Src '*') -Destination $root -Recurse -Force -Exclude 'card.sha256'
Set-Volume -DriveLetter $Drive -NewFileSystemLabel 'VIBEOS_RPI5'
Write-VolumeCache -DriveLetter $Drive

$bad = 0
foreach ($line in Get-Content (Join-Path $Src 'card.sha256')) {
    $hash, $rel = $line -split '\s+', 2
    $path = Join-Path $root ($rel.TrimStart('.', '/') -replace '/', '\')
    if (-not (Test-Path $path) -or (Get-FileHash $path).Hash.ToLower() -ne $hash) {
        Write-Output "MISMATCH $rel"; $bad++
    }
}
if ($bad) { throw "$bad file(s) did not read back" }
Write-Output "card written and verified: $((Get-Content (Join-Path $Src 'card.sha256')).Count) files"
