//! yeah

use crate::{error::Error, platform::imp, util::{self, MaybeArc}};
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
}

impl Default for Controls {
    /// Default trait implementation, same as [`WindowControls::new`].
    fn default() -> Self {
        Self::enabled()
    }
}

/// Represents an open window. Dropping it closes the window.
/// 
/// To instantiate windows, use a [`builder`](Self::builder).
pub struct Window(imp::WindowRepr);

/// Builder for creating [`Window`] instances.
///
/// To create a builder, use [`Window::builder`].
#[derive(Clone)]
pub struct WindowBuilder {
    pub(crate) class_name: MaybeArc<str>,
    pub(crate) style: Style,
    pub(crate) title: MaybeArc<str>,
}

impl Window {
    pub const fn builder() -> WindowBuilder {
        WindowBuilder::new()
    }
}

impl WindowBuilder {
    pub(crate) const fn new() -> Self {
        Self {
            class_name: MaybeArc::Static("ramen_window"),
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
