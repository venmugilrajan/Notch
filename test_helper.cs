using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
using System.Diagnostics;
using System.Threading;

public class NotchAutomator {
    [DllImport("user32.dll")]
    public static extern IntPtr FindWindow(string lpClassName, string lpWindowName);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    [DllImport("user32.dll")]
    public static extern bool PostMessage(IntPtr hWnd, uint Msg, IntPtr wParam, IntPtr lParam);

    [DllImport("psapi.dll", SetLastError = true)]
    public static extern bool GetProcessMemoryInfo(IntPtr hProcess, out PROCESS_MEMORY_COUNTERS_EX counters, uint size);

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

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

    public static void SnapWindow(string className, string outputPath) {
        IntPtr hWnd = FindWindow(className, null);
        if (hWnd == IntPtr.Zero) {
            Console.WriteLine("Window not found: " + className);
            return;
        }
        RECT rect;
        GetWindowRect(hWnd, out rect);
        int w = rect.Right - rect.Left;
        int h = rect.Bottom - rect.Top;
        if (w <= 0 || h <= 0) return;

        using (Bitmap bmp = new Bitmap(w, h)) {
            using (Graphics g = Graphics.FromImage(bmp)) {
                g.CopyFromScreen(rect.Left, rect.Top, 0, 0, new Size(w, h), CopyPixelOperation.SourceCopy);
            }
            bmp.Save(outputPath, ImageFormat.Png);
            Console.WriteLine("Snap saved: " + outputPath + " (" + w + "x" + h + ")");
        }
    }

    public static void CropAndZoom(string srcPath, string destPath, int x, int y, int w, int h, int zoom) {
        using (Bitmap src = new Bitmap(srcPath)) {
            using (Bitmap crop = new Bitmap(w, h)) {
                using (Graphics g = Graphics.FromImage(crop)) {
                    g.DrawImage(src, new Rectangle(0, 0, w, h), new Rectangle(x, y, w, h), GraphicsUnit.Pixel);
                }
                using (Bitmap zoomed = new Bitmap(w * zoom, h * zoom)) {
                    using (Graphics gz = Graphics.FromImage(zoomed)) {
                        gz.InterpolationMode = InterpolationMode.NearestNeighbor;
                        gz.PixelOffsetMode = PixelOffsetMode.Half;
                        gz.DrawImage(crop, new Rectangle(0, 0, w * zoom, h * zoom));
                    }
                    zoomed.Save(destPath, ImageFormat.Png);
                    Console.WriteLine("Crop saved: " + destPath);
                }
            }
        }
    }

    public static void ToggleNotch(string className) {
        IntPtr hWnd = FindWindow(className, null);
        if (hWnd != IntPtr.Zero) {
            PostMessage(hWnd, 0x0312, new IntPtr(1001), IntPtr.Zero);
        }
    }

    public static void ToggleGallery(string className) {
        IntPtr hWnd = FindWindow(className, null);
        if (hWnd != IntPtr.Zero) {
            PostMessage(hWnd, 0x0312, new IntPtr(1002), IntPtr.Zero);
        }
    }

    public static void SendDockClick(string className, int dockIndex) {
        IntPtr hWnd = FindWindow(className, null);
        if (hWnd != IntPtr.Zero) {
            int slotW = 760 / 14;
            int x = (dockIndex * slotW) + (slotW / 2);
            int y = 340 - 28;
            IntPtr lParam = new IntPtr((y << 16) | (x & 0xFFFF));
            PostMessage(hWnd, 0x0201, new IntPtr(1), lParam); // WM_LBUTTONDOWN
            PostMessage(hWnd, 0x0202, IntPtr.Zero, lParam);    // WM_LBUTTONUP
        }
    }

    public static void PrintMem(int pid, string label) {
        Process p = Process.GetProcessById(pid);
        PROCESS_MEMORY_COUNTERS_EX mem;
        mem.cb = (uint)Marshal.SizeOf(typeof(PROCESS_MEMORY_COUNTERS_EX));
        if (GetProcessMemoryInfo(p.Handle, out mem, mem.cb)) {
            double privMb = (ulong)mem.PrivateUsage / (1024.0 * 1024.0);
            double wsMb = (ulong)mem.WorkingSetSize / (1024.0 * 1024.0);
            Console.WriteLine(string.Format("{0} -> Private Working Set: {1:F2} MB | WorkingSet: {2:F2} MB", label, privMb, wsMb));
        }
    }
}

