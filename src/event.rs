#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Event {
    /// The window focus has been updated: `true` if focused, `false` if unfocused.
    Focus(bool),
}
