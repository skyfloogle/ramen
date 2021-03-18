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
    title: MaybeArc<str>,
}

impl Window {
    pub const fn builder() -> WindowBuilder {
        WindowBuilder::new()
    }
}

impl WindowBuilder {
    pub(crate) const fn new() -> Self {
        Self {}
    }

    pub fn build(&self) -> Result<Window, Error> {
        imp::spawn_window(self).map(Window)
    }

    pub fn title<T>(&mut self, title: T) -> &mut Self
    where
        T: Into<Cow<'static, str>>,
    {
        self.title = match title.into() {
            Cow::Borrowed(x) => x.into(),
            Cow::Owned(x) => MaybeArc::Dynamic(x.into()),
        };
        self
    }
}
