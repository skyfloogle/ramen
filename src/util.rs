use std::sync::Arc;

pub enum MaybeArc<T: 'static + ?Sized> {
    Static(&'static T),
    Dynamic(Arc<T>),
}

impl<T> AsRef<T> for MaybeArc<T> {
    fn as_ref(&self) -> &T {
        match self {
            Self::Static(s) => s,
            Self::Dynamic(d) => d.as_ref(),
        }
    }
}
