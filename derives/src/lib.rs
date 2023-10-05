extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::Ident;
use syn::{punctuated::Punctuated, Token};

/// Derives useful functions for aged objects
///
/// # Panics
/// if the type in question is not valid
#[proc_macro_derive(Aged)]
pub fn aged_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_aged_macro(&ast)
}

fn impl_aged_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    quote! {
      impl #name {
        /// Provides a way to know a `time::Duration` since the object was fetched
        ///
        /// May be used to, for example, re-fetch it once it gets too old, and thus probably outdated
        pub fn get_age(&self) -> ::time::Duration {
          ::time::OffsetDateTime::now_utc() - self.fetched_at
        }
      }
    }
    .into()
}

/// Derives useful functions for hex ids
///
/// # Panics
/// if the type in question is not a tuple with first element being a 12-byte array
#[proc_macro_derive(HexId)]
pub fn hex_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    // Build the trait implementation
    let name = &ast.ident;
    quote! {
      impl #name {
        /// Displays id as a hex string
        fn display_as_hex(&self) -> String {
            let mut res = String::with_capacity(24);
            for b in self.0 {
                // if byte's value is less than 16, this string will be only 1 character long
                let s = if b < 16u8 {
                    format!("0{:x}", b)
                } else {
                    format!("{:x}", b)
                };
                res.push_str(&s);
            }
            res
        }
      }
    }
    .into()
}

#[proc_macro]
pub fn data_type(input: TokenStream) -> TokenStream {
    let parser = Punctuated::<syn::Ident, Token![,]>::parse_terminated;
    let identifiers = parser
        .parse(input)
        .expect("Macro input should be a list of identifiers");
    let mut identifiers = identifiers.into_iter();
    let name = identifiers
        .next()
        .expect("At least one identifier is required");
    let fields: proc_macro2::TokenStream = identifiers.map(data_field).collect();
    quote! {
        #[derive(Debug, ::serde::Deserialize, ::derive_getters::Getters, ::derives::Aged, Clone)]
        #[serde(deny_unknown_fields)]
        pub struct #name {
            #fields
            #[serde(skip, default = "::time::OffsetDateTime::now_utc")]
            #[getter(skip)]
            fetched_at: ::time::OffsetDateTime,
        }
    }
    .into()
}

#[allow(clippy::too_many_lines)]
fn data_field(name: Ident) -> proc_macro2::TokenStream {
    let s = name.to_string();
    match s.as_str() {
        // cave story madness
        "id" => quote! {
            #[serde(rename = "_id")]
            id: Id,
        },
        "opt_id" => quote! {
            #[serde(rename = "_id", default)]
            id: Option<Id>,
        },
        "article_id" => quote! {
            article: super::ArticleId,
        },
        "list_id" => quote! {
            list: super::ListId,
        },
        "title" => quote! {
            title: Title,
        },
        "seo_title" => quote! {
            #[serde(rename = "seoTitle")]
            seo_title: SeoTitle,
        },
        "name" => quote! {
            name: Name,
        },
        "article_name" => quote! {
            name: super::ArticleTitle,
        },
        "description" => quote! {
            description: Description,
        },
        "user_description" => quote! {
            description: Option<Description>,
        },
        "user_articles" => quote! {
            articles: Vec<super::AuthorArticle>,
        },
        "short_description" => quote! {
            #[serde(rename = "descriptionShort")]
            short_description: Option<ShortDescription>,
        },
        "slug" => quote! {
            slug: Slug,
        },
        "main_tag_name" => quote! {
            #[serde(rename = "mainTag")]
            main_tag: super::TagName,
        },
        "main_tag_id" => quote! {
            #[serde(rename = "mainTagId")]
            main_tag_id: super::TagId,
        },
        "main_tag_slug" => quote! {
            #[serde(rename = "mainTagSlug")]
            main_tag_slug: super::TagSlug,
        },
        "thumb_picture" => quote! {
            #[serde(rename = "thumbPicture")]
            thumb_picture: Option<super::MaybeUrl>,
        },
        "picture" => quote! {
            picture: Option<super::MaybeUrl>,
        },
        "owner_id" => quote! {
            owner: super::UserId,
        },
        "owner_article" => quote! {
            owner: super::ArticleUser,
        },
        "owner_comment" => quote! {
            owner: super::CommentUser,
        },
        "maybe_comment_owner" => quote! {
            owner: Option<super::CommentUser>, // TODO check for that.
        },
        "is_bookmarked" => quote! {
            #[serde(rename = "isBookmarked")]
            is_bookmarked: bool,
        },
        "read_time" => quote! {
            #[serde(
                rename = "readTime",
                deserialize_with = "super::serde_utils::duration_from_seconds"
            )]
            read_time: ::time::Duration,
        },
        "created_at" => quote! {
            #[serde(rename = "createdAt", with = "time::serde::iso8601")]
            created_at: ::time::OffsetDateTime,
        },
        "tag_ids" => quote! {
            tags: Vec<super::TagId>,
        },
        "tag_users" => quote! {
            tags: Vec<super::UserTag>,
        },
        "tag_articles" => quote! {
            tags: Vec<super::ArticleTag>,
        },
        "like_num" => quote! {
            #[serde(rename = "likeNum")]
            like_num: usize,
        },
        "likes_num" => quote! {
            #[serde(rename = "likesNum")]
            likes_num: usize, // yes, really
        },
        "comment_num" => quote! {
            #[serde(rename = "commentNum")]
            comment_num: usize,
        },
        "comment_dom" => quote! {
            #[serde(deserialize_with = "super::serde_utils::html_from_str")]
            comment: ::html_parser::Dom,
        },
        "comments" => quote! {
            comments: Vec<super::ArticleComment>,
        },
        "reply_num" => quote! {
            #[serde(rename = "replyNum")]
            reply_num: usize,
        },
        "articles_num" => quote! {
            #[serde(rename = "articlesNum")]
            articles_num: usize,
        },
        "mentions_num" => quote! {
            #[serde(rename = "mentionsNum")]
            mentions_num: usize,
        },
        "sensitive" => quote! {
            sensitive: bool,
        },
        "relationships" => quote! {
            relationships: super::Relationships,
        },
        "ads" => quote! {
            ads: Option<bool>, // TODO check if it's really optional
        },
        "index" => quote! {
            index: Option<bool>,
        },
        "is_liked" => quote! {
            #[serde(
                rename = "isLiked",
                deserialize_with = "super::serde_utils::flag_from_number"
            )]
            is_liked: bool,
        },
        "is_liked_bool" => quote! {
            #[serde(rename = "isLiked")]
            is_liked: bool,
        },
        "is_blocked" => quote! {
            #[serde(rename = "isBlocked")]
            is_blocked: bool,
        },
        "hidden_by_author" => quote! {
            #[serde(rename = "hiddenByAuthor")]
            hidden_by_author: bool,
        },
        "author_articles" => quote! {
            #[serde(rename = "authorArticles")]
            author_articles: Vec<super::SearchArticle>,
        },
        "recommended_articles" => quote! {
            #[serde(rename = "recommendedArticles")]
            recommended_articles: Vec<super::RecommendedArticle>,
        },
        "article_tags" => quote! {
            articles: Vec<super::TagArticle>,
        },
        "content" => quote! {
            content: ::serde_json::Value, // TODO perform proper content typing
        },
        "reply_to_comment" => quote! {
            #[serde(rename = "replyToComment")]
            reply_to_comment: Id,
        },
        "reply_to_user" => quote! {
            #[serde(rename = "replyToUser")]
            reply_to_user: super::UserId,
        },
        "root_comment" => quote! {
            #[serde(rename = "rootComment")]
            root_comment: Id,
        },
        "root_comment_owner" => quote! {
            #[serde(rename = "rootCommentOwner")]
            root_comment_owner: super::UserId,
        },
        "default" => quote! {
            #[serde(default)]
            default: bool,
        },
        "ignore" => quote! {
            #[serde(default)]
            ignore: bool,
        },
        "username" => quote! {
            username: super::UserName,
        },
        "opt_username" => quote! {
            #[serde(default)]
            username: Option<super::UserName>,
        },
        "display_name" => quote! {
            name: super::UserDisplayName,
        },
        "opt_display_name" => quote! {
            #[serde(default)]
            name: Option<super::UserDisplayName>,
        },
        "following_num" => quote! {
            #[serde(rename = "followingNum")]
            following_num: usize,
        },
        "followers_num" => quote! {
            #[serde(rename = "followersNum")]
            followers_num: usize,
        },
        "email" => quote! {
            email: String, // TODO check that
        },
        "read_num" => quote! {
            #[serde(rename = "readNum")]
            read_num: usize,
        },
        "first_published_at" => quote! {
            #[serde(rename = "firstPublishedAt")]
            first_published_at: Option<::time::OffsetDateTime>,
        },
        "author_tags" => quote! {
            #[serde(rename = "authorTags")]
            author_tags: Vec<super::UserTag>,
        },
        "notifications_num" => quote! {
            #[serde(rename = "notificationsNum")]
            notifications_num: usize,
        },
        "socials" => quote! {
            #[serde(default)]
            socials: Socials,
        },
        "avatar" => quote! {
            #[serde(default)]
            avatar: Option<super::MaybeUrl>,
        },
        "donate_url" => quote! {
            #[serde(rename = "donateUrl", default)]
            donate_url: Option<super::MaybeUrl>,
        },
        "unused_pin_created_at" => quote! {
            #[serde(
                skip_serializing,
                default,
                rename = "pinCreatedAt",
                deserialize_with = "super::serde_utils::optional_iso_time"
            )]
            #[getter(skip)]
            #[allow(dead_code)]
            pin_created_at: Option<::time::OffsetDateTime>, // TODO unused
        },
        "unused___v" => quote! {
            #[serde(skip_serializing)]
            #[getter(skip)]
            #[allow(dead_code)]
            __v: usize, // TODO unused
        },
        "unused_general" => quote! {
            #[serde(skip_serializing)]
            #[getter(skip)]
            #[allow(dead_code)]
            general: Option<bool>, // TODO unused
        },
        "unused_facebook_id" => quote! {
            #[getter(skip)]
            #[allow(dead_code)]
            facebook_id: Option<String>, // TODO unused
        },
        "unused_google_id" => quote! {
            #[getter(skip)]
            #[allow(dead_code)]
            google_id: Option<String>, // TODO unused
        },
        "unused_password" => quote! {
            #[serde(skip_serializing)]
            #[getter(skip)]
            #[allow(dead_code)]
            password: Option<SecretString>, // TODO unused
        },
        other => panic!("Unknown field: {other}"),
    }
}
