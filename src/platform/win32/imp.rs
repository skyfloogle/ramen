use crate::{
    error::Error,
    platform::win32::ffi,
    util::LazyCell,
    window::WindowBuilder,
};
use std::{mem, ptr, slice, sync::Mutex, thread};

fn str_to_wstr(src: &str, buffer: &mut Vec<ffi::WCHAR>) -> ffi::LPCWSTR {
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
        // TODO: Does this mess up multi-byte UTF-16?
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

/// Retrieves the base module HINSTANCE.
#[inline]
pub fn this_hinstance() -> ffi::HINSTANCE {
    extern "system" {
        // Microsoft's linkers provide a static HINSTANCE to not have to query it at runtime.
        // Source: https://devblogs.microsoft.com/oldnewthing/20041025-00/?p=37483
        // (I love you Raymond Chen)
        static __ImageBase: [u8; 64];
    }
    (unsafe { &__ImageBase }) as *const [u8; 64] as ffi::HINSTANCE
}

pub struct WindowImpl {
    // ...
}

/// To avoid two threads trying to register a window class at the same time,
/// this global mutex is locked while doing window class queries / entries.
static CLASS_REGISTRY_LOCK: LazyCell<Mutex<()>> = LazyCell::new(|| Mutex::new(()));

pub fn spawn_window(builder: &WindowBuilder) -> Result<WindowImpl, Error> {
    let builder = builder.clone();
    let thread = thread::spawn(move || unsafe {
        // Convert class name & title to `WCHAR` string for Win32
        let mut class_name_buf = Vec::new();
        let class_name = str_to_wstr(builder.class_name.as_ref(), &mut class_name_buf);
        let mut title_buf = Vec::new();
        let title = str_to_wstr(builder.title.as_ref(), &mut title_buf);

        // Create the window class if it doesn't exist yet
        let mut class_created_this_thread = false;
        let class_registry_lock = CLASS_REGISTRY_LOCK.lock().unwrap();
        let mut class_info = mem::MaybeUninit::<ffi::WNDCLASSEXW>::uninit();
        (*class_info.as_mut_ptr()).cbSize = mem::size_of_val(&class_info) as ffi::DWORD;
        if ffi::GetClassInfoExW(this_hinstance(), class_name, class_info.as_mut_ptr()) == 0 {
            // The window class not existing sets the thread global error flag.
            ffi::SetLastError(ffi::ERROR_SUCCESS);

            // If this is the thread registering this window class,
            // it's the one responsible for setting class-specific data below
            class_created_this_thread = true;

            // Fill in & register class (`cbSize` is set before this if block)
            let class = &mut *class_info.as_mut_ptr();
            class.style = ffi::CS_OWNDC;
            class.lpfnWndProc = window_proc;
            class.cbClsExtra = mem::size_of::<usize>() as ffi::c_int;
            class.cbWndExtra = 0;
            class.hInstance = this_hinstance();
            class.hIcon = ptr::null_mut();
            class.hCursor = ptr::null_mut();
            class.hbrBackground = ptr::null_mut();
            class.lpszMenuName = ptr::null_mut();
            // TODO: Filter reserved class names
            class.lpszClassName = class_name;
            class.hIconSm = ptr::null_mut();

            // _: The fields on `WNDCLASSEXW` are known to be valid
            let _ = ffi::RegisterClassExW(class);
        }
        mem::drop(class_registry_lock);

        // ... (Create)

        // No longer needed, free memory
        mem::drop(builder);
        mem::drop(class_name_buf);
        mem::drop(title_buf);
    });
    todo!()
}

unsafe extern "system" fn window_proc(
    hwnd: ffi::HWND,
    msg: ffi::UINT,
    wparam: ffi::WPARAM,
    lparam: ffi::LPARAM,
) -> ffi::LRESULT {
    0
}