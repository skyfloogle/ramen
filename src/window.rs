//! yeah

use crate::{
    error::Error,
    event::Event,
    monitor::{/*Point, */ Scale, Size},
    platform::imp,
    util::{self, MaybeArc},
};
use std::borrow::Cow;

/// Represents the availability of the minimize, maximize, and close buttons on a [`Window`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Controls {
    pub minimize: bool,
    pub maximize: bool,
    pub close: bool,
}

impl Controls {
    /// Creates window controls from the provided values.
    pub const fn new(minimize: bool, maximize: bool, close: bool) -> Self {
        Controls {
            minimize,
            maximize,
            close,
        }
    }

    /// Creates window controls with all 3 buttons enabled.
    pub const fn enabled() -> Self {
        Self::new(true, true, true)
    }

    /// Creates window controls with the minimize & close buttons available.
    pub const fn no_maximize() -> Self {
        Self::new(true, false, true)
    }

    pub(crate) fn to_bits(&self) -> u32 {
        (self.minimize as u32) << 2 | (self.maximize as u32) << 1 | self.close as u32
    }

    pub(crate) fn from_bits(x: u32) -> Self {
        Self {
            minimize: x & (1 << 2) != 0,
            maximize: x & (1 << 1) != 0,
            close: x & 1 != 0,
        }
    }
}

impl Default for Controls {
    /// Default trait implementation, same as [`Controls::new`].
    fn default() -> Self {
        Self::enabled()
    }
}

/// yeah
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Cursor {
    /// ⇖
    Arrow = 0,

    /// (Invisible)
    Blank,

    /// \+
    Cross,

    /// 👆
    Hand,

    /// 👆?
    Help,

    /// I
    IBeam,

    /// ⇖⌛
    Progress,

    /// ⤢
    ResizeNESW,

    /// ↕
    ResizeNS,

    /// ⤡
    ResizeNWSE,

    /// ↔
    ResizeWE,

    /// ✥
    ResizeAll,

    /// 🚫
    Unavailable,

    /// ⌛
    Wait,
}

/// Represents an open window. Dropping it closes the window.
///
/// To instantiate windows, use a [`builder`](Self::builder).
pub struct Window(pub(crate) imp::WindowRepr);

/// Builder for creating [`Window`] instances.
///
/// To create a builder, use [`Window::builder`].
#[derive(Clone)]
pub struct WindowBuilder {
    pub(crate) class_name: MaybeArc<str>,
    pub(crate) cursor: Cursor,
    pub(crate) inner_size: Size,
    pub(crate) style: Style,
    pub(crate) title: MaybeArc<str>,
}

impl Window {
    pub const fn builder() -> WindowBuilder {
        WindowBuilder::new()
    }
}

impl Window {
    /// Executes an arbitrary function in the window thread, blocking until it returns.
    ///
    /// This is **not** how functions such as [`set_visible`](Self::set_visible) are implemented,
    /// but rather a way to guarantee that native low-level calls are executed in the remote thread if necessary,
    /// especially on platforms like Win32 that make excessive use of thread globals.
    ///
    /// # Example
    /// Note that you can choose to yield a value of any type from the closure:
    /// ```no_run
    /// # let window = ramen::window::Window::builder().build().unwrap();
    /// let result = window.execute(|window| {
    ///     window.set_title("hi from the window thread");
    ///     1702
    /// });
    /// ```
    #[inline]
    pub fn execute<F, T>(&self, f: F) -> T
    where
        F: Send + FnOnce(&Self) -> T,
        T: Send,
    {
        self.0.execute(move || f(self))
    }

    #[inline]
    pub fn events(&self) -> &[Event] {
        self.0.events()
    }

    #[inline]
    pub fn swap_events(&mut self) {
        self.0.swap_events()
    }

    /// Gets the inner size of the window.
    ///
    /// It should be preferred to cache this and process events to listen for changes,
    /// as it requires a wait for the window thread.
    ///
    /// # Example
    ///
    /// The [`Size`] variant will depend on the one supplied to
    /// [`Window::set_inner_size`] or [`WindowBuilder::inner_size`].\
    /// For more information on the DPI scaling system, read the documentation
    /// on either of those two functions.
    ///
    /// Regardless of DPI mode, with the provided [`Scale`]
    /// the returned value can be converted to either unit:
    ///
    /// ```no_run
    /// # let window = ramen::window::Window::builder().build().unwrap();
    /// let (size, scale) = window.inner_size();
    ///
    /// let (lwidth, lheight) = size.as_logical(scale); // get logical size
    /// let (pwidth, pheight) = size.as_physical(scale); // get physical size
    /// ```
    #[inline]
    pub fn inner_size(&self) -> (Size, Scale) {
        self.0.inner_size()
    }

    // TODO: borderless

    /// Sets the availability of the window controls.
    ///  `None` indicates that no control menu is desired.
    #[inline]
    pub fn set_controls(&self, controls: Option<Controls>) {
        self.0.set_controls(controls);
    }

    /// Non-blocking variant of [`set_controls`](Self::set_controls).
    #[inline]
    pub fn set_controls_async(&self, controls: Option<Controls>) {
        self.0.set_controls_async(controls);
    }

    /// Sets the cursor that's shown when the mouse is inside of the window's inner area.
    #[inline]
    pub fn set_cursor(&self, cursor: Cursor) {
        self.0.set_cursor(cursor);
    }

    /// Non-blocking variant of [`set_cursor`](Self::set_cursor).
    #[inline]
    pub fn set_cursor_async(&self, cursor: Cursor) {
        self.0.set_cursor_async(cursor);
    }

    /// BRUH
    ///
    ///
    /// If the size provided is [`Logical`](Size::Logical), the window will scale accordingly
    /// if the DPI changes (for example by being dragged onto a different monitor, or changing settings).
    /// If it's [`Physical`](Size::Physical) then no scaling will be done and it'll be treated as an exact pixel value.
    #[inline]
    pub fn set_inner_size(&self, size: Size) {
        self.0.set_inner_size(size)
    }

    /// Sets whether the window is resizable by dragging the edges.
    #[inline]
    pub fn set_resizable(&self, resizable: bool) {
        self.0.set_resizable(resizable);
    }

    /// Non-blocking variant of [`set_resizable`](Self::set_resizable).
    #[inline]
    pub fn set_resizable_async(&self, resizable: bool) {
        self.0.set_resizable_async(resizable);
    }

    /// Sets the text that appears in the title bar of the window.
    ///
    /// Note that if the window is borderless, fullscreen, or simply has no title bar,
    /// the change will not be visible.\
    /// It will however persist for when the style is changed to later include a title bar.
    #[inline]
    pub fn set_title(&self, title: &str) {
        self.0.set_title(title);
    }

    /// Non-blocking variant of [`set_title`](Self::set_title).
    #[inline]
    pub fn set_title_async(&self, title: &str) {
        self.0.set_title_async(title);
    }

    /// Sets whether the window is hidden (`false`) or visible (`true`).
    #[inline]
    pub fn set_visible(&self, visible: bool) {
        self.0.set_visible(visible);
    }

    /// Non-blocking variant of [`set_visible`](Self::set_visible).
    #[inline]
    pub fn set_visible_async(&self, visible: bool) {
        self.0.set_visible_async(visible);
    }
}

impl WindowBuilder {
    pub(crate) const fn new() -> Self {
        Self {
            class_name: MaybeArc::Static("ramen_window"),
            cursor: Cursor::Arrow,
            inner_size: Size::Logical(800.0, 608.0),
            style: Style {
                borderless: false,
                resizable: true,
                visible: true,
                controls: Some(Controls::enabled()),
                rtl_layout: false,

                #[cfg(windows)]
                tool_window: false,
            },
            title: MaybeArc::Static("a nice window"),
        }
    }

    pub fn build(&self) -> Result<Window, Error> {
        imp::spawn_window(self).map(Window)
    }
}

impl WindowBuilder {
    /// Sets whether the window is initially without a border.
    ///
    /// Defaults to `false`.
    #[inline]
    pub fn borderless(&mut self, borderless: bool) -> &mut Self {
        self.style.borderless = borderless;
        self
    }

    /// Sets the platform-specific window class name.
    ///
    /// - Win32: `lpszClassName` in
    /// [`WNDCLASSEXW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassexw)
    /// - TODO: Other platforms!
    ///
    /// Defaults to `"ramen_window"`.
    pub fn class_name<T>(&mut self, class_name: T) -> &mut Self
    where
        T: Into<Cow<'static, str>>,
    {
        let mut class_name = class_name.into();
        if util::str_has_nulls(class_name.as_ref()) {
            let mut dirty = class_name.into_owned();
            util::str_sweep_nulls(&mut dirty);
            class_name = Cow::Owned(dirty);
        }
        self.class_name = match class_name {
            Cow::Borrowed(b) => MaybeArc::Static(b),
            Cow::Owned(o) => MaybeArc::Dynamic(o.into()),
        };
        self
    }

    /// Sets the initial window controls.
    /// `None` indicates that no control menu is desired.
    ///
    /// Defaults to [`Controls::enabled`].
    #[inline]
    pub fn controls(&mut self, controls: Option<Controls>) -> &mut Self {
        self.style.controls = controls;
        self
    }

    /// Sets the initial cursor shown in the window.
    ///
    /// Defaults to [`Cursor::Arrow`].
    #[inline]
    pub fn cursor(&mut self, cursor: Cursor) -> &mut Self {
        self.cursor = cursor;
        self
    }

    /// Sets the initial inner size of the window.
    ///
    /// If the size provided is [`Logical`](Size::Logical), the window will scale accordingly
    /// if the DPI changes (for example by being dragged onto a different monitor, or changing settings).
    /// If it's [`Physical`](Size::Physical) then no scaling will be done and it'll be treated as an exact pixel value.
    ///
    /// Defaults to `Size::Logical(800.0, 608.0)`.
    #[inline]
    pub fn inner_size(&mut self, inner_size: Size) -> &mut Self {
        self.inner_size = inner_size;
        self
    }

    /// Sets whether the window is initially resizable.
    ///
    /// Defaults to `true`.
    #[inline]
    pub fn resizable(&mut self, resizable: bool) -> &mut Self {
        self.style.resizable = resizable;
        self
    }

    /// Sets whether the window controls and titlebar have a right-to-left layout.
    ///
    /// Defaults to `false`.
    #[inline]
    pub fn rtl_layout(&mut self, rtl_layout: bool) -> &mut Self {
        self.style.rtl_layout = rtl_layout;
        self
    }

    /// Sets the initial window title.
    ///
    /// Defaults to `"a nice window"`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ramen::window::Window;
    ///
    /// let mut builder = Window::builder()
    ///     .title("Cool Window") // static reference, or
    ///     .title(String::from("Cool Window")); // owned data
    /// ```
    pub fn title<T>(&mut self, title: T) -> &mut Self
    where
        T: Into<Cow<'static, str>>,
    {
        let mut title = title.into();
        if util::str_has_nulls(title.as_ref()) {
            let mut dirty = title.into_owned();
            util::str_sweep_nulls(&mut dirty);
            title = Cow::Owned(dirty);
        }
        self.title = match title {
            Cow::Borrowed(b) => MaybeArc::Static(b),
            Cow::Owned(o) => MaybeArc::Dynamic(o.into()),
        };
        self
    }

    /// Sets whether the window is initially visible.
    ///
    /// Defaults to `true`.
    #[inline]
    pub fn visible(&mut self, visible: bool) -> &mut Self {
        self.style.visible = visible;
        self
    }
}

#[derive(Clone)]
pub(crate) struct Style {
    pub borderless: bool,
    pub resizable: bool,
    pub visible: bool,
    pub controls: Option<Controls>,
    pub rtl_layout: bool,

    #[cfg(windows)]
    pub tool_window: bool,
}
