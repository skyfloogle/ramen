//! yeah

use crate::{error::Error, platform::imp, util::MaybeArc};
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
            title: MaybeArc::Static("a nice window"),
        }
    }

    pub fn build(&self) -> Result<Window, Error> {
        imp::spawn_window(self).map(Window)
    }
}

impl WindowBuilder {
    pub fn class_name<T>(&mut self, class_name: T) -> &mut Self
    where
        T: Into<Cow<'static, str>>,
    {
        self.class_name = match class_name.into() {
            Cow::Borrowed(b) => MaybeArc::Static(b),
            Cow::Owned(o) => MaybeArc::Dynamic(o.into()),
        };
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
        self.title = match title.into() {
            Cow::Borrowed(b) => MaybeArc::Static(b),
            Cow::Owned(o) => MaybeArc::Dynamic(o.into()),
        };
        self
    }
}
