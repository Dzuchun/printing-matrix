use derives::data_type;

super::id_type! {"article comment"}

data_type! {
    Article,
    id,
    comment_dom,
    maybe_comment_owner,
    article_id,
    hidden_by_author,
    reply_num,
    likes_num,
    created_at,
    is_liked_bool,
    is_blocked,
    unused___v,
}

data_type! {
    Reply,
    id,
    comment_dom,
    owner_comment,
    article_id,
    hidden_by_author,
    reply_num,
    likes_num,
    created_at,
    is_liked_bool,
    is_blocked,
    reply_to_comment,
    reply_to_user,
    root_comment,
    root_comment_owner,
    unused___v,
}
