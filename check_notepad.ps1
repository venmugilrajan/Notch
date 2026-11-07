$code = @"
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
public class PureCProgram {
    [DllImport("psapi.dll", SetLastError = true)]
    public static extern bool GetProcessMemoryInfo(IntPtr hProcess, out PROCESS_MEMORY_COUNTERS_EX counters, uint size);

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

    public static void CheckProc(int pid) {
        Process p = Process.GetProcessById(pid);
        PROCESS_MEMORY_COUNTERS_EX c;
        GetProcessMemoryInfo(p.Handle, out c, (uint)Marshal.SizeOf(typeof(PROCESS_MEMORY_COUNTERS_EX)));
        double privMb = c.PrivateUsage.ToUInt64() / (1024.0 * 1024.0);
        double wsMb = c.WorkingSetSize.ToUInt64() / (1024.0 * 1024.0);
        Console.WriteLine(string.Format("PID: {0} ({1}) -> PrivateUsage: {2:F2} MB | WorkingSet: {3:F2} MB", pid, p.ProcessName, privMb, wsMb));
    }
}
"@
Add-Type -TypeDefinition $code

$p = Start-Process "notepad.exe" -PassThru
Start-Sleep -Milliseconds 600
[PureCProgram]::CheckProc($p.Id)
Stop-Process -Id $p.Id -Force
