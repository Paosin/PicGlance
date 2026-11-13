param([string]$Action, [string]$Target)
$ErrorActionPreference = 'Stop'
if ($Action -eq 'explorer') {
    $shell = New-Object -ComObject Shell.Application
    $file = Get-Item -LiteralPath $Target
    $folder = $shell.Namespace($file.DirectoryName)
    $item = $folder.ParseName($file.Name)
    $verb = $item.Verbs() | Where-Object { $_.Name.Replace('&', '') -eq 'Open with PicGlance' } | Select-Object -First 1
    if (-not $verb) { throw 'Explorer did not expose the PicGlance verb.' }
    $verb.DoIt()
    return
}

if ($Action -eq 'handler') {
    Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class TestHandler {
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    public struct ShellExecuteInfo {
        public uint size, mask;
        public IntPtr owner;
        public string verb, file, parameters, directory;
        public int show;
        public IntPtr instance, idList;
        public string className;
        public IntPtr classKey;
        public uint hotKey;
        public IntPtr icon, process;
    }
    [DllImport("shell32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern bool ShellExecuteEx(ref ShellExecuteInfo info);
}
'@
    $info = New-Object 'TestHandler+ShellExecuteInfo'
    $info.size = [Runtime.InteropServices.Marshal]::SizeOf($info)
    $info.mask = 1
    $info.verb = 'open'
    $info.file = (Get-Item -LiteralPath $Target).FullName
    $info.show = 1
    $info.className = 'PicGlance.Desktop.png'
    if (-not [TestHandler]::ShellExecuteEx([ref]$info)) {
        throw [ComponentModel.Win32Exception]::new([Runtime.InteropServices.Marshal]::GetLastWin32Error())
    }
    return
}

Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class TestWindow {
    private delegate bool EnumCallback(IntPtr window, IntPtr data);
    [DllImport("user32.dll")] private static extern bool EnumWindows(EnumCallback callback, IntPtr data);
    [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr window, out uint pid);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] private static extern int GetWindowText(IntPtr window, System.Text.StringBuilder name, int length);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr window);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr window, uint msg, IntPtr wp, IntPtr lp);
    public static IntPtr Find(uint expectedPid) {
        IntPtr result = IntPtr.Zero;
        EnumWindows((window, data) => {
            uint pid;
            GetWindowThreadProcessId(window, out pid);
            if (pid != expectedPid) return true;
            var title = new System.Text.StringBuilder(256);
            GetWindowText(window, title, title.Capacity);
            if (!title.ToString().StartsWith("PicGlance Desktop Test")) return true;
            result = window;
            return false;
        }, IntPtr.Zero);
        return result;
    }
}
'@
$handle = [TestWindow]::Find([uint32]$Target)
if ($Action -eq 'close') {
    if ($handle -eq 0 -or -not [TestWindow]::PostMessage($handle, 16, 0, 0)) { throw 'Failed to close test window.' }
} elseif ($Action -eq 'visible') {
    [TestWindow]::IsWindowVisible($handle)
}
