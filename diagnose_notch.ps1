Add-Type -AssemblyName System.Drawing
$code = @"
using System;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;

public class NotchDiagnostics {
    [DllImport("user32.dll")] public static extern IntPtr FindWindow(string c, string w);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hWnd, uint Msg, IntPtr wParam, IntPtr lParam);
    [DllImport("user32.dll")] public static extern IntPtr GetDC(IntPtr h);
    [DllImport("user32.dll")] public static extern int ReleaseDC(IntPtr h, IntPtr d);
    [DllImport("gdi32.dll")] public static extern uint GetPixel(IntPtr d, int x, int y);

    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }

    public static void Inspect() {
        IntPtr h = FindWindow("NotchBarWindowClass", null);
        Console.WriteLine("HWND: " + h.ToInt64());
        if (h != IntPtr.Zero) {
            RECT r;
            GetWindowRect(h, out r);
            bool vis = IsWindowVisible(h);
            Console.WriteLine(string.Format("Rect: {0},{1} - {2},{3} ({4}x{5}), Visible: {6}", r.Left, r.Top, r.Right, r.Bottom, r.Right - r.Left, r.Bottom - r.Top, vis));

            IntPtr d = GetDC(IntPtr.Zero);
            uint c = GetPixel(d, (r.Left + r.Right) / 2, (r.Top + r.Bottom) / 2);
            ReleaseDC(IntPtr.Zero, d);
            Console.WriteLine(string.Format("Screen pixel at center: R={0} G={1} B={2}", c & 0xFF, (c >> 8) & 0xFF, (c >> 16) & 0xFF));
        }
    }

    public static void SendExpand() {
        IntPtr h = FindWindow("NotchBarWindowClass", null);
        if (h != IntPtr.Zero) {
            PostMessage(h, 0x0312, new IntPtr(1001), IntPtr.Zero);
        }
    }
}
"@
Add-Type -TypeDefinition $code -ReferencedAssemblies "System.Drawing.dll"

$proc = Start-Process "R:\Projects\notch\target\release\notchbar.exe" -PassThru
Start-Sleep -Seconds 1

Write-Host "--- Collapsed ---"
[NotchDiagnostics]::Inspect()

Write-Host "--- Sending Expand Hotkey ---"
[NotchDiagnostics]::SendExpand()
Start-Sleep -Seconds 1

Write-Host "--- Expanded ---"
[NotchDiagnostics]::Inspect()

Stop-Process -Id $proc.Id -Force
