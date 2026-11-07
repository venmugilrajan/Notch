$code = Get-Content "R:\Projects\notch\mem_verify.cs" -Raw
Add-Type -TypeDefinition $code

$p = Get-Process -Name notchbar -ErrorAction SilentlyContinue
if ($p) { Stop-Process -Id $p.Id -Force }

$proc = Start-Process -FilePath "R:\Projects\notch\target\release\notchbar.exe" -PassThru
Write-Host "Started NotchBar PID: $($proc.Id). Waiting 60s for Collapsed Idle..."
Start-Sleep -Seconds 60
[MemoryVerifier]::Sample($proc.Id, "a_collapsed_idle_60s")

Write-Host "Expanding Home panel... Waiting 60s for Expanded Home Idle..."
[MemoryVerifier]::ToggleNotch()
Start-Sleep -Seconds 60
[MemoryVerifier]::Sample($proc.Id, "b_expanded_home_60s")

Write-Host "Switching to Clipboard panel... Waiting 5s..."
[MemoryVerifier]::ClickClipboardTab()
Start-Sleep -Seconds 5
[MemoryVerifier]::Sample($proc.Id, "c_expanded_clipboard")

Write-Host "Collapsing NotchBar... Waiting 5s..."
[MemoryVerifier]::ToggleNotch()
Start-Sleep -Seconds 5
[MemoryVerifier]::Sample($proc.Id, "d_collapsed_again")

Stop-Process -Id $proc.Id -Force
Write-Host "RAM Measurement finished cleanly."

