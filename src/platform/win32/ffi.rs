// win32 api goes here

#![allow(dead_code)] // shut up for a bit

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

// Constants
pub const CP_UTF8: DWORD = 65001;

// Static Linked Functions
#[link(name = "Kernel32")]
extern "system" {
    pub fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: c_int,
        lpWideCharStr: LPWSTR,
        cchWideChar: c_int,
    ) -> c_int;
}
