use crate::{
    error::Error,
    platform::win32::ffi,
    util::LazyCell,
    window::{self, WindowBuilder},
};
use std::{cell::UnsafeCell, mem, ptr, slice, sync::Mutex, thread};

// (Get/Set)(Class/Window)Long(A/W) all took LONG, a 32-bit type.
// When MS went from 32 to 64 bit, they realized how big of a mistake this was,
// seeing as some of those values need to be as big as a pointer is (like size_t).
// Unfortunately they exported the 32-bit ones on 64-bit with mismatching signatures.
// These functions wrap both of those function sets to `usize`, which matches on 32 & 64 bit.
#[cfg(target_pointer_width = "32")]
#[inline]
pub unsafe fn get_class_data(hwnd: ffi::HWND, offset: ffi::c_int) -> usize {
    ffi::GetClassLongW(hwnd, offset) as usize
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub unsafe fn get_class_data(hwnd: ffi::HWND, offset: ffi::c_int) -> usize {
    ffi::GetClassLongPtrW(hwnd, offset) as usize
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub unsafe fn set_class_data(hwnd: ffi::HWND, offset: ffi::c_int, data: usize) -> usize {
    ffi::SetClassLongW(hwnd, offset, data as ffi::LONG) as usize
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub unsafe fn set_class_data(hwnd: ffi::HWND, offset: ffi::c_int, data: usize) -> usize {
    ffi::SetClassLongPtrW(hwnd, offset, data as ffi::LONG_PTR) as usize
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub unsafe fn get_window_data(hwnd: ffi::HWND, offset: ffi::c_int) -> usize {
    ffi::GetWindowLongW(hwnd, offset) as usize
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub unsafe fn get_window_data(hwnd: ffi::HWND, offset: ffi::c_int) -> usize {
    ffi::GetWindowLongPtrW(hwnd, offset) as usize
}
#[cfg(target_pointer_width = "32")]
#[inline]
pub unsafe fn set_window_data(hwnd: ffi::HWND, offset: ffi::c_int, data: usize) -> usize {
    ffi::SetWindowLongW(hwnd, offset, data as ffi::LONG) as usize
}
#[cfg(target_pointer_width = "64")]
#[inline]
pub unsafe fn set_window_data(hwnd: ffi::HWND, offset: ffi::c_int, data: usize) -> usize {
    ffi::SetWindowLongPtrW(hwnd, offset, data as ffi::LONG_PTR) as usize
}

/// Converts a &str to an LPCWSTR-compatible string array.
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

impl window::Style {
    /// Gets this style as a bitfield. Note that it does not include the close button.
    /// The close button is a menu property, not a window style.
    pub(crate) fn dword_style(&self) -> ffi::DWORD {
        let mut style = 0;

        if self.borderless {
            // TODO: Why does this just not work without THICKFRAME? Borderless is dumb.
            style |= ffi::WS_POPUP | ffi::WS_THICKFRAME;
        } else {
            style |= ffi::WS_OVERLAPPED | ffi::WS_BORDER | ffi::WS_CAPTION;
        }

        if self.resizable {
            style |= ffi::WS_THICKFRAME;
        }

        if self.visible {
            style |= ffi::WS_VISIBLE;
        }

        if let Some(controls) = &self.controls {
            if controls.minimize {
                style |= ffi::WS_MINIMIZEBOX;
            }
            if controls.maximize {
                style |= ffi::WS_MAXIMIZEBOX;
            }
            style |= ffi::WS_SYSMENU;
        }

        style
    }

    /// Gets the extended window style.
    pub(crate) fn dword_style_ex(&self) -> ffi::DWORD {
        let mut style = 0;

        if self.rtl_layout {
            style |= ffi::WS_EX_LAYOUTRTL;
        }

        if self.tool_window {
            style |= ffi::WS_EX_TOOLWINDOW;
        }

        style
    }

    /// Sets both styles for target window handle.
    pub(crate) fn set_for(&self, hwnd: ffi::HWND) {
        let style = self.dword_style();
        let style_ex = self.dword_style_ex();
        unsafe {
            let _ = set_window_data(hwnd, ffi::GWL_STYLE, style as usize);
            let _ = set_window_data(hwnd, ffi::GWL_EXSTYLE, style_ex as usize);
        }
    }
}

/// Implementation container for `Window`
pub struct WindowImpl {
    // ...
}

/// Info struct for `WM_(NC)CREATE`
pub struct WindowImplCreateParams {
    user_data: *mut WindowImplData,
}

/// User data structure
pub struct WindowImplData {
    // ...
}

impl Default for WindowImplData {
    fn default() -> Self {
        Self {
            // ...
        }
    }
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

        let style = builder.style.dword_style();
        let style_ex = builder.style.dword_style_ex();
        let (width, height) = (1280, 720);
        let (pos_x, pos_y) = (ffi::CW_USEDEFAULT, ffi::CW_USEDEFAULT);
        let user_data: Box<UnsafeCell<WindowImplData>> = Default::default();

        // A user pointer is supplied for `WM_NCCREATE` & `WM_CREATE` as lpParam
        let create_params = WindowImplCreateParams {
            user_data: user_data.get(),
        };
        let hwnd = ffi::CreateWindowExW(
            style_ex,
            class_name,
            title,
            style,
            pos_x,
            pos_y,
            width,
            height,
            ptr::null_mut(), // parent hwnd
            ptr::null_mut(), // menu handle
            this_hinstance(),
            (&create_params) as *const _ as ffi::LPVOID,
        );
        println!("RESULT: {:?}", hwnd);

        // ... (Create)

        // No longer needed, free memory
        mem::drop(builder);
        mem::drop(class_name_buf);
        mem::drop(title_buf);
    });
    loop {}
    todo!()
}

unsafe extern "system" fn window_proc(
    hwnd: ffi::HWND,
    msg: ffi::UINT,
    wparam: ffi::WPARAM,
    lparam: ffi::LPARAM,
) -> ffi::LRESULT {
    ffi::DefWindowProcW(hwnd, msg, wparam, lparam)
}
