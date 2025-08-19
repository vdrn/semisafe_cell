#![cfg_attr(feature = "coerce_unsized", feature(coerce_unsized))]

#[cfg(feature = "coerce_unsized")]
use core::ops::CoerceUnsized;

pub use implementation::*;
#[cfg(any(debug_assertions, feature = "full_safety"))]
mod implementation {
    macro_rules! gcell {
        ($inner:ty, $inner_ref:ty, $inner_ref_mut:ty, $cell:ident, $ref:ident, $ref_mut:ident) => {
            #[derive(Default, PartialEq, Debug)]
            pub struct $cell<T: ?Sized>($inner);

            #[cfg(feature = "coerce_unsized")]
            impl<T: super::CoerceUnsized<U>, U> super::CoerceUnsized<$cell<U>> for $cell<T> {}

            impl<T> $cell<T> {
                pub const fn new(value: T) -> Self {
                    Self(<$inner>::new(value))
                }
            }
            impl<T: ?Sized> $cell<T> {
                /// # Safety
                ///
                /// Safe with `debug_assertions` or `full_safety` feature enabled.
                ///
                /// Otherwise, the caller must ensure Rust's aliasing rules are observed:
                /// `borrow` XOR `borrow_mut`.
                #[inline]
                #[track_caller]
                pub unsafe fn borrow(&self) -> $ref<'_, T> {
                    $ref(self.0.borrow())
                }
                /// # Safety
                ///
                /// Safe with `debug_assertions` or `full_safety` feature enabled.
                ///
                /// Otherwise, the caller must ensure Rust's aliasing rules are observed:
                /// `borrow` XOR `borrow_mut`.
                #[inline]
                #[track_caller]
                pub unsafe fn borrow_mut(&self) -> $ref_mut<'_, T> {
                    $ref_mut(self.0.borrow_mut())
                }
                #[inline]
                pub fn get_mut(&mut self) -> &mut T {
                    self.0.get_mut()
                }
                #[inline]
                pub fn ptr(&self) -> *const T {
                    self.0.as_ptr()
                }
                #[inline]
                pub fn mut_ptr(&self) -> *mut T {
                    self.0.as_ptr()
                }
            }

            pub struct $ref<'a, T: ?Sized>($inner_ref);
            impl<'a, T: ?Sized> std::ops::Deref for $ref<'a, T> {
                type Target = T;
                #[inline]
                fn deref(&self) -> &Self::Target {
                    self.0.deref()
                }
            }

            pub struct $ref_mut<'a, T: ?Sized>($inner_ref_mut);
            impl<'a, T: ?Sized> std::ops::Deref for $ref_mut<'a, T> {
                type Target = T;

                #[inline]
                fn deref(&self) -> &Self::Target {
                    self.0.deref()
                }
            }
            impl<'a, T: ?Sized> std::ops::DerefMut for $ref_mut<'a, T> {
                #[inline]
                fn deref_mut(&mut self) -> &mut Self::Target {
                    self.0.deref_mut()
                }
            }
        };
    }

    use std::cell::{Ref, RefCell, RefMut};
    gcell!(RefCell<T>, Ref<'a, T>, RefMut<'a, T>, SemiSafeCell, SemiSafeRef, SemiSafeRefMut);

    use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
    gcell!(
        AtomicRefCell<T>,
        AtomicRef<'a, T>,
        AtomicRefMut<'a, T>,
        SyncSemiSafeCell,
        SyncSemiSafeRef,
        SyncSemiSafeRefMut
    );
}

#[cfg(not(any(debug_assertions, feature = "full_safety")))]
mod implementation {
    macro_rules! gcell {
        ($inner:ty, $cell:ident, $ref:ident, $ref_mut:ident) => {
            #[derive(Default, Debug)]
            pub struct $cell<T: ?Sized>($inner);

            #[cfg(feature = "coerce_unsized")]
            impl<T: super::CoerceUnsized<U>, U> super::CoerceUnsized<$cell<U>> for $cell<T> {}

            impl<T> $cell<T> {
                #[inline]
                pub const fn new(value: T) -> Self {
                    Self(<$inner>::new(value))
                }
            }
            impl<T: ?Sized> $cell<T> {
                /// # Safety
                ///
                /// Safe with `debug_assertions` or `full_safety` feature enabled.
                ///
                /// Otherwise, the caller must ensure Rust's aliasing rules are observed:
                /// `borrow` XOR `borrow_mut`.
                #[inline]
                pub unsafe fn borrow(&self) -> $ref<'_, T> {
                    //  SAFETY:
                    //  We only hand out references to data, so most preconditions for converting
                    //  a pointer to a reference are met (non-null, aligned, valid `T`, dereferencable)
                    //  The caller must ensure that Rust's reference aliasing rules are observed.
                    $ref(unsafe { &*self.0.get() })
                }
                /// # Safety
                ///
                /// Safe with `debug_assertions` or `full_safety` feature enabled.
                ///
                /// Otherwise, the caller must ensure Rust's aliasing rules are observed:
                /// `borrow` XOR `borrow_mut`.
                #[inline]
                pub unsafe fn borrow_mut(&self) -> $ref_mut<'_, T> {
                    //  SAFETY:
                    //  We only hand out references to data, so most preconditions for converting
                    //  a pointer to a reference are met (non-null, aligned, valid `T`, dereferencable)
                    //  The caller must ensure that Rust's reference aliasing rules are observed.
                    $ref_mut(unsafe { &mut *self.0.get() })
                }
                #[inline]
                pub fn get_mut(&mut self) -> &mut T {
                    self.0.get_mut()
                }
                #[inline]
                pub fn ptr(&self) -> *const T {
                    self.0.get()
                }
                #[inline]
                pub fn mut_ptr(&self) -> *mut T {
                    self.0.get()
                }
            }

            pub struct $ref<'a, T: ?Sized>(&'a T);
            impl<'a, T: ?Sized> std::ops::Deref for $ref<'a, T> {
                type Target = T;
                #[inline]
                fn deref(&self) -> &Self::Target {
                    self.0
                }
            }

            pub struct $ref_mut<'a, T: ?Sized>(&'a mut T);
            impl<'a, T: ?Sized> std::ops::Deref for $ref_mut<'a, T> {
                type Target = T;
                #[inline]
                fn deref(&self) -> &Self::Target {
                    self.0
                }
            }
            impl<'a, T: ?Sized> std::ops::DerefMut for $ref_mut<'a, T> {
                #[inline]
                fn deref_mut(&mut self) -> &mut Self::Target {
                    self.0
                }
            }
        };
    }

    use std::cell::UnsafeCell;
    gcell!(UnsafeCell<T>, SemiSafeCell, SemiSafeRef, SemiSafeRefMut);

    gcell!(UnsafeCell<T>, SyncSemiSafeCell, SyncSemiSafeRef, SyncSemiSafeRefMut);
    unsafe impl<T: ?Sized + Sync> Sync for SyncSemiSafeCell<T> {}
}
