// win32 api goes here

// TODO: Remove `dead_code` when all is done
#![allow(bad_style, dead_code, overflowing_literals, clippy::upper_case_acronyms)]

// Opaque handles
macro_rules! def_handle {
    // documented, exported
    ($doc: literal, $name: ident, $private_name: ident $(,)?) => {
        #[doc(hidden)]
        pub enum $private_name {}
        #[doc = $doc]
        pub type $name = *mut $private_name;
    };
    // internal
    ($name: ident, $private_name: ident $(,)?) => {
        #[doc(hidden)]
        pub enum $private_name {}
        pub type $name = *mut $private_name;
    };
}
def_handle!("Opaque handle to the executable file in memory.", HINSTANCE, HINSTANCE__);
def_handle!("Opaque handle to a monitor.", HMONITOR, HMONITOR__);
def_handle!("Opaque handle to a window.", HWND, HWND__);
def_handle!(DPI_AWARENESS_CONTEXT, DPI_AWARENESS_CONTEXT__);
def_handle!(FARPROC, __some_function);
def_handle!(HBRUSH, HBRUSH__);
def_handle!(HDC, HDC__);
def_handle!(HHOOK, HHOOK__);
def_handle!(HICON, HICON__);
def_handle!(HMENU, HMENU__);
def_handle!(HMODULE, HMODULE__);
pub type HCURSOR = HICON;

// Typedefs
pub use core::ffi::c_void;
pub type c_char = i8;
pub type c_schar = i8;
pub type c_uchar = u8;
pub type c_short = i16;
pub type c_ushort = u16;
pub type c_int = i32;
pub type c_uint = u32;
pub type c_long = i32;
pub type c_ulong = u32;
pub type c_longlong = i64;
pub type c_ulonglong = u64;
pub type wchar_t = u16;
pub type ATOM = WORD;
pub type BOOL = c_int;
pub type BYTE = c_uchar;
pub type CHAR = c_char;
pub type DWORD = c_ulong;
pub type HANDLE = *mut c_void;
pub type HLOCAL = HANDLE;
pub type HRESULT = c_long;
pub type INT = c_int;
pub type LANGID = USHORT;
pub type LONG = c_long;
pub type LONG_PTR = isize;
pub type LPARAM = LONG_PTR;
pub type LRESULT = LONG_PTR;
pub type NTSTATUS = LONG;
pub type PROCESS_DPI_AWARENESS = u32;
pub type SHORT = c_short;
pub type UINT = c_uint;
pub type UINT_PTR = usize;
pub type ULONG_PTR = usize;
pub type USHORT = c_ushort;
pub type WCHAR = wchar_t;
pub type WORD = c_ushort;
pub type WPARAM = UINT_PTR;

// Function typedefs
pub type HOOKPROC = unsafe extern "system" fn(c_int, WPARAM, LPARAM) -> LRESULT;
pub type WNDPROC = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;

// Constants
pub const _WIN32_WINNT_VISTA: WORD = 0x0600;
pub const _WIN32_WINNT_WINBLUE: WORD = 0x0603;
pub const CCHILDREN_TITLEBAR: usize = 5;
pub const CP_UTF8: DWORD = 65001;
pub const CS_OWNDC: UINT = 0x0020;
pub const CW_USEDEFAULT: c_int = 0x80000000;
pub const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: DPI_AWARENESS_CONTEXT = -4isize as _;
pub const ERROR_SUCCESS: DWORD = 0; // lol
pub const FALSE: BOOL = 0;
pub const GCL_CBCLSEXTRA: c_int = -20;
pub const GWL_EXSTYLE: c_int = -20;
pub const GWL_STYLE: c_int = -16;
pub const GWL_USERDATA: c_int = -21;
pub const HCBT_DESTROYWND: c_int = 4;
pub const HTCAPTION: LRESULT = 2;
pub const IDC_APPSTARTING: *const WCHAR = 32650 as *const WCHAR;
pub const IDC_ARROW: *const WCHAR = 32512 as *const WCHAR;
pub const IDC_CROSS: *const WCHAR = 32515 as *const WCHAR;
pub const IDC_HAND: *const WCHAR = 32649 as *const WCHAR;
pub const IDC_HELP: *const WCHAR = 32651 as *const WCHAR;
pub const IDC_IBEAM: *const WCHAR = 32513 as *const WCHAR;
pub const IDC_ICON: *const WCHAR = 32641 as *const WCHAR;
pub const IDC_NO: *const WCHAR = 32648 as *const WCHAR;
pub const IDC_SIZE: *const WCHAR = 32640 as *const WCHAR;
pub const IDC_SIZEALL: *const WCHAR = 32646 as *const WCHAR;
pub const IDC_SIZENESW: *const WCHAR = 32643 as *const WCHAR;
pub const IDC_SIZENS: *const WCHAR = 32645 as *const WCHAR;
pub const IDC_SIZENWSE: *const WCHAR = 32642 as *const WCHAR;
pub const IDC_SIZEWE: *const WCHAR = 32644 as *const WCHAR;
pub const IDC_UPARROW: *const WCHAR = 32516 as *const WCHAR;
pub const IDC_WAIT: *const WCHAR = 32514 as *const WCHAR;
pub const IMAGE_CURSOR: UINT = 2;
pub const HTCLIENT: LRESULT = 1;
pub const LR_DEFAULTSIZE: UINT = 0x00000040;
pub const LR_SHARED: UINT = 0x00008000;
pub const MF_BYCOMMAND: UINT = 0x00000000;
pub const MF_DISABLED: UINT = 0x00000002;
pub const MF_ENABLED: UINT = 0x00000000;
pub const MF_GRAYED: UINT = 0x00000001;
pub const PROCESS_PER_MONITOR_DPI_AWARE: PROCESS_DPI_AWARENESS = 2;
pub const PROCESS_SYSTEM_DPI_AWARE: PROCESS_DPI_AWARENESS = 1;
pub const SC_CLOSE: WPARAM = 0xF060;
pub const SC_MAXIMIZE: WPARAM = 0xF030;
pub const SC_RESTORE: WPARAM = 0xF120;
pub const SIZE_RESTORED: WPARAM = 0;
pub const SIZE_MINIMIZED: WPARAM = 1;
pub const SIZE_MAXIMIZED: WPARAM = 2;
pub const SIZE_MAXSHOW: WPARAM = 3;
pub const SIZE_MAXHIDE: WPARAM = 4;
pub const SW_HIDE: c_int = 0;
pub const SW_SHOW: c_int = 5;
pub const SWP_ASYNCWINDOWPOS: UINT = 0x4000;
pub const SWP_DEFERERASE: UINT = 0x2000;
pub const SWP_DRAWFRAME: UINT = SWP_FRAMECHANGED;
pub const SWP_FRAMECHANGED: UINT = 0x0020;
pub const SWP_HIDEWINDOW: UINT = 0x0080;
pub const SWP_NOACTIVATE: UINT = 0x0010;
pub const SWP_NOCOPYBITS: UINT = 0x0100;
pub const SWP_NOMOVE: UINT = 0x0002;
pub const SWP_NOOWNERZORDER: UINT = 0x0200;
pub const SWP_NOREDRAW: UINT = 0x0008;
pub const SWP_NOREPOSITION: UINT = SWP_NOOWNERZORDER;
pub const SWP_NOSENDCHANGING: UINT = 0x0400;
pub const SWP_NOSIZE: UINT = 0x0001;
pub const SWP_NOZORDER: UINT = 0x0004;
pub const SWP_SHOWWINDOW: UINT = 0x0040;
pub const TRUE: BOOL = 1;
pub const VER_BUILDNUMBER: DWORD = 0x0000004;
pub const VER_GREATER_EQUAL: BYTE = 3;
pub const VER_MAJORVERSION: DWORD = 0x0000002;
pub const VER_MINORVERSION: DWORD = 0x0000001;
pub const VER_SERVICEPACKMAJOR: DWORD = 0x0000020;
pub const VER_SERVICEPACKMINOR: DWORD = 0x0000010;
pub const WH_CBT: c_int = 5;
pub const WM_NULL: UINT = 0x0000;
pub const WM_CREATE: UINT = 0x0001;
pub const WM_DESTROY: UINT = 0x0002;
pub const WM_MOVE: UINT = 0x0003;
pub const WM_SIZE: UINT = 0x0005;
pub const WM_ACTIVATE: UINT = 0x0006;
pub const WM_SETFOCUS: UINT = 0x0007;
pub const WM_KILLFOCUS: UINT = 0x0008;
pub const WM_ENABLE: UINT = 0x000A;
pub const WM_SETREDRAW: UINT = 0x000B;
pub const WM_SETTEXT: UINT = 0x000C;
pub const WM_PAINT: UINT = 0x000F;
pub const WM_CLOSE: UINT = 0x0010;
pub const WM_ERASEBKGND: UINT = 0x0014;
pub const WM_SHOWWINDOW: UINT = 0x0018;
pub const WM_ACTIVATEAPP: UINT = 0x001C;
pub const WM_SETCURSOR: UINT = 0x0020;
pub const WM_NCCREATE: UINT = 0x0081;
pub const WM_NCDESTROY: UINT = 0x0082;
pub const WM_NCLBUTTONDOWN: UINT = 0x00A1;
pub const WM_KEYDOWN: UINT = 0x0100;
pub const WM_KEYUP: UINT = 0x0101;
pub const WM_SYSKEYDOWN: UINT = 0x0104;
pub const WM_SYSKEYUP: UINT = 0x0105;
pub const WM_SYSCOMMAND: UINT = 0x0112;
pub const WM_MOUSEMOVE: UINT = 0x0200;
pub const WM_LBUTTONDOWN: UINT = 0x0201;
pub const WM_LBUTTONUP: UINT = 0x0202;
pub const WM_RBUTTONDOWN: UINT = 0x0204;
pub const WM_RBUTTONUP: UINT = 0x0205;
pub const WM_MBUTTONDOWN: UINT = 0x0207;
pub const WM_MBUTTONUP: UINT = 0x0208;
pub const WM_MOUSEWHEEL: UINT = 0x020A;
pub const WM_XBUTTONDOWN: UINT = 0x020B;
pub const WM_XBUTTONUP: UINT = 0x020C;
pub const WM_MOVING: UINT = 0x0216;
pub const WM_EXITSIZEMOVE: UINT = 0x0232;
pub const WM_USER: UINT = 0x0400;
pub const WS_BORDER: DWORD = 0x00800000;
pub const WS_CAPTION: DWORD = 0x00C00000;
pub const WS_CHILD: DWORD = 0x40000000;
pub const WS_CLIPCHILDREN: DWORD = 0x02000000;
pub const WS_CLIPSIBLINGS: DWORD = 0x04000000;
pub const WS_DISABLED: DWORD = 0x08000000;
pub const WS_DLGFRAME: DWORD = 0x00400000;
pub const WS_EX_LAYOUTRTL: DWORD = 0x00400000;
pub const WS_EX_TOOLWINDOW: DWORD = 0x00000080;
pub const WS_GROUP: DWORD = 0x00020000;
pub const WS_HSCROLL: DWORD = 0x00100000;
pub const WS_ICONIC: DWORD = WS_MINIMIZE;
pub const WS_MAXIMIZE: DWORD = 0x01000000;
pub const WS_MAXIMIZEBOX: DWORD = 0x00010000;
pub const WS_MINIMIZE: DWORD = 0x20000000;
pub const WS_MINIMIZEBOX: DWORD = 0x00020000;
pub const WS_OVERLAPPED: DWORD = 0x00000000;
pub const WS_OVERLAPPEDWINDOW: DWORD =
    WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
pub const WS_POPUP: DWORD = 0x80000000;
pub const WS_SIZEBOX: DWORD = WS_THICKFRAME;
pub const WS_SYSMENU: DWORD = 0x00080000;
pub const WS_TABSTOP: DWORD = 0x00010000;
pub const WS_THICKFRAME: DWORD = 0x00040000;
pub const WS_TILED: DWORD = WS_OVERLAPPED;
pub const WS_TILEDWINDOW: DWORD = WS_OVERLAPPEDWINDOW;
pub const WS_VISIBLE: DWORD = 0x10000000;
pub const WS_VSCROLL: DWORD = 0x00200000;
pub const XBUTTON1: WORD = 0x0001;
pub const XBUTTON2: WORD = 0x0002;

// Structs
#[repr(C)]
pub struct POINT {
    pub x: LONG,
    pub y: LONG,
}
#[repr(C)]
#[derive(Debug)]
pub struct RECT {
    pub left: LONG,
    pub top: LONG,
    pub right: LONG,
    pub bottom: LONG,
}
#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: UINT,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: DWORD,
    pub pt: POINT,
}
#[repr(C)]
pub struct OSVERSIONINFOEXW {
    pub dwOSVersionInfoSize: DWORD,
    pub dwMajorVersion: DWORD,
    pub dwMinorVersion: DWORD,
    pub dwBuildNumber: DWORD,
    pub dwPlatformId: DWORD,
    pub szCSDVersion: [WCHAR; 128],
    pub wServicePackMajor: WORD,
    pub wServicePackMinor: WORD,
    pub wSuiteMask: WORD,
    pub wProductType: BYTE,
    pub wReserved: BYTE,
}
#[repr(C)]
pub struct TITLEBARINFO {
    pub cbSize: DWORD,
    pub rcTitleBar: RECT,
    pub rgstate: [DWORD; CCHILDREN_TITLEBAR + 1],
}
#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: UINT,
    pub style: UINT,
    pub lpfnWndProc: WNDPROC,
    pub cbClsExtra: c_int,
    pub cbWndExtra: c_int,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: *const WCHAR,
    pub lpszClassName: *const WCHAR,
    pub hIconSm: HICON,
}
#[repr(C)]
pub struct IMAGE_DOS_HEADER {
    e_magic: WORD,
    e_cblp: WORD,
    e_cp: WORD,
    e_crlc: WORD,
    e_cparhdr: WORD,
    e_minalloc: WORD,
    e_maxalloc: WORD,
    e_ss: WORD,
    e_sp: WORD,
    e_csum: WORD,
    e_ip: WORD,
    e_cs: WORD,
    e_lfarlc: WORD,
    e_ovno: WORD,
    e_res: [WORD; 4],
    e_oemid: WORD,
    e_oeminfo: WORD,
    e_res2: [WORD; 10],
    e_lfanew: LONG,
}

// Static Linked Functions
#[link(name = "Kernel32")]
extern "system" {
    pub fn GetLastError() -> DWORD;
    pub fn SetLastError(dwErrCode: DWORD);
    pub fn ExitProcess(uExitCode: UINT);
    pub fn GetCurrentThreadId() -> DWORD;
    pub fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: *const CHAR,
        cbMultiByte: c_int,
        lpWideCharStr: *mut WCHAR,
        cchWideChar: c_int,
    ) -> c_int;
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: *const CHAR) -> FARPROC;
    pub fn LoadLibraryExA(lpLibFileName: *const CHAR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE;
    pub fn VerSetConditionMask(ConditionMask: c_ulonglong, TypeMask: DWORD, Condition: BYTE) -> c_ulonglong;
}
#[link(name = "User32")]
extern "system" {
    // Window class management
    pub fn GetClassInfoExW(hinst: HINSTANCE, lpszClass: *const WCHAR, lpwcx: *mut WNDCLASSEXW) -> BOOL;
    pub fn RegisterClassExW(lpWndClass: *const WNDCLASSEXW) -> ATOM;

    // Window management
    pub fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: *const WCHAR,
        lpWindowName: *const WCHAR,
        dwStyle: DWORD,
        x: c_int,
        y: c_int,
        nWidth: c_int,
        nHeight: c_int,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: *mut c_void,
    ) -> HWND;
    pub fn AdjustWindowRectEx(lpRect: *mut RECT, dwStyle: DWORD, bMenu: BOOL, dwExStyle: DWORD) -> BOOL;
    pub fn ClientToScreen(hWnd: HWND, lpPoint: *mut POINT) -> BOOL;
    pub fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn GetTitleBarInfo(hwnd: HWND, pti: *mut TITLEBARINFO) -> BOOL;
    pub fn SetWindowPos(hWnd: HWND, hWndInsertAfter: HWND, X: c_int, Y: c_int, cx: c_int, cy: c_int, uFlags: UINT) -> BOOL;
    pub fn WindowFromPoint(Point: POINT) -> HWND;
    pub fn DestroyWindow(hWnd: HWND) -> BOOL;

    // Hooking API
    pub fn CallNextHookEx(hhk: HHOOK, nCode: c_int, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn SetWindowsHookExW(idHook: c_int, lpfn: HOOKPROC, hmod: HINSTANCE, dwThreadId: DWORD) -> HHOOK;
    pub fn UnhookWindowsHookEx(hhk: HHOOK) -> BOOL;

    // Message loop
    pub fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT) -> BOOL;
    pub fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    pub fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn DispatchMessageW(lpmsg: *const MSG) -> LRESULT;
    pub fn PostQuitMessage(nExitCode: c_int);

    // Message loop utility
    pub fn ShowWindow(hWnd: HWND, nCmdShow: c_int) -> BOOL;
    pub fn ShowWindowAsync(hWnd: HWND, nCmdShow: c_int) -> BOOL;

    // Keyboard & mouse related
    pub fn ClipCursor(lpRect: *const RECT) -> BOOL;
    pub fn SetCursorPos(X: c_int, Y: c_int) -> BOOL;
    pub fn GetCursorPos(lpPoint: *mut POINT) -> BOOL;
    pub fn SetCapture(hWnd: HWND) -> HWND;
    pub fn GetCapture() -> HWND;
    pub fn ReleaseCapture() -> BOOL;
    pub fn GetAsyncKeyState(vKey: c_int) -> SHORT;
    pub fn GetSystemMetrics(nIndex: c_int) -> c_int;
    pub fn SetCursor(hCursor: HCURSOR) -> HCURSOR;

    // Misc legacy garbage
    pub fn EnableMenuItem(hMenu: HMENU, uIDEnableItem: UINT, uEnable: UINT) -> BOOL;
    pub fn GetSystemMenu(hWnd: HWND, bRevert: BOOL) -> HMENU;

    // Yeah, whatever
    pub fn LoadImageW(hInst: HINSTANCE, name: *const WCHAR, type_: UINT, cx: c_int, cy: c_int, fuLoad: UINT) -> HANDLE;
    pub fn PtInRect(lprc: *const RECT, pt: POINT) -> BOOL;

    // Class/window storage manipulation
    pub fn GetClassLongW(hWnd: HWND, nIndex: c_int) -> DWORD;
    pub fn SetClassLongW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG) -> DWORD;
    pub fn GetWindowLongW(hWnd: HWND, nIndex: c_int) -> LONG;
    pub fn SetWindowLongW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG) -> LONG;
    #[cfg(target_pointer_width = "64")]
    pub fn GetClassLongPtrW(hWnd: HWND, nIndex: c_int) -> ULONG_PTR;
    #[cfg(target_pointer_width = "64")]
    pub fn SetClassLongPtrW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG_PTR) -> ULONG_PTR;
    #[cfg(target_pointer_width = "64")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: c_int) -> LONG_PTR;
    #[cfg(target_pointer_width = "64")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG_PTR) -> LONG_PTR;
}

#[inline]
unsafe fn dlopen(name: *const CHAR) -> HMODULE {
    // Patch loading mechanism here, if you wish
    LoadLibraryExA(name, 0 as HANDLE, 0)
}

dyn_link! {
    pub struct Win32DL(dlopen => HMODULE | GetProcAddress) {
        "Dwmapi.dll" {
            /// (Windows Vista+)
            /// Advanced querying of window attributes via the desktop window manager.
            fn DwmGetWindowAttribute(
                hWnd: HWND,
                dwAttribute: DWORD,
                pvAttribute: *mut c_void,
                cbAttribute: DWORD,
            ) -> HRESULT;

            /// (Windows Vista+)
            /// Advanced setting of window attributes via the desktop window manager.
            fn DwmSetWindowAttribute(
                hWnd: HWND,
                dwAttribute: DWORD,
                pvAttribute: *const c_void,
                cbAttribute: DWORD,
            ) -> HRESULT;
        },

        "Ntdll.dll" {
            /// (Win2000+)
            /// This is used in place of VerifyVersionInfoW, as it's not manifest dependent, and doesn't lie.
            fn RtlVerifyVersionInfo(
                VersionInfo: *mut OSVERSIONINFOEXW,
                TypeMask: DWORD,
                ConditionMask: c_ulonglong,
            ) -> NTSTATUS;
        },

        "Shcore.dll" {
            /// (Win8.1+)
            /// The intended way to query a monitor's DPI values since PMv1 and above.
            fn GetDpiForMonitor(
                hmonitor: HMONITOR,
                dpiType: u32,
                dpiX: *mut UINT,
                dpiY: *mut UINT,
            ) -> HRESULT;
        },

        "User32.dll" {
            // (Win10 1607+)
            // It's a version of AdjustWindowRectEx with DPI, but they added it 7 years late.
            // The DPI parameter accounts for scaled non-client areas, not to scale client areas.
            fn AdjustWindowRectExForDpi(
                lpRect: *mut RECT,
                dwStyle: DWORD,
                bMenu: BOOL,
                dwExStyle: DWORD,
                dpi: UINT,
            ) -> BOOL;

            /// (Win10 1603+)
            /// Enables automatic scaling of the non-client area as a hack for PMv1 DPI mode.
            fn EnableNonClientDpiScaling(hwnd: HWND) -> BOOL;

            /// (Vista+)
            /// First introduction of DPI awareness, this function enables System-Aware DPI.
            fn SetProcessDPIAware() -> BOOL;

            /// (Win8.1+)
            /// Allows you to set either System-Aware DPI mode, or Per-Monitor-Aware (v1).
            fn SetProcessDpiAwareness(value: PROCESS_DPI_AWARENESS) -> HRESULT;

            /// (Win10 1703+)
            /// Allows you to set either System-Aware DPI mode, or Per-Monitor-Aware (v1 *or* v2).
            fn SetProcessDpiAwarenessContext(value: DPI_AWARENESS_CONTEXT) -> BOOL;
        },
    }
}

impl Win32DL {
    pub unsafe fn link() -> Self {
        // Trying to load a nonexistent dynamic library or symbol sets the thread-global error.
        // Since this is intended and acceptable for missing functions, we restore the error state.

        let prev_error = GetLastError();
        let instance = Self::_link();
        SetLastError(prev_error);
        instance
    }
}

// (Get/Set)(Class/Window)Long(A/W) all took LONG, a 32-bit type.
// When MS went from 32 to 64 bit, they realized how big of a mistake this was,
// seeing as some of those values need to be as big as a pointer is (like size_t).
// To make things worse, they exported the 32-bit ones on 64-bit with mismatching signatures.
// These functions wrap both of those function sets to `usize`, which matches on 32 & 64 bit.
#[cfg(target_pointer_width = "32")]
#[inline]
pub unsafe fn get_class_data(hwnd: HWND, offset: c_int) -> usize {
    GetClassLongW(hwnd, offset) as usize
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub unsafe fn get_class_data(hwnd: HWND, offset: c_int) -> usize {
    GetClassLongPtrW(hwnd, offset) as usize
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub unsafe fn set_class_data(hwnd: HWND, offset: c_int, data: usize) -> usize {
    SetClassLongW(hwnd, offset, data as LONG) as usize
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub unsafe fn set_class_data(hwnd: HWND, offset: c_int, data: usize) -> usize {
    SetClassLongPtrW(hwnd, offset, data as LONG_PTR) as usize
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub unsafe fn get_window_data(hwnd: HWND, offset: c_int) -> usize {
    GetWindowLongW(hwnd, offset) as usize
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub unsafe fn get_window_data(hwnd: HWND, offset: c_int) -> usize {
    GetWindowLongPtrW(hwnd, offset) as usize
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub unsafe fn set_window_data(hwnd: HWND, offset: c_int, data: usize) -> usize {
    SetWindowLongW(hwnd, offset, data as LONG) as usize
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub unsafe fn set_window_data(hwnd: HWND, offset: c_int, data: usize) -> usize {
    SetWindowLongPtrW(hwnd, offset, data as LONG_PTR) as usize
}

// WINAPI defines these as `int` but that's annoying and stupid for several reasons.
// We redefine them as u8's.
pub const VK_LBUTTON: u8 = 0x01;
pub const VK_RBUTTON: u8 = 0x02;
pub const VK_CANCEL: u8 = 0x03;
pub const VK_MBUTTON: u8 = 0x04;
pub const VK_XBUTTON1: u8 = 0x05;
pub const VK_XBUTTON2: u8 = 0x06;
pub const VK_BACK: u8 = 0x08;
pub const VK_TAB: u8 = 0x09;
pub const VK_CLEAR: u8 = 0x0C;
pub const VK_RETURN: u8 = 0x0D;
pub const VK_SHIFT: u8 = 0x10;
pub const VK_CONTROL: u8 = 0x11;
pub const VK_MENU: u8 = 0x12;
pub const VK_PAUSE: u8 = 0x13;
pub const VK_CAPITAL: u8 = 0x14;
pub const VK_KANA: u8 = 0x15;
pub const VK_IME_ON: u8 = 0x16;
pub const VK_JUNJA: u8 = 0x17;
pub const VK_FINAL: u8 = 0x18;
pub const VK_HANJA: u8 = 0x19;
pub const VK_KANJI: u8 = 0x19;
pub const VK_IME_OFF: u8 = 0x1A;
pub const VK_ESCAPE: u8 = 0x1B;
pub const VK_CONVERT: u8 = 0x1C;
pub const VK_NONCONVERT: u8 = 0x1D;
pub const VK_ACCEPT: u8 = 0x1E;
pub const VK_MODECHANGE: u8 = 0x1F;
pub const VK_SPACE: u8 = 0x20;
pub const VK_PRIOR: u8 = 0x21;
pub const VK_NEXT: u8 = 0x22;
pub const VK_END: u8 = 0x23;
pub const VK_HOME: u8 = 0x24;
pub const VK_LEFT: u8 = 0x25;
pub const VK_UP: u8 = 0x26;
pub const VK_RIGHT: u8 = 0x27;
pub const VK_DOWN: u8 = 0x28;
pub const VK_SELECT: u8 = 0x29;
pub const VK_PRINT: u8 = 0x2A;
pub const VK_EXECUTE: u8 = 0x2B;
pub const VK_SNAPSHOT: u8 = 0x2C;
pub const VK_INSERT: u8 = 0x2D;
pub const VK_DELETE: u8 = 0x2E;
pub const VK_HELP: u8 = 0x2F;
pub const VK_LWIN: u8 = 0x5B;
pub const VK_RWIN: u8 = 0x5C;
pub const VK_APPS: u8 = 0x5D;
pub const VK_SLEEP: u8 = 0x5F;
pub const VK_NUMPAD0: u8 = 0x60;
pub const VK_NUMPAD1: u8 = 0x61;
pub const VK_NUMPAD2: u8 = 0x62;
pub const VK_NUMPAD3: u8 = 0x63;
pub const VK_NUMPAD4: u8 = 0x64;
pub const VK_NUMPAD5: u8 = 0x65;
pub const VK_NUMPAD6: u8 = 0x66;
pub const VK_NUMPAD7: u8 = 0x67;
pub const VK_NUMPAD8: u8 = 0x68;
pub const VK_NUMPAD9: u8 = 0x69;
pub const VK_MULTIPLY: u8 = 0x6A;
pub const VK_ADD: u8 = 0x6B;
pub const VK_SEPARATOR: u8 = 0x6C;
pub const VK_SUBTRACT: u8 = 0x6D;
pub const VK_DECIMAL: u8 = 0x6E;
pub const VK_DIVIDE: u8 = 0x6F;
pub const VK_F1: u8 = 0x70;
pub const VK_F2: u8 = 0x71;
pub const VK_F3: u8 = 0x72;
pub const VK_F4: u8 = 0x73;
pub const VK_F5: u8 = 0x74;
pub const VK_F6: u8 = 0x75;
pub const VK_F7: u8 = 0x76;
pub const VK_F8: u8 = 0x77;
pub const VK_F9: u8 = 0x78;
pub const VK_F10: u8 = 0x79;
pub const VK_F11: u8 = 0x7A;
pub const VK_F12: u8 = 0x7B;
pub const VK_F13: u8 = 0x7C;
pub const VK_F14: u8 = 0x7D;
pub const VK_F15: u8 = 0x7E;
pub const VK_F16: u8 = 0x7F;
pub const VK_F17: u8 = 0x80;
pub const VK_F18: u8 = 0x81;
pub const VK_F19: u8 = 0x82;
pub const VK_F20: u8 = 0x83;
pub const VK_F21: u8 = 0x84;
pub const VK_F22: u8 = 0x85;
pub const VK_F23: u8 = 0x86;
pub const VK_F24: u8 = 0x87;
pub const VK_NAVIGATION_VIEW: u8 = 0x88;
pub const VK_NAVIGATION_MENU: u8 = 0x89;
pub const VK_NAVIGATION_UP: u8 = 0x8A;
pub const VK_NAVIGATION_DOWN: u8 = 0x8B;
pub const VK_NAVIGATION_LEFT: u8 = 0x8C;
pub const VK_NAVIGATION_RIGHT: u8 = 0x8D;
pub const VK_NAVIGATION_ACCEPT: u8 = 0x8E;
pub const VK_NAVIGATION_CANCEL: u8 = 0x8F;
pub const VK_NUMLOCK: u8 = 0x90;
pub const VK_SCROLL: u8 = 0x91;
pub const VK_OEM_NEC_EQUAL: u8 = 0x92;
pub const VK_OEM_FJ_JISHO: u8 = 0x92;
pub const VK_OEM_FJ_MASSHOU: u8 = 0x93;
pub const VK_OEM_FJ_TOUROKU: u8 = 0x94;
pub const VK_OEM_FJ_LOYA: u8 = 0x95;
pub const VK_OEM_FJ_ROYA: u8 = 0x96;
pub const VK_LSHIFT: u8 = 0xA0;
pub const VK_RSHIFT: u8 = 0xA1;
pub const VK_LCONTROL: u8 = 0xA2;
pub const VK_RCONTROL: u8 = 0xA3;
pub const VK_LMENU: u8 = 0xA4;
pub const VK_RMENU: u8 = 0xA5;
pub const VK_BROWSER_BACK: u8 = 0xA6;
pub const VK_BROWSER_FORWARD: u8 = 0xA7;
pub const VK_BROWSER_REFRESH: u8 = 0xA8;
pub const VK_BROWSER_STOP: u8 = 0xA9;
pub const VK_BROWSER_SEARCH: u8 = 0xAA;
pub const VK_BROWSER_FAVORITES: u8 = 0xAB;
pub const VK_BROWSER_HOME: u8 = 0xAC;
pub const VK_VOLUME_MUTE: u8 = 0xAD;
pub const VK_VOLUME_DOWN: u8 = 0xAE;
pub const VK_VOLUME_UP: u8 = 0xAF;
pub const VK_MEDIA_NEXT_TRACK: u8 = 0xB0;
pub const VK_MEDIA_PREV_TRACK: u8 = 0xB1;
pub const VK_MEDIA_STOP: u8 = 0xB2;
pub const VK_MEDIA_PLAY_PAUSE: u8 = 0xB3;
pub const VK_LAUNCH_MAIL: u8 = 0xB4;
pub const VK_LAUNCH_MEDIA_SELECT: u8 = 0xB5;
pub const VK_LAUNCH_APP1: u8 = 0xB6;
pub const VK_LAUNCH_APP2: u8 = 0xB7;
pub const VK_OEM_1: u8 = 0xBA;
pub const VK_OEM_PLUS: u8 = 0xBB;
pub const VK_OEM_COMMA: u8 = 0xBC;
pub const VK_OEM_MINUS: u8 = 0xBD;
pub const VK_OEM_PERIOD: u8 = 0xBE;
pub const VK_OEM_2: u8 = 0xBF;
pub const VK_OEM_3: u8 = 0xC0;
pub const VK_OEM_4: u8 = 0xDB;
pub const VK_OEM_5: u8 = 0xDC;
pub const VK_OEM_6: u8 = 0xDD;
pub const VK_OEM_7: u8 = 0xDE;
pub const VK_OEM_8: u8 = 0xDF;
pub const VK_OEM_AX: u8 = 0xE1;
pub const VK_OEM_102: u8 = 0xE2;
pub const VK_ICO_HELP: u8 = 0xE3;
pub const VK_ICO_00: u8 = 0xE4;
pub const VK_PROCESSKEY: u8 = 0xE5;
pub const VK_ICO_CLEAR: u8 = 0xE6;
pub const VK_PACKET: u8 = 0xE7;
pub const VK_OEM_RESET: u8 = 0xE9;
pub const VK_OEM_JUMP: u8 = 0xEA;
pub const VK_OEM_PA1: u8 = 0xEB;
pub const VK_OEM_PA2: u8 = 0xEC;
pub const VK_OEM_PA3: u8 = 0xED;
pub const VK_OEM_WSCTRL: u8 = 0xEE;
pub const VK_OEM_CUSEL: u8 = 0xEF;
pub const VK_OEM_ATTN: u8 = 0xF0;
pub const VK_OEM_FINISH: u8 = 0xF1;
pub const VK_OEM_COPY: u8 = 0xF2;
pub const VK_OEM_AUTO: u8 = 0xF3;
pub const VK_OEM_ENLW: u8 = 0xF4;
pub const VK_OEM_BACKTAB: u8 = 0xF5;
pub const VK_ATTN: u8 = 0xF6;
pub const VK_CRSEL: u8 = 0xF7;
pub const VK_EXSEL: u8 = 0xF8;
pub const VK_EREOF: u8 = 0xF9;
pub const VK_PLAY: u8 = 0xFA;
pub const VK_ZOOM: u8 = 0xFB;
pub const VK_NONAME: u8 = 0xFC;
pub const VK_PA1: u8 = 0xFD;
pub const VK_OEM_CLEAR: u8 = 0xFE;
