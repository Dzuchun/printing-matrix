use core::mem::discriminant;
use core::{fmt::Debug, hash::Hash};

use alloc::string::String;
use url::{ParseError, Url};

fn parse_error_tag(err: ParseError) -> u8 {
    match err {
        ParseError::EmptyHost => 0,
        ParseError::IdnaError => 1,
        ParseError::InvalidPort => 2,
        ParseError::InvalidIpv4Address => 3,
        ParseError::InvalidIpv6Address => 4,
        ParseError::InvalidDomainCharacter => 5,
        ParseError::RelativeUrlWithoutBase => 6,
        ParseError::RelativeUrlWithCannotBeABaseBase => 7,
        ParseError::SetHostOnCannotBeABaseUrl => 8,
        ParseError::Overflow => 9,
        _ => 10,
    }
}

#[derive(Clone, Copy)]
struct ParseErrorWrapper(ParseError);

impl Debug for ParseErrorWrapper {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl Hash for ParseErrorWrapper {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        discriminant(&self.0).hash(state);
    }
}

impl PartialEq for ParseErrorWrapper {
    fn eq(&self, other: &Self) -> bool {
        parse_error_tag(self.0) == parse_error_tag(other.0)
    }
}
// I don't want to discriminate anything other than the tag
impl Eq for ParseErrorWrapper {}

impl PartialOrd for ParseErrorWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ParseErrorWrapper {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        parse_error_tag(self.0).cmp(&parse_error_tag(other.0))
    }
}

/// Invalid url, containing original string (`source`) and error that happened while trying to
/// parse it (`error()`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "serialize", serde(transparent))]
pub struct BadUrl {
    pub source: String,
    #[cfg_attr(feature = "serialize", serde(skip))]
    error_inner: ParseErrorWrapper,
}

impl BadUrl {
    pub fn error(&self) -> ParseError {
        self.error_inner.0
    }
}

/// (supposedly) a url originating from user content
///
/// Apparently, users can specify invalid urls in their profile
// TODO investigate - try querying a lot of users and finding some sort of pattern (idk)
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::IsVariant,
    derive_more::Unwrap,
    derive_more::TryUnwrap,
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub enum MaybeUrl {
    Url(Url),
    BadUrl(BadUrl),
}

#[cfg(feature = "deserialize")]
impl<'de> serde::Deserialize<'de> for MaybeUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let source: String = serde::Deserialize::deserialize(deserializer)?;
        match source.parse() {
            Ok(url) => Ok(MaybeUrl::Url(url)),
            Err(error) => Ok(MaybeUrl::BadUrl(BadUrl {
                source,
                error_inner: ParseErrorWrapper(error),
            })),
        }
    }
}

impl AsRef<str> for MaybeUrl {
    fn as_ref(&self) -> &str {
        match self {
            MaybeUrl::Url(url) => url.as_str(),
            MaybeUrl::BadUrl(bad_url) => bad_url.source.as_str(),
        }
    }
}
