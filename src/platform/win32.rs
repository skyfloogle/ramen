//! Win32-specific implementations and API extensions.

pub(crate) mod ffi;
pub(crate) mod imp;

// Required re-exports
pub(crate) use imp::spawn_window;
pub(crate) type WindowRepr = imp::WindowImpl;

// Bonus
pub use ffi::{HINSTANCE, HMONITOR, HWND};
pub use imp::this_hinstance;
