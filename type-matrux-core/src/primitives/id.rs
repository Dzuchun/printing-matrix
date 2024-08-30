/// Identifier of something
///
/// Other [Id]s you see all over the API internally hold this very struct
///
/// All of the [Id]s are *deliberately* immutable, as an attempt to have an invariant of each `Id`
/// corresponding to existing entity on the site.
#[derive(Debug, ::derive_more::AsRef, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(transparent)
)]
pub struct Id([u8; 12]);

impl core::fmt::UpperHex for Id {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for b in self.0 {
            write!(f, "{b:2X}")?;
        }
        Ok(())
    }
}

impl core::fmt::LowerHex for Id {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for b in self.0 {
            write!(f, "{b:2x}")?;
        }
        Ok(())
    }
}

macro_rules! id {
    ($name:ident, $entity:literal) => {
        paste::paste! {
            #[derive(
                Debug,
                ::derive_more::AsRef,
                Clone,
                Copy,
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
            #[doc = "Identifier of " $entity ". Please refer to [Id] for more info"]
            pub struct [<$name Id>](Id);

            impl [<$name Id>] {
                pub fn into_id(self) -> Id {
                    self.0
                }
            }

            impl core::convert::AsRef<[u8; 12]> for [<$name Id>] {
                fn as_ref(&self) -> &[u8; 12] {
                    &self.0.0
                }
            }

            impl core::fmt::UpperHex for [<$name Id>] {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f, "{:X}", self.0)
                }
            }

            impl core::fmt::LowerHex for [<$name Id>] {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f, "{:x}", self.0)
                }
            }
        }
    };
}

id! {Tag, "article tag"}
