Add-Type -AssemblyName System.Drawing
$code = @"
using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;

public class NotchTestHarness {
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

    public static void PressHotkey() {
        keybd_event(0x11, 0, 0, UIntPtr.Zero);
        keybd_event(0x12, 0, 0, UIntPtr.Zero);
        keybd_event(0x4E, 0, 0, UIntPtr.Zero);
        keybd_event(0x4E, 0, 2, UIntPtr.Zero);
        keybd_event(0x12, 0, 2, UIntPtr.Zero);
        keybd_event(0x11, 0, 2, UIntPtr.Zero);
    }
}
"@
Add-Type -TypeDefinition $code -ReferencedAssemblies "System.Drawing"

Start-Process "R:\Projects\notch\target\release\notchbar.exe"
Start-Sleep -Milliseconds 600
[NotchTestHarness]::SaveScreen("R:\Projects\notch\stage1_collapsed.png", 320, 80)

[NotchTestHarness]::PressHotkey()
Start-Sleep -Milliseconds 500
[NotchTestHarness]::SaveScreen("R:\Projects\notch\stage1_expanded.png", 820, 380)

Stop-Process -Name notchbar -Force
