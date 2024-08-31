use alloc::string::String;
use core::num::NonZeroU64;

pub mod id;

pub mod slug;

#[cfg(feature = "maybe-url")]
pub mod maybe_url;

/// Name of the article tag
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(transparent)
)]
pub struct TagName(pub alloc::string::String);

/// Number of mentions of specific tag
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(transparent)
)]
pub struct TagMentions(pub u64);

/// Search page index, i.e. how far you are into the search
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(transparent)
)]
pub struct PageIndex(pub core::num::NonZeroU64);

impl PageIndex {
    /// Returns next page index
    ///
    /// ### Panics
    /// If page index reaches `u64::MAX`
    pub fn next(self) -> PageIndex {
        PageIndex(
            self.0
                .checked_add(1)
                .expect("Reached u64 limit for number of pages"),
        )
    }

    /// Creates page index from anything that can be converted into `NonZeroU64`
    pub fn from(v: impl Into<NonZeroU64>) -> Self {
        Self(v.into())
    }

    /// Attempts to create page index from anything that can attempt to create `NonZeroU64`
    pub fn try_from<T: TryInto<NonZeroU64>>(v: T) -> Result<Self, T::Error> {
        Ok(Self(v.try_into()?))
    }
}

// SAFETY: 1 is crearly not a zero
//
// Basides - passing in 0 here results in `unreachable` call, breaking compilation in the process
const ONE: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(1) };
pub const FIRST_PAGE: PageIndex = PageIndex(ONE);

/// Represents if user had subscribed and/or blocked something (other user, tag, article, etc)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct Relationships {
    #[cfg_attr(
        any(feature = "serialize", feature = "deserialize"),
        serde(rename = "isSubscribed")
    )]
    pub is_subscribed: bool,
    #[cfg_attr(
        any(feature = "serialize", feature = "deserialize"),
        serde(rename = "isBlocked")
    )]
    pub is_blocked: bool,
}

/// Username of some user
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(transparent)
)]
pub struct Username(pub String);

/// Display name of some user
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(transparent)
)]
pub struct DisplayName(pub String);

/// Avatar of some user
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(transparent)
)]
#[cfg(feature = "maybe-url")]
pub struct Avatar(pub maybe_url::MaybeUrl);
