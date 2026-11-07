$code = @"
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
public class PwsInspector {
    [StructLayout(LayoutKind.Sequential)]
    public struct PROCESS_MEMORY_COUNTERS_EX {
        public uint cb;
        public uint PageFaultCount;
        public UIntPtr PeakWorkingSetSize;
        public UIntPtr WorkingSetSize;
        public UIntPtr QuotaPeakPagedPoolUsage;
        public UIntPtr QuotaPagedPoolUsage;
        public UIntPtr QuotaPeakNonPagedPoolUsage;
        public UIntPtr QuotaNonPagedPoolUsage;
        public UIntPtr PagefileUsage;
        public UIntPtr PeakPagefileUsage;
        public UIntPtr PrivateUsage;
    }
    [DllImport("psapi.dll", SetLastError = true)]
    public static extern bool GetProcessMemoryInfo(IntPtr hProcess, out PROCESS_MEMORY_COUNTERS_EX counters, uint size);
    [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);

    public static void PressHotkey() {
        keybd_event(0x11, 0, 0, UIntPtr.Zero);
        keybd_event(0x12, 0, 0, UIntPtr.Zero);
        keybd_event(0x4E, 0, 0, UIntPtr.Zero);
        keybd_event(0x4E, 0, 2, UIntPtr.Zero);
        keybd_event(0x12, 0, 2, UIntPtr.Zero);
        keybd_event(0x11, 0, 2, UIntPtr.Zero);
    }

    public static double GetPws(int pid) {
        Process p = Process.GetProcessById(pid);
        PROCESS_MEMORY_COUNTERS_EX counters;
        if (GetProcessMemoryInfo(p.Handle, out counters, (uint)Marshal.SizeOf(typeof(PROCESS_MEMORY_COUNTERS_EX)))) {
            return ((double)counters.PrivateUsage.ToUInt64()) / (1024.0 * 1024.0);
        }
        return 0;
    }
}
"@
Add-Type -TypeDefinition $code

$proc = Start-Process "R:\Projects\notch\target\release\notchbar.exe" -PassThru
Start-Sleep -Milliseconds 600
$idlePws = [PwsInspector]::GetPws($proc.Id)

[PwsInspector]::PressHotkey()
Start-Sleep -Milliseconds 500
$expandedPws = [PwsInspector]::GetPws($proc.Id)

Stop-Process -Id $proc.Id -Force

Write-Output "IDLE_PWS:$idlePws"
Write-Output "EXPANDED_PWS:$expandedPws"
