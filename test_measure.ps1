$proc = Start-Process "R:\Projects\notch\target\release\notchbar.exe" -PassThru
Start-Sleep -Milliseconds 600

$p = Get-Process -Id $proc.Id
$pws = [math]::round($p.PrivateMemorySize64 / 1MB, 2)
$ws = [math]::round($p.WorkingSet64 / 1MB, 2)
Write-Output "IDLE -> PrivateWorkingSet: $pws MB | WorkingSet: $ws MB"

# Press hotkey to expand
[NotchTestHarness]::PressHotkey()
Start-Sleep -Milliseconds 500

$p2 = Get-Process -Id $proc.Id
$pws2 = [math]::round($p2.PrivateMemorySize64 / 1MB, 2)
$ws2 = [math]::round($p2.WorkingSet64 / 1MB, 2)
Write-Output "EXPANDED -> PrivateWorkingSet: $pws2 MB | WorkingSet: $ws2 MB"

Stop-Process -Id $proc.Id -Force
