//! yeah

use crate::{error::Error, platform::imp, util::MaybeArc};
use std::borrow::Cow;

/// Represents an open window. Dropping it closes the window.
/// 
/// To instantiate windows, use a [`builder`](Self::builder).
pub struct Window(imp::WindowRepr);

/// Builder for creating [`Window`] instances.
///
/// To create a builder, use [`Window::builder`].
pub struct WindowBuilder {
    class_name: MaybeArc<str>,
    title: MaybeArc<str>,
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
