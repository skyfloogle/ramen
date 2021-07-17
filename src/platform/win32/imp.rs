#![allow(
    // "WM_USER + 0" is good C constant semantics.
    clippy::identity_op,

    // "This lint cannot detect if the mutex is actually used for waiting before a critical section."
    // Which it is being used for.
    clippy::mutex_atomic,
)]

use crate::{
    error::Error,
    event::{CloseReason, Event},
    monitor::{Scale, Size},
    util::{sync::{self, Condvar, Mutex}, FixedVec, LazyCell},
    window::{self, Cursor, WindowBuilder},
};
use std::{cell::UnsafeCell, mem, num::NonZeroI32, ops, ptr, sync::{atomic, Arc}, thread};

#[cfg(feature = "input")]
use crate::{event::{Key, MouseButton}, monitor::Point};

// TODO: Maybe deglob
use crate::platform::win32::ffi::*;

/// To avoid two threads trying to register a window class at the same time,
/// this global mutex is locked while doing window class queries / entries.
static CLASS_REGISTRY_LOCK: LazyCell<Mutex<()>> = LazyCell::new(|| Mutex::new(()));

// Global immutable struct containing dynamically acquired API state
static WIN32: LazyCell<Win32State> = LazyCell::new(Win32State::new);

/// TODO: yeah
const BASE_DPI: UINT = 96;
/// TODO: Measure this
const MAX_EVENTS_PER_SWAP: usize = 4096;
/// Marker to filter out implementation magic like `CicMarshalWndClass`
const HOOKPROC_MARKER: &[u8; 4] = b"viri";

// Custom window messages (see `window_proc` for docs)
const RAMEN_WM_DROP:          UINT = WM_USER + 0;
const RAMEN_WM_EXECUTE:       UINT = WM_USER + 1; // TODO:
// const RAMEN_WM_SETBORDERLESS: UINT = WM_USER + 2; // TODO:
const RAMEN_WM_SETCONTROLS:   UINT = WM_USER + 3;
const RAMEN_WM_SETCURSOR:     UINT = WM_USER + 4;
// const RAMEN_WM_SETFULLSCREEN: UINT = WM_USER + 5; // TODO:
const RAMEN_WM_SETTEXT_ASYNC: UINT = WM_USER + 6;
const RAMEN_WM_SETTHICKFRAME: UINT = WM_USER + 7;
const RAMEN_WM_SETINNERSIZE:  UINT = WM_USER + 8;
const RAMEN_WM_GETINNERSIZE:  UINT = WM_USER + 9;
const RAMEN_WM_ISDPILOGICAL:  UINT = WM_USER + 10;
const RAMEN_WM_SETMAXIMIZED:  UINT = WM_USER + 11;

/// Retrieves the base module [`HINSTANCE`].
#[inline]
pub fn this_hinstance() -> HINSTANCE {
    extern "system" {
        // Microsoft's linkers provide a static HINSTANCE to not have to query it at runtime.
        // More info: https://devblogs.microsoft.com/oldnewthing/20041025-00/?p=37483
        static __ImageBase: IMAGE_DOS_HEADER;
    }
    (unsafe { &__ImageBase }) as *const IMAGE_DOS_HEADER as HINSTANCE
}

/// Checks the current Windows version (see usage in `Win32State`)
unsafe fn is_windows_ver_or_greater(dl: &Win32DL, major: WORD, minor: WORD, sp_major: WORD) -> bool {
    let mut osvi: OSVERSIONINFOEXW = mem::zeroed();
    osvi.dwOSVersionInfoSize = mem::size_of_val(&osvi) as DWORD;
    osvi.dwMajorVersion = major.into();
    osvi.dwMinorVersion = minor.into();
    osvi.wServicePackMajor = sp_major;

    let mask = VER_MAJORVERSION | VER_MINORVERSION | VER_SERVICEPACKMAJOR;
    let mut cond = VerSetConditionMask(0, VER_MAJORVERSION, VER_GREATER_EQUAL);
    cond = VerSetConditionMask(cond, VER_MINORVERSION, VER_GREATER_EQUAL);
    cond = VerSetConditionMask(cond, VER_SERVICEPACKMAJOR, VER_GREATER_EQUAL);

    dl.RtlVerifyVersionInfo(&mut osvi, mask, cond) == Some(0)
}

/// Checks a specific Windows 10 update level (see usage in `Win32State`)
unsafe fn is_win10_ver_or_greater(dl: &Win32DL, build: WORD) -> bool {
    let mut osvi: OSVERSIONINFOEXW = mem::zeroed();
    osvi.dwOSVersionInfoSize = mem::size_of_val(&osvi) as DWORD;
    osvi.dwMajorVersion = 10;
    osvi.dwMinorVersion = 0;
    osvi.dwBuildNumber = build.into();

    let mask = VER_MAJORVERSION | VER_MINORVERSION | VER_BUILDNUMBER;
    let mut cond = VerSetConditionMask(0, VER_MAJORVERSION, VER_GREATER_EQUAL);
    cond = VerSetConditionMask(cond, VER_MINORVERSION, VER_GREATER_EQUAL);
    cond = VerSetConditionMask(cond, VER_BUILDNUMBER, VER_GREATER_EQUAL);

    dl.RtlVerifyVersionInfo(&mut osvi, mask, cond) == Some(0)
}

struct Win32State {
    /// Whether the system is at least on Windows 10 1607 (build 14393 - "Anniversary Update").
    at_least_anniversary_update: bool,

    /// The DPI mode that's enabled process-wide. The newest available is selected.
    /// MSDN recommends setting this with the manifest but that's rather unpleasant.
    /// Instead, it's set dynamically at runtime when this struct is instanced.
    dpi_mode: Win32DpiMode,

    /// Dynamically linked Win32 functions that might not be available on all systems.
    dl: Win32DL,
}

enum Win32DpiMode {
    Unsupported,
    System,
    PerMonitorV1,
    PerMonitorV2,
}

impl Win32State {
    fn new() -> Self {
        const VISTA_MAJ: WORD = (_WIN32_WINNT_VISTA >> 8) & 0xFF;
        const VISTA_MIN: WORD = _WIN32_WINNT_VISTA & 0xFF;
        const W81_MAJ: WORD = (_WIN32_WINNT_WINBLUE >> 8) & 0xFF;
        const W81_MIN: WORD = _WIN32_WINNT_WINBLUE & 0xFF;

        unsafe {
            let dl = Win32DL::link();

            let at_least_vista = is_windows_ver_or_greater(&dl, VISTA_MAJ, VISTA_MIN, 0);
            let at_least_8_point_1 = is_windows_ver_or_greater(&dl, W81_MAJ, W81_MIN, 0);
            let at_least_anniversary_update = is_win10_ver_or_greater(&dl, 14393);
            let at_least_creators_update = is_win10_ver_or_greater(&dl, 15063);

            let dpi_mode = if at_least_creators_update {
                let _ = dl.SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
                Win32DpiMode::PerMonitorV2
            } else if at_least_8_point_1 {
                let _ = dl.SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE);
                Win32DpiMode::PerMonitorV1
            } else if at_least_vista {
                let _ = dl.SetProcessDPIAware();
                Win32DpiMode::System
            } else {
                Win32DpiMode::Unsupported
            };

            Self {
                at_least_anniversary_update,
                dpi_mode,
                dl,
            }
        }
    }
}

/// Win32 functions need the full outer size for creation. This function calculates that size from an inner size.
///
/// Since for legacy reasons things like drop shadow are part of the bounds, don't use this for reporting outer size.
unsafe fn adjust_window_for_dpi(
    win32: &Win32State,
    size: Size,
    style: DWORD,
    style_ex: DWORD,
    dpi: UINT,
) -> (LONG, LONG) {
    let (width, height) = size.as_physical(dpi as f64 / BASE_DPI as f64);
    let mut window = RECT { left: 0, top: 0, right: width as LONG, bottom: height as LONG };
    if match win32.dpi_mode {
        // Non-client area DPI scaling is enabled in PMv1 Win10 1607+ and PMv2 (any).
        // For PMv1, this is done with EnableNonClientDpiScaling at WM_NCCREATE.
        Win32DpiMode::PerMonitorV1 if win32.at_least_anniversary_update => true,
        Win32DpiMode::PerMonitorV2 => true,
        _ => false,
    } {
        let _ = win32.dl.AdjustWindowRectExForDpi(&mut window, style, FALSE, style_ex, dpi);
    } else {
        // TODO: This *is* correct for old PMv1, right? How does broken NC scaling work?
        let _ = AdjustWindowRectEx(&mut window, style, FALSE, style_ex);
    }
    rect_to_size2d(&window)
}

fn cursor_to_int_resource(cursor: Cursor) -> *const WCHAR {
    match cursor {
        Cursor::Arrow => IDC_ARROW,
        Cursor::Blank => ptr::null(),
        Cursor::Cross => IDC_CROSS,
        Cursor::Hand => IDC_HAND,
        Cursor::Help => IDC_HELP,
        Cursor::IBeam => IDC_IBEAM,
        Cursor::Progress => IDC_APPSTARTING,
        Cursor::ResizeNESW => IDC_SIZENESW,
        Cursor::ResizeNS => IDC_SIZENS,
        Cursor::ResizeNWSE => IDC_SIZENWSE,
        Cursor::ResizeWE => IDC_SIZEWE,
        Cursor::ResizeAll => IDC_SIZEALL,
        Cursor::Unavailable => IDC_NO,
        Cursor::Wait => IDC_WAIT,
    }
}

#[inline]
fn rect_to_size2d(rect: &RECT) -> (LONG, LONG) {
    (rect.right - rect.left, rect.bottom - rect.top)
}

/// Gets this style as a bitfield. Note that it does not include the close button.
/// The close button is a menu property, not a window style.
fn style_as_win32(style: &window::Style) -> DWORD {
    let mut dword = 0;

    if style.borderless {
        // TODO: Why does this just not work without THICKFRAME? Borderless is dumb.
        dword |= WS_POPUP | WS_THICKFRAME;
    } else {
        dword |= WS_OVERLAPPED | WS_BORDER | WS_CAPTION;
    }

    if style.resizable {
        dword |= WS_THICKFRAME;
    }

    if style.visible {
        dword |= WS_VISIBLE;
    }

    if let Some(controls) = &style.controls {
        if controls.minimize {
            dword |= WS_MINIMIZEBOX;
        }
        if controls.maximize {
            dword |= WS_MAXIMIZEBOX;
        }
        dword |= WS_SYSMENU;
    }

    dword
}

/// Gets the extended window style bits.
fn style_as_win32_ex(style: &window::Style) -> DWORD {
    let mut dword = 0;

    if style.rtl_layout {
        dword |= WS_EX_LAYOUTRTL;
    }

    if style.tool_window {
        dword |= WS_EX_TOOLWINDOW;
    }

    dword
}

/// Due to legacy reasons, changing the window frame does nothing (since it's cached),
/// until you update it with SetWindowPos with just "oh yeah, the frame changed, that's about it".
#[inline]
unsafe fn ping_window_frame(hwnd: HWND) {
    const MASK: UINT = SWP_NOMOVE | SWP_NOSIZE | SWP_NOOWNERZORDER | SWP_NOZORDER | SWP_FRAMECHANGED;
    let _ = SetWindowPos(hwnd, ptr::null_mut(), 0, 0, 0, 0, MASK);
}

/// Convenience function to take a `window::Style` and slap it on a HWND.
fn update_window_style(hwnd: HWND, style: &window::Style) {
    let dword = style_as_win32(&style);
    let dword_ex = style_as_win32_ex(&style);
    unsafe {
        let _ = set_window_data(hwnd, GWL_STYLE, dword as usize);
        let _ = set_window_data(hwnd, GWL_EXSTYLE, dword_ex as usize);
        ping_window_frame(hwnd);
    }
}

/// Converts a &str to an LPCWSTR-compatible wide string.
///
/// If the length is 0 (aka `*retv == 0x00`) then no allocation was made (it points to a static NULL).
fn str_to_wstr(src: &str, buffer: &mut Vec<WCHAR>) -> *const WCHAR {
    // NOTE: Yes, indeed, `std::os::windows::OsStr(ing)ext` does exist in the standard library,
    // but it requires you to fit your data in the OsStr(ing) model and it's not hyper optimized
    // unlike mb2wc with handwritten SSE (allegedly), alongside being the native conversion function

    // MultiByteToWideChar can't actually handle 0 length because 0 return means error
    if src.is_empty() || src.len() > c_int::max_value() as usize {
        return [0x00].as_ptr()
    }

    unsafe {
        let str_ptr: *const CHAR = src.as_ptr().cast();
        let str_len = src.len() as c_int;

        // Calculate buffer size
        let req_buffer_size = MultiByteToWideChar(
            CP_UTF8, 0,
            str_ptr, str_len,
            ptr::null_mut(), 0, // `lpWideCharStr == NULL` means query size
        ) as usize + 1; // +1 for null terminator

        // Ensure buffer capacity
        buffer.clear();
        buffer.reserve(req_buffer_size);

        // Write to our buffer
        let chars_written = MultiByteToWideChar(
            CP_UTF8, 0,
            str_ptr, str_len,
            buffer.as_mut_ptr(), req_buffer_size as c_int,
        ) as usize;

        // Add null terminator & yield
        *buffer.as_mut_ptr().add(chars_written) = 0x00;
        buffer.set_len(req_buffer_size);
        buffer.as_ptr()
    }
}

/// Implementation container for `window::Window`
pub struct WindowImpl {
    hwnd: HWND,
    thread: Option<thread::JoinHandle<()>>,
    user: *mut WindowImplData, // 'thread
}

// Pointers automatically lose Send and Sync, so...
unsafe impl Send for WindowImpl {}
unsafe impl Sync for WindowImpl {}

/// Win32 specific extensions to the [`Window`](crate::window::Window) API.
pub trait WindowExt {
    /// Gets the Win32 window handle (a.k.a. [`HWND`]).
    fn hwnd(&self) -> HWND;
}

impl WindowExt for window::Window {
    #[inline]
    fn hwnd(&self) -> HWND {
        self.0.hwnd
    }
}

/// Win32 specific extensions to the [`WindowBuilder`](crate::window::WindowBuilder) API.
///
/// # Example
///
/// ```no_run
/// use ramen::platform::win32::WindowBuilderExt as _;
/// use ramen::window::Window;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let window = Window::builder()
///     .tool_window(true)
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub trait WindowBuilderExt {
    /// Sets whether the window uses the [`WS_EX_TOOLWINDOW`](
    /// https://docs.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles#WS_EX_TOOLWINDOW)
    /// style.
    ///
    /// This is equivalent to the .NET [`WindowStyle.ToolWindow`](
    /// https://docs.microsoft.com/en-us/dotnet/api/system.windows.windowstyle?view=net-5.0#System_Windows_WindowStyle_ToolWindow)
    /// property.
    ///
    /// From MSDN: *The window is intended to be used as a floating toolbar.*
    /// *A tool window has a title bar that is shorter than a normal title bar,*
    /// *and the window title is drawn using a smaller font.*
    /// *A tool window does not appear in the taskbar or in the dialog*
    /// *that appears when the user presses ALT+TAB.*
    fn tool_window(&mut self, tool_window: bool) -> &mut Self;
}

impl WindowBuilderExt for WindowBuilder {
    fn tool_window(&mut self, tool_window: bool) -> &mut Self {
        // TODO: Make this available for `Window` too
        self.style.tool_window = tool_window;
        self
    }
}

/// Info struct for `WM_(NC)CREATE`
pub struct WindowImplCreateParams {
    error: Option<Error>,
    user: *mut WindowImplData,
}

/// User data structure.
///
/// Mostly unsynchronized (and is therefore for the window thread only), handle with care.
pub struct WindowImplData {
    /// Current size of the client area (inner area)
    client_area_size: (u32, u32),

    /// Reason that `CloseRequest` was sent (consumed by `WM_CLOSE`)
    close_reason: Option<CloseReason>,

    /// The current (as in, where the window is) monitor DPI.
    current_dpi: UINT,

    /// The cursor sent to `WM_SETCURSOR`
    cursor: HCURSOR,

    /// Whether things should be scaled according to DPI.
    is_dpi_logical: bool,

    /// Indicates whether the window should be closing and destroying.
    /// TODO: explain
    destroy_flag: atomic::AtomicBool,

    /// The current window style which both `DWORD` styles can be built out of.
    style: window::Style,

    // Very lightweight event-swap system...
    // Read `Self::push_event` for more info
    ev_buf_sync: Mutex<bool>,
    ev_buf_ping: Condvar,
    ev_buf_is_primary: bool,
    ev_buf_primary: FixedVec<Event, MAX_EVENTS_PER_SWAP>,
    ev_buf_secondary: FixedVec<Event, MAX_EVENTS_PER_SWAP>,

    // State flag dump
    is_focused: bool,
    is_maximized: bool,
    is_minimized: bool,
}

pub fn spawn_window(builder: &WindowBuilder) -> Result<WindowImpl, Error> {
    let builder = builder.clone();

    // Convert class name & title to `WCHAR` string for Win32
    // This and the `recv` Arc are the only allocations, none in the thread
    let mut class_name_buf = Vec::new();
    let mut title_buf = Vec::new();
    let class_name = str_to_wstr(builder.class_name.as_ref(), &mut class_name_buf) as usize;
    let title = str_to_wstr(builder.title.as_ref(), &mut title_buf) as usize;
    let recv = Arc::new((Mutex::new(Option::<Result<WindowImpl, Error>>::None), Condvar::new()));
    let recv2 = Arc::clone(&recv); // remote thread's handle object

    let thread = thread::spawn(move || unsafe {
        // HACK: Since Rust doesn't trust us to share pointers, we move a `usize`
        // There are cleaner fixes for this, but this works just fine
        let class_name = class_name as *const WCHAR;
        let title = title as *const WCHAR;

        // Create the window class if it doesn't exist yet
        let mut class_created_this_thread = false;
        let class_registry_lock = sync::mutex_lock(CLASS_REGISTRY_LOCK.get());
        let mut class_info = mem::MaybeUninit::<WNDCLASSEXW>::uninit();
        (*class_info.as_mut_ptr()).cbSize = mem::size_of_val(&class_info) as DWORD;
        if GetClassInfoExW(this_hinstance(), class_name, class_info.as_mut_ptr()) == 0 {
            // The window class not existing sets the thread global error flag.
            SetLastError(ERROR_SUCCESS);

            // If this is the thread registering this window class,
            // it's the one responsible for setting class-specific data below
            class_created_this_thread = true;

            // Fill in & register class (`cbSize` is set before this if block)
            let class = &mut *class_info.as_mut_ptr();
            class.style = CS_OWNDC;
            class.lpfnWndProc = window_proc;
            class.cbClsExtra = mem::size_of::<usize>() as c_int;
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
            let _ = RegisterClassExW(class);
        }
        mem::drop(class_registry_lock);

        let dpi = BASE_DPI; // TODO:
        let style = style_as_win32(&builder.style);
        let style_ex = style_as_win32_ex(&builder.style);

        let (width, height) = adjust_window_for_dpi(WIN32.get(), builder.inner_size, style, style_ex, dpi);
        let (pos_x, pos_y) = (CW_USEDEFAULT, CW_USEDEFAULT);

        // Special
        let user_data: UnsafeCell<WindowImplData> = UnsafeCell::new(WindowImplData {
            client_area_size: builder.inner_size.as_physical(dpi as f64 / BASE_DPI as f64),
            close_reason: None, // unknown
            current_dpi: dpi,
            cursor: {
                let rsrc = cursor_to_int_resource(builder.cursor);
                if !rsrc.is_null() {
                    LoadImageW(ptr::null_mut(), rsrc, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE | LR_SHARED).cast()
                } else {
                    ptr::null_mut()
                }
            },
            is_dpi_logical: matches!(builder.inner_size, Size::Logical(..)),
            destroy_flag: atomic::AtomicBool::new(false),
            style: builder.style.clone(),

            ev_buf_sync: Mutex::new(false),
            ev_buf_ping: Condvar::new(),
            ev_buf_is_primary: true,
            ev_buf_primary: FixedVec::new(),
            ev_buf_secondary: FixedVec::new(),

            is_focused: false,
            is_maximized: false,
            is_minimized: false,
        });

        // A user pointer is supplied for `WM_NCCREATE` & `WM_CREATE` as lpParam
        let mut create_params = WindowImplCreateParams {
            error: None,
            user: user_data.get(),
        };
        let hwnd = CreateWindowExW(
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
            (&mut create_params) as *mut _ as *mut c_void,
        );

        if hwnd.is_null() && create_params.error.is_none() {
            // TODO: Push create failure
        }

        let (mutex, condvar) = &*recv2;
        let mut lock = sync::mutex_lock(&mutex);
        if let Some(err) = create_params.error.take() {
            *lock = Some(Err(err));
            return // early return (dropped by caller)
        } else {
            *lock = Some(Ok(WindowImpl {
                hwnd,
                thread: None, // filled in by caller
                user: user_data.get(),
            }));
        }
        sync::condvar_notify1(&condvar);
        mem::drop(lock);

        // No longer needed, free memory
        mem::drop(builder);
        mem::drop(recv2);

        // Set marker to identify our windows in HOOKPROC functions
        if class_created_this_thread {
            let _ = set_class_data(hwnd, 0, u32::from_le_bytes(*HOOKPROC_MARKER) as usize);
        }

        // Setup `HCBT_DESTROYWND` hook
        // TODO: explain this
        let thread_id = GetCurrentThreadId();
        let hhook = SetWindowsHookExW(WH_CBT, hcbt_destroywnd_hookproc, ptr::null_mut(), thread_id);

        // Run message loop until error or exit
        let mut msg = mem::MaybeUninit::zeroed().assume_init();
        'message_loop: loop {
            // `HWND hWnd` is set to NULL here to query all messages on the thread,
            // as the exit condition/signal `WM_QUIT` is not associated with any window.
            // This is one of the main motives (besides no blocking) to give each window a thread.
            match GetMessageW(&mut msg, ptr::null_mut(), 0, 0) {
                -1 => panic!("Hard error {:#06X} in GetMessageW loop!", GetLastError()),
                0 => if (*user_data.get()).destroy_flag.load(atomic::Ordering::Acquire) {
                    break 'message_loop
                },
                _ => {
                    // Dispatch message to `window_proc`
                    // NOTE: Some events call `window_proc` directly instead of through here
                    let _ = DispatchMessageW(&msg);
                },
            }
        }

        // Free `HCBT_DESTROYWND` hook (thread global)
        let _ = UnhookWindowsHookEx(hhook);

        // No need to unregister classes, that's done on exit
    });

    // Wait until the thread is done creating the window or notifying us why it couldn't do that
    let (mutex, condvar) = &*recv;
    let mut lock = sync::mutex_lock(&mutex);
    loop {
        if let Some(result) = (&mut *lock).take() {
            break result.map(|mut window| {
                window.thread = Some(thread);
                window
            })
        } else {
            sync::condvar_wait(&condvar, &mut lock);
        }
    }
}

impl WindowImpl {
    #[inline]
    pub fn execute<F, T>(&self, f: F) -> T
    where
        F: Send + FnOnce() -> T,
        T: Send,
    {
        let mut result = mem::MaybeUninit::<T>::uninit();

        // SAFETY: `SendMessageW` blocks until WindowProc has responded.
        let out_ptr = result.as_mut_ptr();
        let mut f = Some(Box::new(move || unsafe {
            *out_ptr = f();
        }) as Box<dyn FnOnce()>);

        unsafe {
            let _ = SendMessageW(self.hwnd, RAMEN_WM_EXECUTE, (&mut f) as *mut _ as WPARAM, 0);
            result.assume_init()
        }
    }

    pub fn events(&self) -> &[Event] {
        // SAFETY: The event buffer isn't swapped until `swap_events` is called (takes &mut self)

        // The backbuffer contains the "last" events, so use the opposite the active one
        let user_data = unsafe { &*self.user };
        if user_data.ev_buf_is_primary {
            user_data.ev_buf_secondary.slice()
        } else {
            user_data.ev_buf_primary.slice()
        }
    }

    #[inline]
    pub fn inner_size(&self) -> (Size, Scale) {
        let mut size = mem::MaybeUninit::<Size>::uninit();
        let mut scale = mem::MaybeUninit::<Scale>::uninit();
        unsafe {
            let _ = SendMessageW(
                self.hwnd,
                RAMEN_WM_GETINNERSIZE,
                size.as_mut_ptr() as WPARAM,
                scale.as_mut_ptr() as LPARAM,
            );
            (size.assume_init(), scale.assume_init())
        }
    }

    #[inline]
    pub fn is_dpi_logical(&self) -> bool {
        unsafe {
            SendMessageW(self.hwnd, RAMEN_WM_ISDPILOGICAL, 0, 0) != 0
        }
    }

    #[inline]
    pub fn set_controls(&self, controls: Option<window::Controls>) {
        let controls = controls.map(|c| c.to_bits()).unwrap_or(!0);
        unsafe {
            let _ = SendMessageW(self.hwnd, RAMEN_WM_SETCONTROLS, controls as WPARAM, 0);
        }
    }

    #[inline]
    pub fn set_cursor(&self, cursor: Cursor) {
        unsafe {
            let _ = SendMessageW(self.hwnd, RAMEN_WM_SETCURSOR, cursor as u32 as WPARAM, 0);
        }
    }

    #[inline]
    pub fn set_cursor_async(&self, cursor: Cursor) {
        unsafe {
            let _ = PostMessageW(self.hwnd, RAMEN_WM_SETCURSOR, cursor as u32 as WPARAM, 0);
        }
    }

    #[inline]
    pub fn set_controls_async(&self, controls: Option<window::Controls>) {
        let controls = controls.map(|c| c.to_bits()).unwrap_or(!0);
        unsafe {
            let _ = PostMessageW(self.hwnd, RAMEN_WM_SETCONTROLS, controls as WPARAM, 0);
        }
    }

    #[inline]
    pub fn set_inner_size(&self, size: Size) {
        unsafe {
            let _ = SendMessageW(self.hwnd, RAMEN_WM_SETINNERSIZE, 0, (&size) as *const Size as LPARAM);
        }
    }

    #[inline]
    pub fn set_maximized(&self, maximized: bool) {
        unsafe {
            let _ = SendMessageW(self.hwnd, RAMEN_WM_SETMAXIMIZED, maximized as WPARAM, 0);
        }
    }

    #[inline]
    pub fn set_maximized_async(&self, maximized: bool) {
        unsafe {
            let _ = PostMessageW(self.hwnd, RAMEN_WM_SETMAXIMIZED, maximized as WPARAM, 0);
        }
    }

    #[inline]
    pub fn set_resizable(&self, resizable: bool) {
        unsafe {
            let _ = SendMessageW(self.hwnd, RAMEN_WM_SETTHICKFRAME, resizable as WPARAM, 0);
        }
    }

    #[inline]
    pub fn set_resizable_async(&self, resizable: bool) {
        unsafe {
            let _ = PostMessageW(self.hwnd, RAMEN_WM_SETTHICKFRAME, resizable as WPARAM, 0);
        }
    }

    #[inline]
    pub fn set_title(&self, title: &str) {
        let mut wstr = Vec::new();
        let lpcwstr = str_to_wstr(title, &mut wstr);
        unsafe {
            // TODO: explicit pass on settext in windowproc
            let _ = SendMessageW(self.hwnd, WM_SETTEXT, 0, lpcwstr as LPARAM);
        }
    }

    pub fn set_title_async(&self, title: &str) {
        // Win32 has special behaviour on WM_SETTEXT, since it takes a pointer to a buffer.
        // You can't actually call it asynchronously, *in case* it's being sent from a different process.
        // Only if they had Rust back then, this poorly documented stupid detail would not exist,
        // as trying to use PostMessageW with WM_SETTEXT silently fails because it's scared of lifetimes.
        // As a workaround, we just define our own event, WM_SETTEXT_ASYNC, and still support WM_SETTEXT.
        // This is far better than using the "unused parameter" in WM_SETTEXT anyways.
        // More info on this: https://devblogs.microsoft.com/oldnewthing/20110916-00/?p=9623
        let mut wstr: Vec<WCHAR> = Vec::new();
        let lpcwstr = str_to_wstr(title, &mut wstr);
        unsafe {
            // TODO: okay maybe do filter null in str_to_wstr..ugh
            if *lpcwstr == 0x00 {
                // If the string is empty, nothing is allocated and nothing needs to be passed
                // lParam == NULL indicates that it should be empty
                let _ = PostMessageW(self.hwnd, RAMEN_WM_SETTEXT_ASYNC, 0, 0);
            } else {
                let _ = PostMessageW(
                    self.hwnd,
                    RAMEN_WM_SETTEXT_ASYNC,
                    wstr.len() as WPARAM,
                    lpcwstr as LPARAM,
                );
                mem::forget(wstr); // "leak" the memory as `window_proc` will clean it up
            }
        }
    }

    #[inline]
    pub fn set_visible(&self, visible: bool) {
        unsafe {
            let _ = ShowWindow(self.hwnd, if visible { SW_SHOW } else { SW_HIDE });
        }
    }

    #[inline]
    pub fn set_visible_async(&self, visible: bool) {
        unsafe {
            // They provide a function to do this asynchonously, how handy!
            // The difference being that it does `PostMessage`, not `SendMessage`.
            // It's implemented as `WM_SHOWWINDOW` and is handled by `DefWindowProcW` (or you).
            let _ = ShowWindowAsync(self.hwnd, if visible { SW_SHOW } else { SW_HIDE });
        }
    }

    pub fn swap_events(&mut self) {
        let user_data = unsafe { &mut *self.user };
        let mut lock = sync::mutex_lock(&user_data.ev_buf_sync);

        // clear backbuffer, switch to it
        if user_data.ev_buf_is_primary {
            user_data.ev_buf_secondary.clear();
        } else {
            user_data.ev_buf_primary.clear();
        }
        user_data.ev_buf_is_primary = !user_data.ev_buf_is_primary;

        // deal with potential lockup (see `WindowImplData::push_event`)
        if *lock {
            *lock = false; // "the request to ping the condvar is processed"
            sync::condvar_notify1(&user_data.ev_buf_ping);
        }

        mem::drop(lock);
    }
}

impl WindowImplData {
    #[inline]
    pub fn push_event(&mut self, event: Event) {
        self.push_events(&[event]);
    }

    pub fn push_events(&mut self, events: &[Event]) {
        // If the window thread locks up, the window should too, eventually.
        // This quirk of the event swap system stores a "is cvar waiting" in the mutex,
        // making it so that if swap never occurs, this eventually will indefinitely block,
        // and thus "Not Responding" will occur and a crash will snowball into the window too.
        // This is similar to what Win32 does with its event buffer.
        let mut lock = sync::mutex_lock(&self.ev_buf_sync);
        loop {
            let ev_buf = if self.ev_buf_is_primary {
                &mut self.ev_buf_primary
            } else {
                &mut self.ev_buf_secondary
            };
            if *lock || !ev_buf.push_many(events) {
                *lock = true; // "the condvar should be pinged"
                sync::condvar_wait(&self.ev_buf_ping, &mut lock);
            } else {
                break
            }
        }
    }
}

/// Due to legacy reasons, the close button is a system menu item and not a window style.
/// This function is for turning it on and off (enabled and disabled, rather).
unsafe fn set_close_button(hwnd: HWND, enabled: bool) {
    let menu: HMENU = GetSystemMenu(hwnd, FALSE);
    let flag = if enabled {
        MF_BYCOMMAND | MF_ENABLED
    } else {
        MF_BYCOMMAND | MF_DISABLED | MF_GRAYED
    };
    let _ = EnableMenuItem(menu, SC_CLOSE as UINT, flag);
}

/// Client area -> Screen space
unsafe fn client_area_screen_space(hwnd: HWND) -> RECT {
    let mut client_area: RECT = mem::zeroed();
    let _ = GetClientRect(hwnd, &mut client_area);
    let _ = ClientToScreen(hwnd, &mut client_area.left as *mut _ as *mut POINT);
    let _ = ClientToScreen(hwnd, &mut client_area.right as *mut _ as *mut POINT);
    client_area
}

#[inline]
unsafe fn user_data<'a>(hwnd: HWND) -> &'a mut WindowImplData {
    &mut *(get_window_data(hwnd, GWL_USERDATA) as *mut WindowImplData)
}

/// See the place it's used in `spawn_window` for an explanation
unsafe extern "system" fn hcbt_destroywnd_hookproc(code: c_int, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HCBT_DESTROYWND {
        let hwnd = wparam as HWND;
        if get_class_data(hwnd, GCL_CBCLSEXTRA) == mem::size_of::<usize>()
            && (get_class_data(hwnd, 0) as u32).to_le_bytes() == *HOOKPROC_MARKER
        {
            // Note that nothing is forwarded here, we decide for our windows
            if user_data(hwnd).destroy_flag.load(atomic::Ordering::Acquire) {
                0 // Allow
            } else {
                1 // Prevent
            }
        } else {
            // Unrelated window, forward
            CallNextHookEx(ptr::null_mut(), code, wparam, lparam)
        }
    } else {
        // Unrelated event, forward
        CallNextHookEx(ptr::null_mut(), code, wparam, lparam)
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    macro_rules! mouse_event {
        ($v:ident , $b:ident) => {{
            #[cfg(feature = "input")] {
                user_data(hwnd).push_event(Event::$v(MouseButton::$b));
            }
            0
        }};
    }

    // Fantastic resource for a list of window messages:
    // https://wiki.winehq.org/List_Of_Windows_Messages
    match msg {
        // No-op event, used for pinging the event loop, stubbing, etc. Return 0.
        WM_NULL => 0,

        // Received when the client area of the window is about to be created.
        // This event is completed *before* `CreateWindowExW` returns, but *after* `WM_NCCREATE`.
        // wParam: Unused, should be ignored.
        // lParam: `CREATESTRUCTW *` (for inspecting, not for writing)
        // Return 0 to continue creation or -1 to destroy and return NULL from `CreateWindowExW`.
        // See also: `WM_NCCREATE`
        WM_CREATE => {
            // `lpCreateParams` is the first field, so `CREATESTRUCTW *` is `WindowImplCreateParams **`
            let _params = &mut **(lparam as *const *mut WindowImplCreateParams);

            // ...

            0 // OK
        },

        // Received when the client area is being destroyed.
        // This is sent by `DestroyWindow`, then `WM_NCDESTROY` is sent, then the window is gone.
        // Nothing can actually be done once this message is received, and you always return 0.
        WM_DESTROY => {
            // Make sure it was received from the window being dropped, and not manually sent.
            if user_data(hwnd).destroy_flag.load(atomic::Ordering::Acquire) {
                // Send `WM_QUIT` with exit code 0
                PostQuitMessage(0);
            }
            0
        },

        // TODO: ...
        WM_MOVE => {
            // TODO: ...
            DefWindowProcW(hwnd, msg, wparam, lparam)
        },

        // [ Event 0x0004 is not known to exist ]

        // Received then the size has changed.
        // wParam: Indicates the reason.
        // lParam: LOWORD=width, HIWORD=height (of client area)
        WM_SIZE => {
            let user_data = user_data(hwnd);
            let mut events = FixedVec::<Event, 3>::new();

            fn set_max_min(user_data: &mut WindowImplData, buf: &mut FixedVec<Event, 3>, max: bool, min: bool) {
                if user_data.is_maximized != max {
                    user_data.is_maximized = max;
                    let _ = buf.push(&Event::Maximize(max));
                }
                if user_data.is_minimized != min {
                    user_data.is_minimized = min;
                    let _ = buf.push(&Event::Minimize(min));
                }
            }

            match wparam {
                SIZE_RESTORED => set_max_min(user_data, &mut events, false, false),
                SIZE_MINIMIZED => set_max_min(user_data, &mut events, false, true),
                SIZE_MAXIMIZED => set_max_min(user_data, &mut events, true, false),
                _ => (), // rest are for pop-up (`WS_POPUP`) windows
            }

            // Minimize events give us a confusing new client size of (0, 0) so we ignore that
            if wparam != SIZE_MINIMIZED {
                // let lhword = ((lparam & 0xFFFF) as WORD as u32, ((lparam >> 16) & 0xFFFF) as WORD as u32);
                // if user_data.client_area_size != lhword {
                //     let (loword, hiword) = lhword;
                //     let inner_size = Size::Physical(loword as u32, hiword as u32);
                //     let dpi_scale = user_data.current_dpi as f64 / BASE_DPI as f64;
                //     let event = if user_data.is_dpi_logical {
                //         Event::Resize((inner_size.to_logical(dpi_scale), dpi_scale))
                //     } else {
                //         Event::Resize((inner_size, dpi_scale))
                //     };
                //     user_data.client_area_size = lhword;
                //     let _ = events.push(&event);
                // }
            }
            user_data.push_events(events.slice());

            0
        },

        // Received when the window is activated or deactivated (focus gain/loss). Return 0.
        // wParam: HIWORD = non-zero if minimized, LOWORD = WA_ACTIVE | WA_CLICKACTIVE | WA_INACTIVE
        // lParam: HWND to window being deactivated (if ACTIVE|CLICKATIVE) otherwise the activated one
        // See also: `WM_ACTIVATEAPP` and `WM_SETFOCUS` & `WM_KILLFOCUS`
        WM_ACTIVATE => {
            // Quoting MSDN:
            // "The high-order word specifies the minimized state of the window being activated
            // or deactivated. A nonzero value indicates the window is minimized."
            //
            // This doesn't work entirely correctly in all situations, as with most of Win32,
            // so if we don't do some logic here we get two events on unfocusing
            // by clicking on the taskbar icon for example, among other things:
            // 1) WM_INACTIVE (HIWORD == 0)
            // 2) WM_ACTIVATE (HIWORD != 0)
            // Note that #2 translates to active(focused) & minimized simultaneously.
            // This would mean the window would be told it's focused after being minimized. Fantastic.

            // These problems could be avoided like this:
            // match (loword, hiword) {
            //     (true, true) => return 0,
            //     (state, _) => {
            //         if user_data.is_focused != state {
            //             user_data.is_focused = state;
            //             user_data.push_event(Event::Focus(state));
            //         }
            //     },
            // }
            // However, that's a waste of time when you can just process `WM_SETFOCUS` and `WM_KILLFOCUS`.

            0
        },

        // Received when a Win32 window receives keyboard focus. Return 0.
        // This is mainly intended for textbox controls but works perfectly fine for actual windows.
        // See also: `WM_ACTIVATE` (to know why this is used for focus events)
        WM_SETFOCUS => {
            let user_data = user_data(hwnd);

            if !user_data.is_focused {
                user_data.is_focused = true;
                user_data.push_event(Event::Focus(true));
            }

            // TODO: Cursor lock nonsense

            0
        },

        // Received when a Win32 window loses keyboard focus. Return 0.
        // See also: `WM_SETFOCUS` and `WM_ACTIVATE`
        WM_KILLFOCUS => {
            let user_data = user_data(hwnd);
            if user_data.is_focused {
                user_data.is_focused = false;
                user_data.push_event(Event::Focus(false));
            }
            0
        },

        // [ Event 0x0009 is not known to exist ]

        // Received when the enable state of the window has been changed.
        // wParam: TRUE if enabled, FALSE if disabled.
        // lParam: Unused, ignore value.
        // Return 0.
        WM_ENABLE => {
            // TODO: Check if re-enabling is needed? Can it just be prevented?
            0
        },

        // Received to update whether the controls should be redrawn.
        // To us this is useless, as we don't use Win32 control drawing, so we ignore it.
        // For more info: https://devblogs.microsoft.com/oldnewthing/20140407-00/?p=1313
        // Return 0.
        WM_SETREDRAW => 0,

        // Received when a system function says we should repaint some of the window.
        // Since we don't care about Win32, we just ignore this. Return 0.
        WM_PAINT => 0,

        // Received when a window is requested to close.
        // wParam & lParam are unused. Return 0.
        WM_CLOSE => {
            let user_data = user_data(hwnd);
            let reason = user_data.close_reason.take().unwrap_or(CloseReason::Unknown);
            user_data.push_event(Event::CloseRequest(reason));
            0
        },

        // TODO: WM_QUERYENDSESSION shenanigans

        // ...

        // Received when the background should be erased.
        // Similarly to `WM_PAINT`, we don't care, as we do our own drawing.
        // wParam: Device context handle (HDC)
        // lParam: Unused, should be ignored.
        // Return non-zero on erase.
        WM_ERASEBKGND => TRUE as LRESULT,

        // Received when the window is about to be shown or hidden.
        // wParam: `TRUE` if shown, `FALSE` if hidden.
        // lParam: The reason this message is sent (see MSDN).
        // Return 0.
        WM_SHOWWINDOW => {
            // If `lparam == 0`, this was received from `ShowWindow` or `ShowWindowAsync`
            if lparam == 0 {
                // If this isn't updated here, the next style change will re-hide the window.
                // It won't redraw either so it'll be stale on the screen until interacted with.
                user_data(hwnd).style.visible = wparam != 0;
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        },

        // Supposedly `WM_ACTIVATE`, but only received if the focus is to a different application.
        // This doesn't seem to be actually true, and it even has the same bugs as `WM_ACTIVATE`.
        // For this reason (and being useless and confusing) it should be ignored. Return 0.
        // See also: `WM_ACTIVATE`
        WM_ACTIVATEAPP => 0,


        // Received when the cursor should be set.
        // wParam: `HWND` which has the cursor in it.
        // lParam: LOWORD is a hit-test, HIWORD indicates sender (see MSDN).
        // Return TRUE if processed or FALSE to continue asking child windows.
        WM_SETCURSOR => {
            // We handle it in the client area, otherwise it's not our business.
            if (hwnd == wparam as HWND) && ((lparam & 0xFFFF) as WORD == HTCLIENT as WORD) {
                let _ = SetCursor(user_data(hwnd).cursor);
                TRUE as LRESULT
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        },

        WM_WINDOWPOSCHANGING => {
            let param = &*(lparam as *const WINDOWPOS);
            if (param.flags & SWP_NOSIZE) == 0 && param.cx != 0 && param.cy != 0 {
                let user_data = user_data(hwnd);
                let dpi_scale = user_data.current_dpi as f64 / BASE_DPI as f64;
                let (width_adj, height_adj) = adjust_window_for_dpi(
                    WIN32.get(),
                    Size::Physical(0, 0),
                    style_as_win32(&user_data.style),
                    style_as_win32_ex(&user_data.style),
                    user_data.current_dpi,
                );
                let new_w = param.cx - width_adj;
                let new_h = param.cy - height_adj;
                if new_w > 0 && new_h > 0 {
                    user_data.push_event(Event::Resize((
                        Size::Physical(
                            new_w as u32,
                            new_h as u32,
                        ),
                        dpi_scale,
                    )));
                    user_data.client_area_size.0 = new_w as u32;
                    user_data.client_area_size.1 = new_h as u32;
                }
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        },

        // Received when a key is pressed or released.
        // wParam: Virtual key code
        // lParam: Giant bitfield. Please just read MSDN (hint: bit 31 means MSB, 0 is LSB)
        // Return 0.
        WM_KEYDOWN | WM_KEYUP => {
            #[cfg(feature = "input")]
            if let Some(key) = translate_vk(wparam) {
                user_data(hwnd).push_event(map_tr_state(extend_key(key, lparam), lparam));
            }
            0
        },

        // Same as `WM_KEYDOWN` & `WM_KEYUP` but with a few (horrific) bitfield quirks.
        WM_SYSKEYDOWN | WM_SYSKEYUP => {
            let mut user_data = user_data(hwnd);

            // As a side-effect of handling "system keys", we actually override Alt+F4.
            // It's re-implemented here, because it's usually expected that Alt+F4 does something.
            if wparam & 0xFF == VK_F4 as WPARAM && lparam & (1 << 29) != 0 {
                user_data.close_reason = Some(CloseReason::KeyboardShortcut);
                let _ = SendMessageW(hwnd, WM_CLOSE, 0, 0);
            }

            // This is one of the worst parts of the Win32 input event system.
            // Countless games have bugs and exploits relating to the Alt & F10 keys.

            // To sum it up, the Alt and F10 keys are very special due to historical reasons.
            // They have their own event if in combination with other keys.
            // Making it worse, "it also occurs when no window currently has the keyboard focus",
            // although this isn't actually observable on new OSes so it's just a legacy feature.

            // Bit 29 in lParam is set if the Alt key is down while emitting this message,
            // which sounds like a reasonable fix to tell apart the two reasons for this message.
            // Except if it's the Alt key being released, it won't be set! So you must trust wParam.
            // F10 doesn't even set any bit because there's no F10 bit, so you trust that one too.
            #[cfg(feature = "input")]
            if let Some(event) = sys_key_event(wparam, lparam) {
                user_data.push_event(event);
            }

            0
        },

        // Received when the non-client area of the window is about to be created.
        // This is *before* `WM_CREATE` and is basically the first event received.
        // wParam: Unused, should be ignored.
        // lParam: `CREATESTRUCTW *` (for inspecting, not for writing)
        // Return `TRUE` to continue creation or `FALSE` to abort and return NULL from `CreateWindowExW`.
        // See also: `WM_CREATE`
        WM_NCCREATE => {
            // `lpCreateParams` is the first field, so `CREATESTRUCTW *` is `WindowImplCreateParams **`
            let params = &mut **(lparam as *const *mut WindowImplCreateParams);

            // Store user data pointer
            let _ = set_window_data(hwnd, GWL_USERDATA, params.user as usize);

            // Enable the non-client area scaling patch for PMv1 if available
            let win32 = WIN32.get();
            if matches!(win32.dpi_mode, Win32DpiMode::PerMonitorV1) && win32.at_least_anniversary_update {
                let _ = win32.dl.EnableNonClientDpiScaling(hwnd);
            }

            // This is where some things like the title contents are stored,
            // so make sure to forward `WM_NCCREATE` to DefWindowProcW
            DefWindowProcW(hwnd, msg, wparam, lparam)
        },

        // Received when the user clicks a window menu control (formerly "system menu").
        // wParam: Command enum.
        // lParam: Mouse position (screen coords, word+word) or accelerator flags in hiword.
        // Return 0.
        WM_SYSCOMMAND => {
            if wparam == SC_CLOSE {
                user_data(hwnd).close_reason = Some(CloseReason::SystemMenu);
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        },

        // Received then the mouse cursor has moved around in the client area.
        // wParam: Big bitfield of which virtual keys are down (see MSDN).
        // lParam: loword = signed X coordinate, hiword = signed Y coordinate
        // Return 0.
        WM_MOUSEMOVE => {
            #[cfg(feature = "input")]
            {
                let user_data = user_data(hwnd);

                // TODO: Relative coordinates, mouse warp, touchscreen etc. God damn.
                let (cw, ch) = user_data.client_area_size;
                let (x, y) = (
                    (lparam & 0xFFFF) as c_short,
                    ((lparam >> 16) & 0xFFFF) as c_short,
                );

                // On some versions of windows, the border padding reports `WM_MOUSEMOVE`!
                // It's not even client area, but it does, and only when `WS_THICKFRAME` is unset?!
                // So if SM_CXBORDER is 1 and SM_CXBORDERPADDING is 4 you'd get -5 <= x <= width+5!
                if x >= 0 && (x as u32) < cw && y >= 0 && (y as u32) < ch {
                    let point = Point::Physical(x as u32, y as u32);
                    let dpi_scale = user_data.current_dpi as f64 / BASE_DPI as f64;
                    let event = if user_data.is_dpi_logical {
                        Event::MouseMove((point.to_logical(dpi_scale), dpi_scale))
                    } else {
                        Event::MouseMove((point, dpi_scale))
                    };
                    user_data.push_event(event);
                }
            }

            0
        },

        // Received when the mouse buttons are down/up. Return 0.
        // wParam indicates what other buttons are down.
        // lParam contains the X and Y coordinate.
        WM_LBUTTONDOWN => mouse_event!(MouseDown, Left),
        WM_LBUTTONUP => mouse_event!(MouseUp, Left),
        WM_RBUTTONDOWN => mouse_event!(MouseDown, Right),
        WM_RBUTTONUP => mouse_event!(MouseUp, Right),
        WM_MBUTTONDOWN => mouse_event!(MouseDown, Middle),
        WM_MBUTTONUP => mouse_event!(MouseUp, Middle),
        _ev @ WM_XBUTTONDOWN | _ev @ WM_XBUTTONUP => {
            #[cfg(feature = "input")]
            {
                // For X buttons, the HIWORD in wParam indicates which X button it is.
                let user_data = user_data(hwnd);
                let event = if _ev == WM_XBUTTONDOWN {
                    Event::MouseDown
                } else {
                    Event::MouseUp
                };
                match ((wparam >> 16) & 0xFFFF) as WORD {
                    XBUTTON1 => user_data.push_event(event(MouseButton::Mouse4)),
                    XBUTTON2 => user_data.push_event(event(MouseButton::Mouse5)),
                    _ => (),
                }
            }
            0
        },

        // Received when the mouse wheel is rotated.
        // wParam: HIWORD=delta in WHEEL_DELTA(120) multiples, LOWORD=vk state (see msdn)
        // lParam: HIWORD=mouse x, LOWORD=mouse y
        // Return 0.
        WM_MOUSEWHEEL => {
            #[cfg(feature = "input")]
            {
                let delta = ((wparam >> 16) & 0xFFFF) as c_short;
                if delta != 0 {
                    user_data(hwnd).push_event(Event::MouseWheel(NonZeroI32::new_unchecked(delta.into())));
                }
            }
            0
        },

        // Custom message: The "real" destroy signal that won't be rejected.
        // TODO: document the rejection emchanism somewhere
        // Return 0.
        RAMEN_WM_DROP => {
            user_data(hwnd).destroy_flag.store(true, atomic::Ordering::Release);
            let _ = DestroyWindow(hwnd);
            0
        },


        // Custom message: Execute a closure inside the window thread.
        // wParam: `*mut Option<Box<dyn FnOnce()>>`
        // lParam: Unused, set to zero.
        // Return 0.
        RAMEN_WM_EXECUTE => {
            // `FnOnce` requires the closure to consume itself, so it's done like this!
            let option = &mut *(wparam as *mut Option<Box<dyn FnOnce()>>);
            if let Some(f) = option.take() {
                f();
            }
            0
        },

        // Custom event: Update window controls.
        // wParam: If anything but !0 (~0 in C terms), window controls bits, else None.
        // lParam: Unused, set to zero.
        RAMEN_WM_SETCONTROLS => {
            let mut user_data = user_data(hwnd);
            let controls = {
                let bits = wparam as u32;
                if bits != !0 {
                    Some(window::Controls::from_bits(bits))
                } else {
                    None
                }
            };
            if user_data.style.controls != controls {
                user_data.style.controls = controls;

                // Update system menu's close button if present
                if let Some(close) = user_data.style.controls.as_ref().map(|c| c.close) {
                    set_close_button(hwnd, close);
                }

                // Set styles, refresh
                update_window_style(hwnd, &user_data.style);
            }
            0
        },

        // Custom event: Set the window cursor that's sent to `WM_SETCURSOR`.
        // wParam: `Cursor as u32`
        // lParam: Unused, set to zero.
        // Return 0.
        RAMEN_WM_SETCURSOR => {
            let user_data = user_data(hwnd);
            let cursor = mem::transmute::<_, Cursor>(wparam as u32);
            let rsrc = cursor_to_int_resource(cursor);

            // `LoadImageW` is not only superseding `LoadCursorW` but it's ~20µs faster. Wow, use this!
            user_data.cursor = if !rsrc.is_null() {
                LoadImageW(ptr::null_mut(), rsrc, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE | LR_SHARED).cast()
            } else {
                ptr::null_mut()
            };

            // Immediately update the cursor icon if it's within the client area.
            let mut mouse_pos: POINT = mem::zeroed();
            if GetCursorPos(&mut mouse_pos) != 0 && WindowFromPoint(POINT { ..mouse_pos }) == hwnd {
                let client_area = client_area_screen_space(hwnd);
                if PtInRect(&client_area, mouse_pos) != 0 {
                    let _ = SetCursor(user_data.cursor);
                }
            }
            0
        },

        // Custom event: Set the title asynchronously.
        // wParam: Buffer length, if lParam != NULL.
        // lParam: Vec<WCHAR> pointer or NULL for empty.
        RAMEN_WM_SETTEXT_ASYNC => {
            if lparam != 0 {
                let wstr = Vec::from_raw_parts(lparam as *mut WCHAR, wparam as usize, wparam as usize);
                let _ = DefWindowProcW(hwnd, WM_SETTEXT, 0, wstr.as_ptr() as LPARAM);
                mem::drop(wstr); // managed by callee, caller should `mem::forget`
            } else {
                let _ = DefWindowProcW(hwnd, WM_SETTEXT, 0, [WCHAR::default()].as_ptr() as LPARAM);
            }
            0
        },

        // Custom event: Set whether the window is resizable.
        // wParam: If non-zero, resizable, otherwise not resizable.
        // lParam: Unused, set to zero.
        RAMEN_WM_SETTHICKFRAME => {
            let mut user_data = user_data(hwnd);
            let resizable = wparam != 0;
            if user_data.style.resizable != resizable {
                user_data.style.resizable = resizable;
                update_window_style(hwnd, &user_data.style);
            }
            0
        },

        // Custom event: Set the inner size.
        // wParam: Unused, set to zero.
        // lParam: `*const Size`
        RAMEN_WM_SETINNERSIZE => {
            let inner_size = &*(lparam as *const Size);
            let user_data = user_data(hwnd);

            user_data.client_area_size = inner_size.as_physical(user_data.current_dpi as f64 / BASE_DPI as f64);
            user_data.is_dpi_logical = matches!(inner_size, Size::Logical(..));
            let (owidth, oheight) = adjust_window_for_dpi(
                WIN32.get(),
                *inner_size,
                style_as_win32(&user_data.style),
                style_as_win32_ex(&user_data.style),
                user_data.current_dpi,
            );

            const MASK: UINT = SWP_NOMOVE | SWP_NOOWNERZORDER | SWP_NOZORDER;
            let _ = SetWindowPos(hwnd, ptr::null_mut(), 0, 0, owidth, oheight, MASK);

            0
        },

        // Custom event: Query the inner size.
        // wParam: `*mut Size` (out)
        // lParam: `*mut Scale` (out)
        RAMEN_WM_GETINNERSIZE => {
            let user_data = user_data(hwnd);
            let out_size = wparam as *mut Size;
            let out_scale = lparam as *mut Scale;

            let dpi_factor = user_data.current_dpi as f64 / BASE_DPI as f64;
            let (width, height) = user_data.client_area_size;
            let inner_size = Size::Physical(width, height);

            if user_data.is_dpi_logical {
                *out_size = inner_size.to_logical(dpi_factor);
            } else {
                *out_size = inner_size;
            }
            *out_scale = dpi_factor;

            0
        },

        // Custom event: Query whether we're in logical DPI mode. Niche thing.
        // wParam & lParam: Unused.
        // Non-zero return if logical.
        RAMEN_WM_ISDPILOGICAL => {
            user_data(hwnd).is_dpi_logical as LPARAM
        },

        RAMEN_WM_SETMAXIMIZED => {
            let user_data = user_data(hwnd);
            let maximized = wparam != 0;
            if user_data.is_maximized != maximized {
                let button = if maximized { SC_MAXIMIZE } else { SC_RESTORE };
                let _ = DefWindowProcW(hwnd, WM_SYSCOMMAND, button, 0);
                user_data.is_maximized = maximized;
                user_data.push_event(Event::Maximize(maximized));
            }
            0
        },

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

impl ops::Drop for WindowImpl {
    fn drop(&mut self) {
        // Signal the window it's OK to close, and wait for the thread to naturally return
        unsafe {
            let _ = PostMessageW(self.hwnd, RAMEN_WM_DROP, 0, 0);
        }
        let _ = self.thread.take().map(thread::JoinHandle::join);
    }
}

#[cfg(feature = "input")]
fn translate_vk(wparam: WPARAM) -> Option<Key> {
    match (wparam & 0xFF) as u8 {
        // Undocumented.
        0x00 => None,

        // These are not keys.
        VK_LBUTTON | VK_RBUTTON | VK_CANCEL | VK_MBUTTON | VK_XBUTTON1 | VK_XBUTTON2 => None,

        // Undefined.
        0x07 => None,

        VK_BACK => Some(Key::Backspace),
        VK_TAB => Some(Key::Tab),

        // Reserved.
        0x0A..=0x0B => None,

        VK_CLEAR => Some(Key::Clear),
        VK_RETURN => Some(Key::Enter),

        // Undefined.
        0x0E..=0x0F => None,

        VK_SHIFT => Some(Key::LShift),
        VK_CONTROL => Some(Key::LControl),
        VK_MENU => Some(Key::LAlt),
        VK_PAUSE => Some(Key::Pause),
        VK_CAPITAL => Some(Key::CapsLock),
        VK_KANA => Some(Key::ImeKana),
        VK_IME_ON => Some(Key::ImeOn),
        VK_JUNJA => Some(Key::ImeJunja),
        VK_FINAL => Some(Key::ImeFinal),
        VK_KANJI => Some(Key::ImeKanji),
        VK_IME_OFF => Some(Key::ImeOff),
        VK_ESCAPE => Some(Key::Escape),
        VK_CONVERT => Some(Key::ImeConvert),
        VK_NONCONVERT => Some(Key::ImeNonConvert),
        VK_ACCEPT => Some(Key::ImeAccept),
        VK_MODECHANGE => Some(Key::ImeModeChange),
        VK_SPACE => Some(Key::Space),
        VK_PRIOR => Some(Key::PageUp),
        VK_NEXT => Some(Key::PageDown),
        VK_END => Some(Key::End),
        VK_HOME => Some(Key::Home),
        VK_LEFT => Some(Key::Left),
        VK_UP => Some(Key::Up),
        VK_RIGHT => Some(Key::Right),
        VK_DOWN => Some(Key::Down),
        VK_SELECT => Some(Key::Select),
        VK_PRINT => Some(Key::Print),
        VK_EXECUTE => Some(Key::Execute),
        VK_SNAPSHOT => Some(Key::PrintScreen), // this one's going in my cringe compilation
        VK_INSERT => Some(Key::Insert),
        VK_DELETE => Some(Key::Delete),
        VK_HELP => Some(Key::Help),

        0x30 => Some(Key::Num0),
        0x31 => Some(Key::Num1),
        0x32 => Some(Key::Num2),
        0x33 => Some(Key::Num3),
        0x34 => Some(Key::Num4),
        0x35 => Some(Key::Num5),
        0x36 => Some(Key::Num6),
        0x37 => Some(Key::Num7),
        0x38 => Some(Key::Num8),
        0x39 => Some(Key::Num9),

        // Undefined.
        0x3A..=0x40 => None,

        0x41 => Some(Key::A),
        0x42 => Some(Key::B),
        0x43 => Some(Key::C),
        0x44 => Some(Key::D),
        0x45 => Some(Key::E),
        0x46 => Some(Key::F),
        0x47 => Some(Key::G),
        0x48 => Some(Key::H),
        0x49 => Some(Key::I),
        0x4A => Some(Key::J),
        0x4B => Some(Key::K),
        0x4C => Some(Key::L),
        0x4D => Some(Key::M),
        0x4E => Some(Key::N),
        0x4F => Some(Key::O),
        0x50 => Some(Key::P),
        0x51 => Some(Key::Q),
        0x52 => Some(Key::R),
        0x53 => Some(Key::S),
        0x54 => Some(Key::T),
        0x55 => Some(Key::U),
        0x56 => Some(Key::V),
        0x57 => Some(Key::W),
        0x58 => Some(Key::X),
        0x59 => Some(Key::Y),
        0x5A => Some(Key::Z),

        VK_LWIN => Some(Key::LSuper),
        VK_RWIN => Some(Key::RSuper),
        VK_APPS => Some(Key::Applications),

        // Reserved.
        0x5E => None,

        VK_SLEEP => Some(Key::Sleep),

        VK_NUMPAD0 => Some(Key::Numpad0),
        VK_NUMPAD1 => Some(Key::Numpad1),
        VK_NUMPAD2 => Some(Key::Numpad2),
        VK_NUMPAD3 => Some(Key::Numpad3),
        VK_NUMPAD4 => Some(Key::Numpad4),
        VK_NUMPAD5 => Some(Key::Numpad5),
        VK_NUMPAD6 => Some(Key::Numpad6),
        VK_NUMPAD7 => Some(Key::Numpad7),
        VK_NUMPAD8 => Some(Key::Numpad8),
        VK_NUMPAD9 => Some(Key::Numpad9),

        VK_MULTIPLY => Some(Key::Multiply),
        VK_ADD => Some(Key::Add),
        VK_SEPARATOR => Some(Key::Separator), // TODO: document this nightmare
        VK_SUBTRACT => Some(Key::Subtract),
        VK_DECIMAL => Some(Key::Decimal),
        VK_DIVIDE => Some(Key::Divide),

        VK_F1 => Some(Key::F1),
        VK_F2 => Some(Key::F2),
        VK_F3 => Some(Key::F3),
        VK_F4 => Some(Key::F4),
        VK_F5 => Some(Key::F5),
        VK_F6 => Some(Key::F6),
        VK_F7 => Some(Key::F7),
        VK_F8 => Some(Key::F8),
        VK_F9 => Some(Key::F9),
        VK_F10 => Some(Key::F10),
        VK_F11 => Some(Key::F11),
        VK_F12 => Some(Key::F12),
        VK_F13 => Some(Key::F13),
        VK_F14 => Some(Key::F14),
        VK_F15 => Some(Key::F15),
        VK_F16 => Some(Key::F16),
        VK_F17 => Some(Key::F17),
        VK_F18 => Some(Key::F18),
        VK_F19 => Some(Key::F19),
        VK_F20 => Some(Key::F20),
        VK_F21 => Some(Key::F21),
        VK_F22 => Some(Key::F22),
        VK_F23 => Some(Key::F23),
        VK_F24 => Some(Key::F24),

        // Unassigned.
        0x88..=0x8F => None,

        VK_NUMLOCK => Some(Key::NumLock),
        VK_SCROLL => Some(Key::ScrollLock),

        // OEM Specific. TODO, perhaps.
        0x92..=0x96 => None,

        // Unassigned.
        0x97..=0x9F => None,

        // These values are only recognized by GetAsyncKeyState and related,
        // but I'll add them for completion regardless.
        VK_LSHIFT => Some(Key::LShift),
        VK_RSHIFT => Some(Key::RShift),
        VK_LCONTROL => Some(Key::LControl),
        VK_RCONTROL => Some(Key::RControl),
        VK_LMENU => Some(Key::LAlt),
        VK_RMENU => Some(Key::RAlt),

        VK_BROWSER_BACK => Some(Key::BrowserBack),
        VK_BROWSER_FORWARD => Some(Key::BrowserForward),
        VK_BROWSER_REFRESH => Some(Key::BrowserRefresh),
        VK_BROWSER_STOP => Some(Key::BrowserStop),
        VK_BROWSER_SEARCH => Some(Key::BrowserSearch),
        VK_BROWSER_FAVORITES => Some(Key::BrowserFavourites),
        VK_BROWSER_HOME => Some(Key::BrowserHome),
        VK_VOLUME_MUTE => Some(Key::VolumeMute),
        VK_VOLUME_DOWN => Some(Key::VolumeDown),
        VK_VOLUME_UP => Some(Key::VolumeUp),
        VK_MEDIA_NEXT_TRACK => Some(Key::MediaNextTrack),
        VK_MEDIA_PREV_TRACK => Some(Key::MediaPreviousTrack),
        VK_MEDIA_STOP => Some(Key::MediaStop),
        VK_MEDIA_PLAY_PAUSE => Some(Key::MediaPlayPause),
        VK_LAUNCH_MAIL => Some(Key::LaunchMail),
        VK_LAUNCH_MEDIA_SELECT => Some(Key::LaunchMediaSelect),
        VK_LAUNCH_APP1 => Some(Key::LaunchApplication1),
        VK_LAUNCH_APP2 => Some(Key::LaunchApplication2),

        // Reserved.
        0xB8..=0xB9 => None,

        VK_OEM_1 => Some(Key::Oem1),
        VK_OEM_PLUS => Some(Key::Plus),
        VK_OEM_COMMA => Some(Key::Comma),
        VK_OEM_MINUS => Some(Key::Minus),
        VK_OEM_PERIOD => Some(Key::Period),
        VK_OEM_2 => Some(Key::Oem2),
        VK_OEM_3 => Some(Key::Oem3),

        // Reserved (VK_GAMEPAD_xxx).
        0xC1..=0xDA => None,

        VK_OEM_4 => Some(Key::Oem4),
        VK_OEM_5 => Some(Key::Oem5),
        VK_OEM_6 => Some(Key::Oem6),
        VK_OEM_7 => Some(Key::Oem7),
        VK_OEM_8 => Some(Key::Oem8),

        // Reserved.
        0xE0 => None,

        // TODO: "OEM Specific"
        0xE1 => None,

        VK_OEM_102 => Some(Key::Oem102),

        // TODO: "OEM Specific"
        0xE3..=0xE4 => None,

        VK_PROCESSKEY => Some(Key::ImeProcess),

        // TODO: "OEM Specific"
        0xE6 => None,

        VK_PACKET => None, // TODO

        // Unassigned.
        0xE8 => None,

        // TODO: "OEM Specific"
        0xE9..=0xF5 => None,

        VK_ATTN => Some(Key::Attn),
        VK_CRSEL => Some(Key::CrSel),
        VK_EXSEL => Some(Key::ExSel),
        VK_EREOF => Some(Key::EraseEof),
        VK_PLAY => Some(Key::Play),
        VK_ZOOM => Some(Key::Zoom),

        // Reserved.
        VK_NONAME => None,

        VK_PA1 => Some(Key::Pa1),
        VK_OEM_CLEAR => Some(Key::OemClear),

        // Undocumented.
        0xFF => None,
    }
}

#[cfg(feature = "input")]
fn sys_key_event(wparam: WPARAM, lparam: LPARAM) -> Option<Event> {
    let alt_bit = (lparam & (1 << 29)) != 0;
    let transition_state = (lparam & (1 << 31)) != 0;

    // If it's not an F10 press, and the alt bit is not set,
    // and it's not a key release, it is not being sent by a key.
    if wparam & 0xFF != VK_F10 as WPARAM && !alt_bit && !transition_state {
        return None
    }

    let virtual_key = translate_vk(wparam)?;
    let key = extend_key(virtual_key, lparam);

    Some(map_tr_state(key, lparam))
}

#[cfg(feature = "input")]
fn extend_key(key: Key, lparam: LPARAM) -> Key {
    let scancode = (lparam & 0x00FF0000) >> 16u8;
    let extended_bit = (lparam & (1 << 24)) != 0;

    match key {
        Key::LShift if scancode == 54 => Key::RShift,
        Key::LControl if extended_bit => Key::RControl,
        Key::LAlt if extended_bit => Key::RAlt,
        x => x,
    }
}

#[cfg(feature = "input")]
fn map_tr_state(key: Key, lparam: LPARAM) -> Event {
    if (lparam & (1 << 31)) == 0 {
        if (lparam & (1 << 30)) != 0 {
            Event::KeyboardRepeat(key)
        } else {
            Event::KeyboardDown(key)
        }
    } else {
        Event::KeyboardUp(key)
    }
}
