use std::{
    cell::UnsafeCell,
    clone::Clone,
    mem::MaybeUninit,
    ops::Deref, ptr,
    sync::{Arc, Once},
};

pub fn str_has_nulls(s: &str) -> bool {
    s.bytes().any(|b| b == 0x00)
}

pub fn str_sweep_nulls(s: &mut String) {
    // SAFETY: 0x00 is a one-byte null and is safe to swap with 0x20
    for byte in unsafe { s.as_mut_vec().iter_mut() } {
        if *byte == 0x00 {
            *byte = b' ';
        }
    }
}

pub struct FixedVec<T, const N: usize> {
    array: MaybeUninit<[T; N]>,
    len: usize,
}

impl<T: Copy, const N: usize> FixedVec<T, N> {
    pub fn clear(&mut self) {
        for el in self.slice_mut() {
            unsafe {
                ptr::drop_in_place(el);
            }
        }
        self.len = 0;
    }

    pub fn new() -> Self {
        Self {
            array: MaybeUninit::uninit(),
            len: 0,
        }
    }

    pub fn push(&mut self, item: &T) -> bool {
        self.push_many(unsafe {
            std::slice::from_raw_parts(item, 1)
        })
    }

    pub fn push_many(&mut self, items: &[T]) -> bool {
        if self.len + items.len() <= N {
            unsafe {
                ptr::copy_nonoverlapping(
                    items.as_ptr(),
                    (&mut *self.array.as_mut_ptr())
                        .get_unchecked_mut(self.len),
                    items.len(),
                );
            }
            self.len += items.len();
            true
        } else {
            false
        }
    }

    pub fn slice(&self) -> &[T] {
        unsafe {
            (&*self.array.as_ptr()).get_unchecked(..self.len)
        }
    }

    pub fn slice_mut(&mut self) -> &mut [T] {
        unsafe {
            (&mut *self.array.as_mut_ptr()).get_unchecked_mut(..self.len)
        }
    }
}

/// Minimal lazily initialized type, similar to the one in `once_cell`.
///
/// Thread safe initialization, immutable-only access.
pub struct LazyCell<T, F = fn() -> T> {
    // Invariant: Written to at most once on first access.
    init: UnsafeCell<Option<F>>,
    ptr: UnsafeCell<*const T>,

    // Synchronization primitive for initializing `init` and `ptr`.
    once: Once,
}

unsafe impl<T, F> Send for LazyCell<T, F> where T: Send {}
unsafe impl<T, F> Sync for LazyCell<T, F> where T: Sync {}

impl<T, F> LazyCell<T, F> {
    pub const fn new(init: F) -> Self {
        Self {
            init: UnsafeCell::new(Some(init)),
            ptr: UnsafeCell::new(ptr::null()),
            once: Once::new(),
        }
    }
}

impl<T, F: FnOnce() -> T> LazyCell<T, F> {
    pub fn get(&self) -> &T {
        self.once.call_once(|| unsafe {
            if let Some(f) = (&mut *self.init.get()).take() {
                let pointer = Box::into_raw(Box::new(f()));
                ptr::write(self.ptr.get(), pointer);
            }
        });

        // SAFETY: A call to `call_once` initialized the pointer
        unsafe {
            &**self.ptr.get()
        }
    }
}

impl<T, F: FnOnce() -> T> Deref for LazyCell<T, F> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

/// Static or dynamic data shared across threads.
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

/// Wrapper for working with both `std` and `parking_lot`.
/// None of these functions should panic when used correctly as they're used in FFI.
#[cfg(not(feature = "parking-lot"))]
pub(crate) mod sync {
    pub use std::sync::{Condvar, Mutex, MutexGuard};
    use std::ptr;

    #[inline]
    pub fn condvar_notify1(cvar: &Condvar) {
        cvar.notify_one();
    }

    pub fn condvar_wait<T>(cvar: &Condvar, guard: &mut MutexGuard<T>) {
        // The signature in `std` is quite terrible and CONSUMES the guard
        // HACK: We "move it out" for the duration of the wait
        unsafe {
            let guard_copy = ptr::read(guard);
            let result = cvar.wait(guard_copy).expect("cvar mutex poisoned (this is a bug)");
            ptr::write(guard, result);
        }
    }

    pub fn mutex_lock<T>(mtx: &Mutex<T>) -> MutexGuard<T> {
        mtx.lock().expect("mutex poisoned (this is a bug)")
    }
}
#[cfg(feature = "parking-lot")]
pub(crate) mod sync {
    pub use parking_lot::{Condvar, Mutex, MutexGuard};

    #[inline]
    pub fn condvar_notify1(cvar: &Condvar) {
        let _ = cvar.notify_one();
    }

    #[inline]
    pub fn condvar_wait<T>(cvar: &Condvar, guard: &mut MutexGuard<T>) {
        cvar.wait(guard);
    }

    #[inline]
    pub fn mutex_lock<T>(mtx: &Mutex<T>) -> MutexGuard<T> {
        mtx.lock()
    }
}
