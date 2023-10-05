mod user;

use std::{borrow::Cow, str::FromStr};

use derive_getters::Getters;
use email_address::EmailAddress;
use secrecy::{Secret, SecretString};
use url::Url;
pub use user::{
    Article as ArticleUser, Authorized as AuthorizedUser, Comment as CommentUser,
    Description as UserDescription, DisplayName as UserDisplayName, Follower as FollowerUser,
    Full as FullUser, Id as UserId, Name as UserName, Short as ShortUser,
    ShortDescription as UserShortDescription, Socials as UserSocials,
};

mod tag;

pub use tag::{
    Article as ArticleTag, Full as FullTag, Id as TagId, Name as TagName, Popular as PopularTag,
    Slug as TagSlug, User as UserTag,
};

mod comment;

pub use comment::{Article as ArticleComment, Id as CommentId, Reply as ReplyComment};

mod article;

pub use article::{
    Author as AuthorArticle, Description as ArticleDescription, Feed as FeedArticle,
    Full as FullArticle, Id as ArticleId, List as ListArticle, Recommended as RecommendedArticle,
    Search as SearchArticle, Short as ShortArticle, Slug as ArticleSlug, Tag as TagArticle,
    Title as ArticleTitle,
};

mod list;

pub use list::{Full as FullList, Id as ListId};

mod bookmark;

pub use bookmark::{Full as FullBookmark, Id as BookmarkId};

macro_rules! id_type {
    {$object_type:literal} => {
        #[derive(Debug, ::derive_more::Into, ::derive_more::AsRef, ::derive_more::Display, ::serde::Deserialize, ::derives::HexId, Clone, PartialEq, Eq, Hash)]
        #[display(fmt = "{}", "self.display_as_hex()")]
        #[serde(transparent)]
        #[doc = concat!("Represents an id of some ", $object_type)]
        pub struct Id(#[serde(with = "::serde_hex::SerHex::<::serde_hex::config::Strict>")] [u8; 12]);
    };
}
pub(self) use id_type;

macro_rules! str_type {
    {$type_name:ident, $object_type:literal, $owner_object:literal} => {
        #[derive(
            Debug,
            ::derive_more::Into,
            ::derive_more::AsRef,
            ::derive_more::Display,
            ::serde::Deserialize,
            Clone,
            PartialEq,
            Eq,
            Hash
        )]
        #[serde(transparent)]
        #[doc = concat!("Represents a ", $object_type, " of some ", $owner_object)]
        pub struct $type_name(String);
    };
}
pub(self) use str_type;

/// Represents user's attitude to some object (other user, tag, article, etc)
#[derive(Debug, serde::Deserialize, derive_getters::Getters, Clone)]
pub struct Relationships {
    #[serde(rename = "isSubscribed")]
    is_subscribed: bool,
    #[serde(rename = "isBlocked")]
    is_blocked: bool,
}

/// Represents user credentials
#[derive(Debug, serde::Deserialize, Getters)]
pub struct Credentials {
    email: EmailAddress,
    password: SecretString,
}

impl Credentials {
    /// Create new credentials object
    pub fn create<'e, 'p>(
        email: impl Into<Cow<'e, str>>,
        password: impl Into<Cow<'p, str>>,
    ) -> Result<Credentials, email_address::Error> {
        Ok(Credentials {
            email: EmailAddress::from_str(&email.into())?,
            password: Secret::new(password.into().to_string()),
        })
    }
}

mod serde_utils {
    use html_parser::Dom;
    use serde::{Deserialize, Deserializer};
    use time::{Duration, OffsetDateTime};

    pub fn duration_from_seconds<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Duration, D::Error> {
        let integer: i64 = Deserialize::deserialize(deserializer)?;
        Ok(Duration::seconds(integer))
    }

    // I have no idea how and why "isLiked" field is represented by a number on a site.
    // This is weird
    pub fn flag_from_number<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
        let num: usize = Deserialize::deserialize(deserializer)?;
        Ok(num > 0)
    }

    pub fn html_from_str<'de, D: ::serde::de::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Dom, D::Error> {
        use ::serde::de::Error;
        let s: String = serde::Deserialize::deserialize(deserializer)?; // FIXME &str results in an error, for some reason
        let s = s.as_str();
        Dom::parse(s).map_err(|_| {
            D::Error::invalid_value(::serde::de::Unexpected::Str(s), &"Valid html fragment")
        })
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn optional_iso_time<'de, D: ::serde::de::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<OffsetDateTime>, D::Error> {
        match Option::<OffsetDateTime>::deserialize(deserializer) {
            Ok(option) => Ok(option),
            Err(_) => Ok(None), // FIXME I guess, that's not the best practice, definitely should redo that
        }
    }
}

/// Url type for all of the requests
///
/// It turns out, users can specify invalid links in their profiles, so this is my way to remedy this.
// TODO investigate
#[derive(Debug, Clone)]
pub enum MaybeUrl {
    /// Valid [`url::Url`]
    Url(Url),
    /// Invalid url. Contains both source string (zeroth field) and error description (first field)
    BadUrl(String, String),
}

impl<'de> serde::Deserialize<'de> for MaybeUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        match s.parse() {
            Ok(url) => Ok(MaybeUrl::Url(url)),
            Err(err) => Ok(MaybeUrl::BadUrl(s, err.to_string())),
        }
    }
}
