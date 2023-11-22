use std::{convert::Infallible, str::FromStr};

use derives::data_type;

super::id_type! {"article"}

super::str_type! {Title, "title", "article"}

impl FromStr for Title {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO perform actual validation
        Ok(Self(s.to_owned()))
    }
}

super::str_type! {SeoTitle, "seo title", "article"}

impl FromStr for SeoTitle {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO perform actual validation
        Ok(Self(s.to_owned()))
    }
}

super::str_type! {Description, "description", "article"}

impl FromStr for Description {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO perform actual validation
        Ok(Self(s.to_owned()))
    }
}

super::str_type! {Slug, "slug", "article"}

impl FromStr for Slug {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO perform actual validation
        Ok(Self(s.to_owned()))
    }
}

data_type!(
    Search,
    id,
    title,
    description,
    slug,
    owner_id,
    thumb_picture,
    picture,
    main_tag_name,
    main_tag_id,
    read_time,
    canonical,
    main_tag_slug,
    created_at,
    is_bookmarked,
    unused_pin_created_at,
);

data_type!(
    Author,
    id,
    title,
    description,
    slug,
    thumb_picture,
    picture,
    main_tag_name,
    main_tag_id,
    main_tag_slug,
    tag_ids,
    owner_id,
    read_time,
    canonical,
    is_bookmarked,
    created_at,
    like_num,
    comment_num,
    sensitive,
    unused_pin_created_at,
);

data_type! {
    Recommended,
    id,
    title,
    description,
    slug,
    main_tag_name,
    main_tag_slug,
    main_tag_id,
    tag_ids,
    sensitive,
    canonical,
    like_num,
    comment_num,
    read_time,
    created_at,
    thumb_picture,
    owner_article,
    is_bookmarked,
}

data_type! {
    Short,
    id,
    title,
    description,
    slug,
    owner_id,
    thumb_picture,
    main_tag_name,
    main_tag_slug,
    main_tag_id,
    tag_ids,
    sensitive,
    like_num,
    comment_num,
    read_time,
    created_at,
    is_bookmarked,
}

data_type! {
    Tag,
    id,
    title,
    description,
    slug,
    thumb_picture,
    main_tag_name,
    main_tag_slug,
    main_tag_id,
    tag_ids,
    sensitive,
    canonical,
    like_num,
    comment_num,
    read_time,
    owner_article,
    is_bookmarked,
    created_at,
    relationships
}

data_type! {
    List,
    id,
    title,
    description,
    slug,
    main_tag_name,
    main_tag_slug,
    main_tag_id,
    read_time,
    created_at,
    is_bookmarked,
}

data_type! {
    Feed,
    id,
    title,
    description,
    slug,
    thumb_picture,
    main_tag_name,
    main_tag_id,
    main_tag_slug,
    tag_users,
    sensitive,
    like_num,
    comment_num,
    read_time,
    created_at,
    is_bookmarked,
    owner_comment,
}

data_type! {
    Full,
    id,
    title,
    seo_title,
    description,
    slug,
    picture,
    thumb_picture,
    main_tag_name,
    main_tag_id,
    main_tag_slug,
    tag_articles,
    ads,
    index,
    sensitive,
    canonical,
    like_num,
    comment_num,
    is_liked,
    read_time,
    created_at,
    is_bookmarked,
    owner_article,
    relationships,
    author_articles,
    recommended_articles,
    comments,
    content,
}
