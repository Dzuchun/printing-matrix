use core::{borrow::Borrow, hash::Hash, ops::Deref};

use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};

/// An atomic share-on-write primitive
#[derive(derive_more::IsVariant)]
pub enum ASow<'r, T: ?Sized> {
    Reference(&'r T),
    Shared(Arc<T>),
}

impl<T: ?Sized> ASow<'_, T> {
    /// - If `Reference`, clones the value
    /// - If `Reference`, attempts to unwrap the `Arc`
    /// - If `Arc` unwrapping fails, clones the value
    #[must_use]
    pub fn into_owned(self) -> T
    where
        T: Sized + Clone,
    {
        match self {
            ASow::Reference(s) => s.clone(),
            ASow::Shared(s) => match Arc::try_unwrap(s) {
                Ok(v) => v,
                Err(s) => s.deref().clone(),
            },
        }
    }
}

impl<T: ?Sized> AsRef<T> for ASow<'_, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T: ?Sized> Borrow<T> for ASow<'_, T> {
    #[inline]
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: ?Sized> Deref for ASow<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            ASow::Reference(s) => s,
            ASow::Shared(rc) => rc,
        }
    }
}

impl<T: ?Sized> Clone for ASow<'_, T> {
    fn clone(&self) -> Self {
        match self {
            ASow::Reference(r) => ASow::Reference(r),
            ASow::Shared(rc) => ASow::Shared(Arc::clone(rc)),
        }
    }
}

impl<T: Default> Default for ASow<'static, T> {
    fn default() -> Self {
        ASow::Shared(Arc::new(T::default()))
    }
}

macro_rules! impl_display_trait {
    ($t:ident) => {
        impl<T: ?Sized + core::fmt::$t> core::fmt::$t for ASow<'_, T> {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                self.deref().fmt(f)
            }
        }
    };
}
impl_display_trait! {Display}
impl_display_trait! {Debug}
impl_display_trait! {LowerExp}
impl_display_trait! {UpperExp}
impl_display_trait! {LowerHex}
impl_display_trait! {UpperHex}
impl_display_trait! {Octal}
impl_display_trait! {Binary}
impl_display_trait! {Pointer}

impl<T: ?Sized + Hash> Hash for ASow<'_, T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

impl<'r, T> From<&'r T> for ASow<'r, T> {
    fn from(value: &'r T) -> Self {
        ASow::Reference(value)
    }
}

impl From<String> for ASow<'static, str> {
    fn from(value: String) -> Self {
        ASow::Shared(Arc::from(value))
    }
}

impl<T> From<Vec<T>> for ASow<'static, [T]> {
    fn from(value: Vec<T>) -> Self {
        ASow::Shared(Arc::from(value))
    }
}

impl<T: ?Sized> From<Box<T>> for ASow<'static, T> {
    fn from(value: Box<T>) -> Self {
        ASow::Shared(Arc::from(value))
    }
}

impl<T: ?Sized + PartialEq> PartialEq for ASow<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(&**other)
    }
}
impl<T: ?Sized + PartialEq> Eq for ASow<'_, T> {}

impl<T: ?Sized + PartialEq> PartialEq<T> for ASow<'_, T> {
    fn eq(&self, other: &T) -> bool {
        self.deref().eq(other)
    }
}

impl<'r, T: ?Sized + PartialEq> PartialEq<&'r T> for ASow<'_, T> {
    fn eq(&self, other: &&'r T) -> bool {
        self.deref().eq(other)
    }
}

impl<T: ?Sized + PartialOrd> PartialOrd for ASow<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.deref().partial_cmp(&**other)
    }
}
impl<T: ?Sized + Ord> Ord for ASow<'_, T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.deref().cmp(&**other)
    }
}

impl<T: ?Sized + PartialOrd> PartialOrd<T> for ASow<'_, T> {
    fn partial_cmp(&self, other: &T) -> Option<core::cmp::Ordering> {
        self.deref().partial_cmp(other)
    }
}

impl<'r, T: ?Sized + PartialOrd> PartialOrd<&'r T> for ASow<'_, T> {
    fn partial_cmp(&self, other: &&'r T) -> Option<core::cmp::Ordering> {
        self.deref().partial_cmp(other)
    }
}

// TODO: probably add eq+ord for String, Vec and Box
