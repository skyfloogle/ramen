//! Platform-specific implementations and API extensions.

// Ignoring an API result by mistake should not be a possibility.
#![deny(unused_results)]

// DEVELOPER NOTES:
//
// The module `imp` (for implementation) should be pub(crate) exported, with:
// - The type `WindowRepr` which is callable as a `WindowTrait` (deref / impl)
// - The function `spawn_window` which is `fn(&WindowBuilder) -> Result<WindowRepr, Error>`
// For an example, see `src/platform/win32.rs`

#[cfg_attr(feature = "nightly-docs", doc(cfg(target_os = "windows")))]
#[cfg_attr(not(feature = "nightly-docs"), cfg(target_os = "windows"))]
pub mod win32;
#[cfg(target_os = "windows")]
pub(crate) use win32 as imp;
