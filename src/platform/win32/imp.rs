use crate::{
    error::Error,
    platform::win32::ffi,
    window::WindowBuilder,
};
use std::{mem, ptr, slice, thread};

pub fn str_to_wstr(src: &str, buffer: &mut Vec<ffi::WCHAR>) -> ffi::LPCWSTR {
    // NOTE: Yes, indeed, `std::os::windows::ffi::OsStr(ing)ext` does exist in the standard library,
    // but it requires you to fit your data in the OsStr(ing) model and it's not hyper optimized
    // unlike mb2wc with handwritten SSE (allegedly), alongside being the native conversion function

    // MultiByteToWideChar can't actually handle 0 length because 0 return means error
    if src.is_empty() || src.len() > ffi::c_int::max_value() as usize {
        return [0x00].as_ptr()
    }

    unsafe {
        let str_ptr: ffi::LPCSTR = src.as_ptr().cast();
        let str_len = src.len() as ffi::c_int;

        // Calculate buffer size
        let req_buffer_size = ffi::MultiByteToWideChar(
            ffi::CP_UTF8, 0,
            str_ptr, str_len,
            ptr::null_mut(), 0, // `lpWideCharStr == NULL` means query size
        ) as usize + 1; // +1 for null terminator

        // Ensure buffer capacity
        buffer.clear();
        buffer.reserve(req_buffer_size);

        // Write to our buffer
        let chars_written = ffi::MultiByteToWideChar(
            ffi::CP_UTF8, 0,
            str_ptr, str_len,
            buffer.as_mut_ptr(), req_buffer_size as ffi::c_int,
        ) as usize;

        // Filter nulls, as Rust allows them in &str
        for x in slice::from_raw_parts_mut(buffer.as_mut_ptr(), chars_written) {
            if *x == 0x00 {
                *x = b' ' as ffi::WCHAR; // 0x00 => Space
            }
        }

        // Add null terminator & yield
        *buffer.as_mut_ptr().add(chars_written) = 0x00;
        buffer.set_len(req_buffer_size);
        buffer.as_ptr()
    }
}

pub struct WindowImpl {
    // ...
}

pub fn spawn_window(builder: &WindowBuilder) -> Result<WindowImpl, Error> {
    let builder = builder.clone();
    let thread = thread::spawn(move || {

        // no longer needed
        mem::drop(builder);
    });
    todo!()
}
