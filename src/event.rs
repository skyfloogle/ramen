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

/// An event received from the event loop of a [`Window`](crate::window::Window).
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

    /// A [`MouseButton`] was pressed.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "input")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "input"))]
    MouseDown(MouseButton),

    /// A [`MouseButton`] was released.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "input")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "input"))]
    MouseUp(MouseButton),

    /// A [`Key`] was pressed.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "input")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "input"))]
    KeyboardDown(Key),

    /// A [`Key`] was auto-repeated by the system.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "input")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "input"))]
    KeyboardRepeat(Key),

    /// A [`Key`] was released.
    #[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "input")))]
    #[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "input"))]
    KeyboardUp(Key),

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

/// Represents a button on the keyboard.
#[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "input")))]
#[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "input"))]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Key {
    Attn,
    LAlt,
    RAlt,
    Applications,
    Backspace,
    CapsLock,
    Clear,
    LControl,
    RControl,
    CrSel,
    Delete,
    End,
    Enter,
    EraseEof,
    Escape,
    Execute,
    ExSel,
    Help,
    Home,
    Insert,
    NumLock,
    Pa1,
    PageUp,
    PageDown,
    Pause,
    Play,
    Print,
    PrintScreen,
    LShift,
    RShift,
    ScrollLock,
    Select,
    Sleep,
    Space,
    LSuper,
    RSuper,
    Tab,
    Zoom,

    Left, Up, Right, Down,

    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Comma, Minus, Period, Plus,
    Oem1, Oem2, Oem3, Oem4, Oem5, Oem6, Oem7, Oem8,
    Oem102, OemClear,

    Add, Subtract, Multiply, Divide,
    Decimal, Separator,
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,

    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,

    BrowserBack,
    BrowserFavourites,
    BrowserForward,
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,

    ImeAccept,
    ImeConvert,
    ImeNonConvert,
    ImeFinal,
    ImeModeChange,
    ImeProcess,
    ImeOn,
    ImeOff,

    // TODO: better understanding of these
    ImeKana,
    ImeKanji,
    ImeJunja,

    MediaNextTrack,
    MediaPreviousTrack,
    MediaPlayPause,
    MediaStop,

    VolumeDown,
    VolumeUp,
    VolumeMute,

    LaunchApplication1,
    LaunchApplication2,
    LaunchMail, // what the fuck?
    LaunchMediaSelect,
}

/// Represents a button on the mouse.
#[cfg_attr(feature = "nightly-docs", doc(cfg(feature = "input")))]
#[cfg_attr(not(feature = "nightly-docs"), cfg(feature = "input"))]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MouseButton {
    /// Left Mouse Button
    Left,

    /// Right Mouse Button
    Right,

    /// M3, sometimes known as "middle mouse" or the scroll wheel button
    Middle,

    /// M4, sometimes known as XButton1
    Mouse4,

    /// M5, sometimes known as XButton2
    Mouse5,
}
