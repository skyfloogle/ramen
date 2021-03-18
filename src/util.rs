use std::{clone::Clone, sync::Arc};

pub enum MaybeArc<T: 'static + ?Sized> {
    Static(&'static T),
    Dynamic(Arc<T>),
}

impl<T: 'static + ?Sized> AsRef<T> for MaybeArc<T> {
    fn as_ref(&self) -> &T {
        match self {
            Self::Static(s) => s,
            Self::Dynamic(d) => d.as_ref(),
        }
    }
}

impl<T: 'static + ?Sized> Clone for MaybeArc<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Static(x) => Self::Static(x),
            Self::Dynamic(x) => Self::Dynamic(Arc::clone(x)),
        }
    }   
}
