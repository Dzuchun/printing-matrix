/// Slug of something. Internally just a string with specific guarantees.
///
/// Other [Slug]s you see all over the place internally just contain this one.
///
/// All of the [Slug]s are *deliberately* immutable, to avoid breaking slug formatting.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(transparent)
)]
// TODO: Add validation with serde
pub struct Slug(alloc::string::String);

impl AsRef<str> for Slug {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl core::fmt::Display for Slug {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// TODO: Add `FromStr` and `TryFrom` implementations

macro_rules! slug {
    ($name:ident, $entity:literal) => {
        paste::paste! {
            #[derive(
                Debug,
                ::derive_more::AsRef,
                Clone,
                PartialEq,
                Eq,
                PartialOrd,
                Ord,
                Hash,
            )]
            #[cfg_attr(feature = "serialize", derive(serde::Serialize))]
            #[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
            #[cfg_attr(
                any(feature = "serialize", feature = "deserialize"),
                serde(transparent)
            )]
            #[doc = "Slug of " $entity ". Please refer to [Slug] for more info"]
            pub struct [<$name Slug>](Slug);

            impl [<$name Slug>] {
                pub fn into_slug(self) -> Slug {
                    self.0
                }
            }

            impl core::convert::AsRef<str> for [<$name Slug>] {
                fn as_ref(&self) -> &str {
                    self.0.0.as_str()
                }
            }

            impl core::fmt::Display for [<$name Slug>] {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f, "{}", self.0.0)
                }
            }

            // TODO: Add `FromStr` and `TryFrom` implementations
        }
    };
}

slug! {Tag, "article tag"}
