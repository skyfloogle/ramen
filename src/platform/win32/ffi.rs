// win32 api goes here

#![allow(bad_style, dead_code)] // shut up for a bit

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
use core::ffi::c_void;
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
pub type LPCSTR = *const CHAR;
pub type LPCVOID = *const c_void;
pub type LPCWSTR = *const WCHAR;
pub type LPVOID = *mut c_void;
pub type LPWSTR = *mut WCHAR;
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
pub const CP_UTF8: DWORD = 65001;
pub const CS_OWNDC: UINT = 0x0020;
pub const ERROR_SUCCESS: DWORD = 0; // lol

// Structs
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
    pub lpszMenuName: LPCWSTR,
    pub lpszClassName: LPCWSTR,
    pub hIconSm: HICON,
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
        lpMultiByteStr: LPCSTR,
        cbMultiByte: c_int,
        lpWideCharStr: LPWSTR,
        cchWideChar: c_int,
    ) -> c_int;
}
#[link(name = "User32")]
extern "system" {
    // Window class management
    pub fn GetClassInfoExW(hinst: HINSTANCE, lpszClass: LPCWSTR, lpwcx: *mut WNDCLASSEXW) -> BOOL;
    pub fn RegisterClassExW(lpWndClass: *const WNDCLASSEXW) -> ATOM;
}
