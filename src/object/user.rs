use std::{collections::HashMap, convert::Infallible, str::FromStr};

use derive_more::{AsRef, Into};
use derives::data_type;
use secrecy::SecretString;
use serde::Deserialize;

use super::MaybeUrl;

super::id_type! {"user"}

super::str_type! {DisplayName, "display name", "user"}

impl FromStr for DisplayName {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO perform actual validation
        Ok(Self(s.to_owned()))
    }
}

super::str_type! {Name, "username", "user"}

impl FromStr for Name {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO perform actual validation
        Ok(Self(s.to_owned()))
    }
}

super::str_type! {ShortDescription, "short description", "user"}

super::str_type! {Description, "description", "user"}

/// User's social links, like telegram and facebook
#[derive(Debug, Into, AsRef, Deserialize, Default, Clone)]
#[serde(transparent)]
pub struct Socials(HashMap<String, MaybeUrl>);

data_type! {
    Short,
    id,
    username,
    display_name,
    avatar,
    relationships
}

data_type! {
    Comment,
    id,
    username,
    display_name,
    avatar
}

data_type! {
    Follower,
    opt_id,
    avatar,
    opt_username,
    opt_display_name,
    short_description,
    relationships,
}

data_type! {
    Authorized,
    id,
    username,
    avatar,
    short_description,
    description,
    following_num,
    followers_num,
    email,
    read_num,
    first_published_at,
    // seems like it is actually present, but only on registering
    // #[serde(rename = "articlesNum")]
    // articles_num: usize,
    author_tags,
    // TODO is it absent, really?
    // #[serde(rename = "createdAt", with = "time::serde::iso8601")]
    // created_at: OffsetDateTime,
    notifications_num,
    socials,
    unused_facebook_id,
    unused_google_id,
    unused_password,
    unused___v,
}

data_type! {
    Article,
    id,
    display_name,
    avatar,
    short_description,
    following_num,
    followers_num,
    read_num,
    username,
    created_at,
    socials,
    donate_url,
}

data_type! {
    Full,
    id,
    display_name,
    avatar,
    username,
    short_description,
    user_description,
    following_num,
    followers_num,
    read_num,
    author_tags,
    created_at,
    socials,
    donate_url,
    relationships,
    user_articles,
}
