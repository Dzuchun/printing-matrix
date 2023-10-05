use std::{convert::Infallible, str::FromStr};

use derives::data_type;

super::id_type! {"tag"}

super::str_type! {Name, "name", "tag"}

impl FromStr for Name {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO perform actual validation
        Ok(Self(s.to_owned()))
    }
}

super::str_type! {Slug, "slug", "tag"}

impl FromStr for Slug {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO perform actual validation
        Ok(Self(s.to_owned()))
    }
}

data_type! {
    Article,
    id,
    name,
    slug,
    created_at,
    default,
    ignore,
    mentions_num,
    unused___v,
    unused_general
}

data_type! {
    User,
    id,
    name,
    slug,
}

data_type! {
    Popular,
    id,
    name,
    slug,
    mentions_num,
    unused___v,
}

data_type! {
    Full,
    id,
    name,
    slug,
    mentions_num,
    relationships,
    article_tags,
}
