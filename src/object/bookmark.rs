use derives::data_type;

super::id_type! {"bookmark"}

data_type! {
    Full,
    id,
    article_id,
    owner_id,
    list_id,
    article_name,
    created_at,
}
