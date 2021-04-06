use crate::monitor::{Point, Scale, Size};

/// Details the source of [`Event::CloseRequest`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CloseReason {
    /// The user has pressed a system control to close the window.
    ///
    /// This is usually the "X button" or the red stop light on the control menu.
    SystemMenu,

    /// The user has pressed the system keyboard shortcut to close the active window.
    ///
    /// This is usually something like Alt+F4, Command+W, or Control+W.
    KeyboardShortcut,

    /// The reason for the close request is unknown.
    ///
    /// Likely reasons include external programs sending the signal.
    Unknown,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Event {
    /// The window has requested to close.
    /// For more information on why, see the associated [`CloseReason`].
    CloseRequest(CloseReason),

    /// The window focus state has been updated (`true` if focused).
    Focus(bool),

    /// The window's maximize state has been updated (`true` if maximized).
    Maximize(bool),

    /// The window's minimize state has been updated (`true` if minimized).
    Minimize(bool),

    /// The mouse has entered (`true`) or left (`false`) the inner area of the window.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "input")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "input"))]
    MouseFocus(bool),

    /// The position of the mouse inside the window has been updated.
    ///
    /// The associated values work the same as [`Event::Resize`].
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "input")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "input"))]
    MouseMove((Point, Scale)),

    /// The window has been resized or had its DPI scaling modified.
    ///
    /// For more info, see: [`Window::inner_size`](crate::window::Window::inner_size)
    Resize((Size, Scale)),
}
