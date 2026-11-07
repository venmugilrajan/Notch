Add-Type -AssemblyName System.Drawing
$code = @"
using System;
using System.Drawing;
using System.Runtime.InteropServices;

public class PrimaryScreen {
    [DllImport("user32.dll")] public static extern IntPtr GetDC(IntPtr h);
    [DllImport("user32.dll")] public static extern int ReleaseDC(IntPtr h, IntPtr d);
    [DllImport("gdi32.dll")] public static extern uint GetPixel(IntPtr d, int x, int y);

    public static void Sample(int x, int y) {
        IntPtr d = GetDC(IntPtr.Zero);
        uint c = GetPixel(d, x, y);
        ReleaseDC(IntPtr.Zero, d);
        uint r = c & 0xFF;
        uint g = (c >> 8) & 0xFF;
        uint b = (c >> 16) & 0xFF;
        Console.WriteLine(string.Format("Pixel at {0},{1}: R={2} G={3} B={4}", x, y, r, g, b));
    }
}
"@
Add-Type -TypeDefinition $code
[PrimaryScreen]::Sample(10, 10)
[PrimaryScreen]::Sample(500, 500)

