use derives::data_type;

super::id_type! {"user list"}

super::str_type! {Name, "name", "list"}

data_type! {
    Full,
    id,
    name,
    articles_num,
    owner_id
}
