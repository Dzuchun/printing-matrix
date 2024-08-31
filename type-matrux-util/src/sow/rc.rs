use core::{borrow::Borrow, hash::Hash, ops::Deref};

use alloc::{boxed::Box, rc::Rc, string::String, vec::Vec};

/// A share-on-write primitive
#[derive(derive_more::IsVariant)]
pub enum Sow<'r, T: ?Sized> {
    Reference(&'r T),
    Shared(Rc<T>),
}

impl<T: ?Sized> Sow<'_, T> {
    /// - If `Reference`, clones the value
    /// - If `Reference`, attempts to unwrap the `Rc`
    /// - If `Rc` unwrapping fails, clones the value
    #[must_use]
    pub fn into_owned(self) -> T
    where
        T: Sized + Clone,
    {
        match self {
            Sow::Reference(s) => s.clone(),
            Sow::Shared(s) => match Rc::try_unwrap(s) {
                Ok(v) => v,
                Err(s) => s.deref().clone(),
            },
        }
    }
}

impl<T: ?Sized> AsRef<T> for Sow<'_, T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T: ?Sized> Borrow<T> for Sow<'_, T> {
    #[inline]
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: ?Sized> Deref for Sow<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Sow::Reference(s) => s,
            Sow::Shared(rc) => rc,
        }
    }
}

impl<T: ?Sized> Clone for Sow<'_, T> {
    fn clone(&self) -> Self {
        match self {
            Sow::Reference(r) => Sow::Reference(r),
            Sow::Shared(rc) => Sow::Shared(Rc::clone(rc)),
        }
    }
}

impl<T: Default> Default for Sow<'static, T> {
    fn default() -> Self {
        Sow::Shared(Rc::new(T::default()))
    }
}

macro_rules! impl_display_trait {
    ($t:ident) => {
        impl<T: ?Sized + core::fmt::$t> core::fmt::$t for Sow<'_, T> {
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

impl<T: ?Sized + Hash> Hash for Sow<'_, T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

impl<'r, T> From<&'r T> for Sow<'r, T> {
    fn from(value: &'r T) -> Self {
        Sow::Reference(value)
    }
}

impl From<String> for Sow<'static, str> {
    fn from(value: String) -> Self {
        Sow::Shared(Rc::from(value))
    }
}

impl<T> From<Vec<T>> for Sow<'static, [T]> {
    fn from(value: Vec<T>) -> Self {
        Sow::Shared(Rc::from(value))
    }
}

impl<T: ?Sized> From<Box<T>> for Sow<'static, T> {
    fn from(value: Box<T>) -> Self {
        Sow::Shared(Rc::from(value))
    }
}

impl<T: ?Sized + PartialEq> PartialEq for Sow<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(&**other)
    }
}
impl<T: ?Sized + PartialEq> Eq for Sow<'_, T> {}

impl<T: ?Sized + PartialEq> PartialEq<T> for Sow<'_, T> {
    fn eq(&self, other: &T) -> bool {
        self.deref().eq(other)
    }
}

impl<'r, T: ?Sized + PartialEq> PartialEq<&'r T> for Sow<'_, T> {
    fn eq(&self, other: &&'r T) -> bool {
        self.deref().eq(other)
    }
}

impl<T: ?Sized + PartialOrd> PartialOrd for Sow<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.deref().partial_cmp(&**other)
    }
}
impl<T: ?Sized + Ord> Ord for Sow<'_, T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.deref().cmp(&**other)
    }
}

impl<T: ?Sized + PartialOrd> PartialOrd<T> for Sow<'_, T> {
    fn partial_cmp(&self, other: &T) -> Option<core::cmp::Ordering> {
        self.deref().partial_cmp(other)
    }
}

impl<'r, T: ?Sized + PartialOrd> PartialOrd<&'r T> for Sow<'_, T> {
    fn partial_cmp(&self, other: &&'r T) -> Option<core::cmp::Ordering> {
        self.deref().partial_cmp(other)
    }
}

// TODO: probably add eq+ord for String, Vec and Box
