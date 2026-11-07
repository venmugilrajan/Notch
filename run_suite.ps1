Add-Type -AssemblyName System.Drawing
$code = @"
using System;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;

public class NotchSuite {
    [DllImport("user32.dll")] public static extern IntPtr GetDC(IntPtr hwnd);
    [DllImport("user32.dll")] public static extern int ReleaseDC(IntPtr hwnd, IntPtr hdc);
    [DllImport("gdi32.dll")] public static extern IntPtr CreateCompatibleDC(IntPtr hdc);
    [DllImport("gdi32.dll")] public static extern IntPtr CreateCompatibleBitmap(IntPtr hdc, int w, int h);
    [DllImport("gdi32.dll")] public static extern IntPtr SelectObject(IntPtr hdc, IntPtr obj);
    [DllImport("gdi32.dll")] public static extern bool BitBlt(IntPtr d, int dx, int dy, int w, int h, IntPtr s, int sx, int sy, int rop);
    [DllImport("gdi32.dll")] public static extern bool DeleteDC(IntPtr hdc);
    [DllImport("gdi32.dll")] public static extern bool DeleteObject(IntPtr obj);
    [DllImport("user32.dll")] public static extern int GetSystemMetrics(int idx);
    [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);
    [DllImport("psapi.dll", SetLastError = true)] public static extern bool GetProcessMemoryInfo(IntPtr hProcess, out PROCESS_MEMORY_COUNTERS_EX counters, uint size);

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

    public static void PressHotkey() {
        keybd_event(0x11, 0, 0, UIntPtr.Zero);
        keybd_event(0x12, 0, 0, UIntPtr.Zero);
        keybd_event(0x4E, 0, 0, UIntPtr.Zero);
        keybd_event(0x4E, 0, 2, UIntPtr.Zero);
        keybd_event(0x12, 0, 2, UIntPtr.Zero);
        keybd_event(0x11, 0, 2, UIntPtr.Zero);
    }

    public static void SaveScreen(string path, int w, int h) {
        int screenW = GetSystemMetrics(0);
        IntPtr hdcScreen = GetDC(IntPtr.Zero);
        IntPtr hdcMem = CreateCompatibleDC(hdcScreen);
        IntPtr hBitmap = CreateCompatibleBitmap(hdcScreen, w, h);
        IntPtr hOld = SelectObject(hdcMem, hBitmap);
        int srcX = (screenW - w) / 2;
        BitBlt(hdcMem, 0, 0, w, h, hdcScreen, srcX, 0, 0x00CC0020);
        using (Bitmap bmp = Image.FromHbitmap(hBitmap)) {
            bmp.Save(path, ImageFormat.Png);
        }
        SelectObject(hdcMem, hOld);
        DeleteObject(hBitmap);
        DeleteDC(hdcMem);
        ReleaseDC(IntPtr.Zero, hdcScreen);
    }

    public static double GetPws(int pid) {
        Process p = Process.GetProcessById(pid);
        PROCESS_MEMORY_COUNTERS_EX c;
        GetProcessMemoryInfo(p.Handle, out c, (uint)Marshal.SizeOf(typeof(PROCESS_MEMORY_COUNTERS_EX)));
        return ((double)c.PrivateUsage.ToUInt64()) / (1024.0 * 1024.0);
    }
}
"@
Add-Type -TypeDefinition $code -ReferencedAssemblies "System.Drawing"

$proc = Start-Process "R:\Projects\notch\target\release\notchbar.exe" -PassThru
Start-Sleep -Milliseconds 600

$idlePws = [NotchSuite]::GetPws($proc.Id)
$p = Get-Process -Id $proc.Id
$idlePs = [math]::round($p.PrivateMemorySize64 / 1MB, 2)
$idleWs = [math]::round($p.WorkingSet64 / 1MB, 2)
[NotchSuite]::SaveScreen("R:\Projects\notch\stage1_collapsed.png", 320, 80)

[NotchSuite]::PressHotkey()
Start-Sleep -Milliseconds 500

$expandedPws = [NotchSuite]::GetPws($proc.Id)
$p2 = Get-Process -Id $proc.Id
$expPs = [math]::round($p2.PrivateMemorySize64 / 1MB, 2)
$expWs = [math]::round($p2.WorkingSet64 / 1MB, 2)
[NotchSuite]::SaveScreen("R:\Projects\notch\stage1_expanded.png", 820, 380)

Stop-Process -Id $proc.Id -Force

Write-Output "RESULT_IDLE_PSAPI:$idlePws"
Write-Output "RESULT_IDLE_PS:$idlePs"
Write-Output "RESULT_IDLE_WS:$idleWs"
Write-Output "RESULT_EXP_PSAPI:$expandedPws"
Write-Output "RESULT_EXP_PS:$expPs"
Write-Output "RESULT_EXP_WS:$expWs"
