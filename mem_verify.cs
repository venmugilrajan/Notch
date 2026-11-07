using System;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;

public class MemoryVerifier {
    [DllImport("user32.dll")] public static extern IntPtr FindWindow(string c, string w);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hWnd, uint Msg, IntPtr wParam, IntPtr lParam);
    [DllImport("psapi.dll", SetLastError = true)] public static extern bool GetProcessMemoryInfo(IntPtr hProcess, out PROCESS_MEMORY_COUNTERS_EX counters, uint size);

    [StructLayout(LayoutKind.Sequential, Size = 72)]
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

    public static void Sample(int pid, string phase) {
        Process p = Process.GetProcessById(pid);
        PROCESS_MEMORY_COUNTERS_EX mem;
        mem.cb = (uint)Marshal.SizeOf(typeof(PROCESS_MEMORY_COUNTERS_EX));
        GetProcessMemoryInfo(p.Handle, out mem, mem.cb);
        
        double privMb = (ulong)mem.PrivateUsage / (1024.0 * 1024.0);
        double wsMb = (ulong)mem.WorkingSetSize / (1024.0 * 1024.0);
        double psPrivMb = (double)p.PrivateMemorySize64 / (1024.0 * 1024.0);

        Console.WriteLine(string.Format("PHASE:{0}|PSAPI_PRIVATE:{1:F2}|PS_PRIVATEMEM:{2:F2}|WORKINGSET:{3:F2}", phase, privMb, psPrivMb, wsMb));
    }

    public static void ToggleNotch() {
        IntPtr h = FindWindow("NotchBarWindowClass", null);
        if (h != IntPtr.Zero) PostMessage(h, 0x0312, new IntPtr(1001), IntPtr.Zero);
    }

    public static void ClickClipboardTab() {
        IntPtr h = FindWindow("NotchBarWindowClass", null);
        if (h != IntPtr.Zero) {
            // Tab 6 is Clipboard (slot index 6). slotW = 760 / 14 = 54. center_x = 6 * 54 + 27 = 351, y = 340 - 28 = 312
            int x = (6 * (760 / 14)) + ((760 / 14) / 2);
            int y = 312;
            IntPtr lParam = new IntPtr((y << 16) | (x & 0xFFFF));
            PostMessage(h, 0x0201, new IntPtr(1), lParam);
            PostMessage(h, 0x0202, IntPtr.Zero, lParam);
        }
    }
}

