//! yeah

/// Represents an open window. Dropping it closes the window.
/// 
/// To instantiate windows, use a [`builder`](Self::builder).
pub struct Window {

}

/// Builder for creating [`Window`] instances.
///
/// To create a builder, use [`Window::builder`].
pub struct WindowBuilder {

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
}
