use crate::{
    error::Error,
    event::{CloseReason, Event},
    util::{sync::{self, Condvar, Mutex}, FixedVec, LazyCell},
    window::{self, WindowBuilder},
};
use std::{cell::UnsafeCell, mem, ops, ptr, slice, sync::{atomic, Arc}, thread};

// TODO: Maybe deglob
use crate::platform::win32::ffi::*;

// TODO: Measure this
const MAX_EVENTS_PER_SWAP: usize = 4096;

/// Marker to filter out implementation magic like `CicMarshalWndClass`
const HOOKPROC_MARKER: &[u8; 4] = b"viri";

// Custom window messages
const RAMEN_WM_DROP: UINT = WM_USER + 0;

// (Get/Set)(Class/Window)Long(A/W) all took LONG, a 32-bit type.
// When MS went from 32 to 64 bit, they realized how big of a mistake this was,
// seeing as some of those values need to be as big as a pointer is (like size_t).
// Unfortunately they exported the 32-bit ones on 64-bit with mismatching signatures.
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

/// Converts a &str to an LPCWSTR-compatible string array.
fn str_to_wstr(src: &str, buffer: &mut Vec<WCHAR>) -> LPCWSTR {
    // NOTE: Yes, indeed, `std::os::windows::OsStr(ing)ext` does exist in the standard library,
    // but it requires you to fit your data in the OsStr(ing) model and it's not hyper optimized
    // unlike mb2wc with handwritten SSE (allegedly), alongside being the native conversion function

    // MultiByteToWideChar can't actually handle 0 length because 0 return means error
    if src.is_empty() || src.len() > c_int::max_value() as usize {
        return [0x00].as_ptr()
    }

    unsafe {
        let str_ptr: LPCSTR = src.as_ptr().cast();
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

/// Retrieves the base module HINSTANCE.
#[inline]
pub fn this_hinstance() -> HINSTANCE {
    extern "system" {
        // Microsoft's linkers provide a static HINSTANCE to not have to query it at runtime.
        // Source: https://devblogs.microsoft.com/oldnewthing/20041025-00/?p=37483
        // (I love you Raymond Chen)
        static __ImageBase: [u8; 64];
    }
    (unsafe { &__ImageBase }) as *const [u8; 64] as HINSTANCE
}

impl window::Style {
    /// Gets this style as a bitfield. Note that it does not include the close button.
    /// The close button is a menu property, not a window style.
    pub(crate) fn dword_style(&self) -> DWORD {
        let mut style = 0;

        if self.borderless {
            // TODO: Why does this just not work without THICKFRAME? Borderless is dumb.
            style |= WS_POPUP | WS_THICKFRAME;
        } else {
            style |= WS_OVERLAPPED | WS_BORDER | WS_CAPTION;
        }

        if self.resizable {
            style |= WS_THICKFRAME;
        }

        if self.visible {
            style |= WS_VISIBLE;
        }

        if let Some(controls) = &self.controls {
            if controls.minimize {
                style |= WS_MINIMIZEBOX;
            }
            if controls.maximize {
                style |= WS_MAXIMIZEBOX;
            }
            style |= WS_SYSMENU;
        }

        style
    }

    /// Gets the extended window style.
    pub(crate) fn dword_style_ex(&self) -> DWORD {
        let mut style = 0;

        if self.rtl_layout {
            style |= WS_EX_LAYOUTRTL;
        }

        if self.tool_window {
            style |= WS_EX_TOOLWINDOW;
        }

        style
    }

    /// Sets both styles for target window handle.
    pub(crate) fn set_for(&self, hwnd: HWND) {
        let style = self.dword_style();
        let style_ex = self.dword_style_ex();
        unsafe {
            let _ = set_window_data(hwnd, GWL_STYLE, style as usize);
            let _ = set_window_data(hwnd, GWL_EXSTYLE, style_ex as usize);
        }
    }
}

/// Implementation container for `Window`
pub struct WindowImpl {
    hwnd: HWND,
    thread: Option<thread::JoinHandle<()>>,
    user: *mut WindowImplData, // 'thread
}

unsafe impl Send for WindowImpl {}
unsafe impl Sync for WindowImpl {}

/// Info struct for `WM_(NC)CREATE`
pub struct WindowImplCreateParams {
    error: Option<Error>,
    user: *mut WindowImplData,
}

/// User data structure
pub struct WindowImplData {
    // Prevent external close attempts
    destroy_flag: atomic::AtomicBool,

    close_reason: Option<CloseReason>,
    focus_state: bool,

    // Read `Self::push_event`
    ev_buf_sync: Mutex<bool>,
    ev_buf_ping: Condvar,
    ev_buf_is_primary: bool,
    ev_buf_primary: FixedVec<Event, MAX_EVENTS_PER_SWAP>,
    ev_buf_secondary: FixedVec<Event, MAX_EVENTS_PER_SWAP>,
}

/// To avoid two threads trying to register a window class at the same time,
/// this global mutex is locked while doing window class queries / entries.
static CLASS_REGISTRY_LOCK: LazyCell<Mutex<()>> = LazyCell::new(|| Mutex::new(()));

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

        let style = builder.style.dword_style();
        let style_ex = builder.style.dword_style_ex();
        let (width, height) = (1280, 720);
        let (pos_x, pos_y) = (CW_USEDEFAULT, CW_USEDEFAULT);

        // Special
        let user_data: UnsafeCell<WindowImplData> = UnsafeCell::new(WindowImplData {
            close_reason: None, // unknown
            destroy_flag: atomic::AtomicBool::new(false),
            focus_state: false,
            ev_buf_sync: Mutex::new(false),
            ev_buf_ping: Condvar::new(),
            ev_buf_is_primary: true,
            ev_buf_primary: FixedVec::new(),
            ev_buf_secondary: FixedVec::new(),
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
            (&mut create_params) as *mut _ as LPVOID,
        );

        if hwnd.is_null() {
            if create_params.error.is_none() {
                // TODO: Push create failure
            }
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
                0 => if (&*user_data.get()).destroy_flag.load(atomic::Ordering::Acquire) {
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
    pub fn events(&self) -> &[Event] {
        // the backbuffer contains "last" events, so use *not* the active one
        let user_data = unsafe { &*self.user };
        if user_data.ev_buf_is_primary {
            user_data.ev_buf_secondary.slice()
        } else {
            user_data.ev_buf_primary.slice()
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
    pub fn push_event(&mut self, event: Event) {
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
            if *lock == true || !ev_buf.push(&event) {
                *lock = true; // "the condvar should be pinged"
                sync::condvar_wait(&self.ev_buf_ping, &mut lock);
            } else {
                break
            }
        }
    }
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
    // Fantastic resource for a list of window messages:
    // https://wiki.winehq.org/List_Of_Windows_Messages
    match msg {
        // No-op event, used for pinging the event loop, etc. Return 0.
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

        // TODO: ...
        WM_SIZE => {
            // TODO: ...
            DefWindowProcW(hwnd, msg, wparam, lparam)
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
            //         if user_data.focus_state != state {
            //             user_data.focus_state = state;
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

            if !user_data.focus_state {
                user_data.focus_state = true;
                user_data.push_event(Event::Focus(true));
            }

            // TODO: Cursor lock nonsense

            0
        },

        // Received when a Win32 window loses keyboard focus. Return 0.
        // See also: `WM_SETFOCUS` and `WM_ACTIVATE`
        WM_KILLFOCUS => {
            let user_data = user_data(hwnd);
            if user_data.focus_state {
                user_data.focus_state = false;
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

        // Supposedly `WM_ACTIVATE`, but only received if the focus is to a different application.
        // This doesn't seem to be actually true, and it even has the same bugs as `WM_ACTIVATE`.
        // For this reason (and being useless and confusing) it should be ignored. Return 0.
        // See also: `WM_ACTIVATE`
        WM_ACTIVATEAPP => 0,

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

            // This is where some things like the title contents are stored,
            // so make sure to forward `WM_NCCREATE` to DefWindowProcW
            DefWindowProcW(hwnd, msg, wparam, lparam)
        },

        // Custom message: The "real" destroy signal that won't be rejected.
        // TODO: document the rejection emchanism somewhere
        // Return 0.
        RAMEN_WM_DROP => {
            user_data(hwnd).destroy_flag.store(true, atomic::Ordering::Release);
            let _ = DestroyWindow(hwnd);
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
