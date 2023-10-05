mod utils;

mod impls;

pub use impls::reqwest::Auth as ReqwestAuth;
use lazy_static::lazy_static;
pub use reqwest::Client as ReqwestApi;

use std::{num::NonZeroUsize, ops::Deref};

use async_trait::async_trait;
use thiserror::Error;
use url::Url;

use crate::object::{
    ArticleId, ArticleSlug, ArticleTitle, AuthorizedUser, CommentId, Credentials, FeedArticle,
    FollowerUser, FullArticle, FullBookmark, FullList, FullTag, FullUser, ListArticle, ListId,
    PopularTag, RecommendedArticle, ReplyComment, ShortUser, TagSlug, UserId, UserName,
};

use self::utils::PageSearchStream;

/// That's [DrukarniaApi]'s error type.
///
/// Variant list might grow in the future, as I implement more features.
#[derive(Debug, Error)]
#[repr(u8)]
#[non_exhaustive]
pub enum Error {
    /// An error caused by HTTP request execution.
    ///
    /// I see no way to categorize these, there's just to much fail reasons here.
    ///
    /// Under normal operation, this sort of error should not occur.
    #[error(transparent)]
    OnExecution(Box<dyn std::error::Error>),
    /// An error happened at response JSON deserializing.
    ///
    /// If you see this sort of error pop up, this is most likely due to Drukarnia API has changed.
    ///
    /// Under normal operation, this sort of error should not occur.
    #[error(
        "JSON deserializing has failed, this is most likely a bug. Please check out issue tracker.\nExplanation: {},\nCause: {}", .0, .1
    )]
    BadJson(serde_json::Error, String),
    /// Server did not return auth token or it was not found.
    ///
    /// This might be a server's fault, an API change or bad credentials.
    #[error("Server did not return auth token, or it was not found")]
    NoToken,
    /// Supplied email and/or password are not correct.
    #[error("Supplied credentials are not correct")]
    BadCredentials,
    /// Queried object (user, article, tag, etc) does not exist.
    #[error("Queried object (user, article, tag, etc) does not exist")]
    NoObject,
}

type Res<T = ()> = Result<T, Error>;

lazy_static! {
    static ref DEFAULT_BASE_URL: Url =
        Url::parse("https://drukarnia.com.ua/").expect("Should be able to parse base url");
}

/// Represents object's ability to make requests to Drukarnia's API.
#[async_trait]
pub trait DrukarniaApi {
    /// Defines a type of this accessor's authenticated version.
    ///
    /// Might even be `Self`, but this is not desired, as authenticated API accessor is kinda different from a regular one.
    type Auth;

    /// Returns base url accessor uses to call DrukarniaAPI.
    ///
    /// `https://drukarnia.com.ua/` by default.
    fn base_url(&self) -> &Url {
        &DEFAULT_BASE_URL
    }

    /// Retrieves currently popular tags at Drukarnia.
    ///
    /// # Implementation
    /// Expected to GET `/api/articles/tags/popular`.
    async fn popular_tags(&self) -> Res<Vec<PopularTag>>;

    /// Retrieves a certain user.
    ///
    /// # Implementation
    /// Expected to GET `/api/users/profile/USER_NAME`
    ///
    /// # Errors
    /// - [`Error::NoObject`]: user with provided name does not exist
    async fn get_user(&self, name: &UserName) -> Res<FullUser>;

    /// Searches a user by it's name.
    ///
    /// # Implementation
    /// Expected to GET `/api/users/info?name=USER_NAME&page=PAGE&withRelationships=true`
    async fn search_user_page(&self, name: &UserName, page: NonZeroUsize) -> Res<Vec<ShortUser>>;

    /// Searches user by it's name.
    ///
    /// # Returns
    /// A stream of search result pages.
    ///
    /// # Implementation details
    /// This function should not be reimplemented.
    ///
    /// Currently, the underlying stream consequently calls for search result pages,
    /// although this might be changed in the future for more optimized approach.
    ///
    /// # Note
    /// Stream ends after first error, since after the error, there's no way for stream to determine, if search results had ended
    fn search_user(&self, name: UserName) -> PageSearchStream<Self::Auth, ShortUser>
    where
        Self: Sized,
    {
        PageSearchStream::create(self, move |page| {
            let name_ = name.clone();
            Box::pin(async move { self.search_user_page(&name_, page).await })
        })
    }

    /// Fetches a tag by it's slug.
    ///
    /// # Implementation
    /// Expected to GET `/api/articles/tags/TAG_SLUG`
    ///
    /// # Errors
    /// [`Error::NoObject`]: tag with provided slug does not exists
    async fn get_tag(&self, slug: &TagSlug) -> Res<FullTag>;

    /// Fetches an article by it's slug.
    ///
    /// # Implementation
    /// Expected to GET `/api/articles/ARTICLE_SLUG`.
    async fn get_article(&self, slug: &ArticleSlug) -> Res<FullArticle>;

    /// Searches an article by it's title.
    ///
    /// # Implementation
    /// Expected to GET `/api/articles/search?name=ARTICLE_TITLE&page=PAGE`
    async fn search_article_page(
        &self,
        name: &ArticleTitle,
        page: NonZeroUsize,
    ) -> Res<Vec<RecommendedArticle>>;

    /// Searches an article by it's title.
    ///
    /// # Returns
    /// A stream of search result pages.
    ///
    /// # Implementation details
    /// This function should not be reimplemented.
    ///
    /// Currently, the underlying stream consequently calls for search result pages,
    /// although this might be changed in the future for more optimized approach.
    ///
    /// # Note
    /// Stream ends after first error, since after the error, there's no way for stream to determine, if search results had ended
    fn search_article(&self, name: ArticleTitle) -> PageSearchStream<Self::Auth, RecommendedArticle>
    where
        Self: Sized,
    {
        PageSearchStream::create(self, move |page| {
            let name_ = name.clone();
            Box::pin(async move { self.search_article_page(&name_, page).await })
        })
    }

    /// Get followers of a user by it's id.
    ///
    /// # Implementation
    /// Expected to GET `/api/relationships/USER_ID/followers?page=PAGE`
    ///
    /// # Errors
    /// - [Error::NoObject]: User with provided id does not exist
    async fn get_followers_page(&self, id: &UserId, page: NonZeroUsize) -> Res<Vec<FollowerUser>>;

    /// Get followers of a user by it's id.
    ///
    /// # Returns
    /// A stream of result pages.
    ///
    /// # Implementation details
    /// This function should not be reimplemented.
    ///
    /// Currently, the underlying stream consequently calls for result pages,
    /// although this might be changed in the future for more optimized approach.
    ///
    /// # Note
    /// Stream ends after first error, since after the error, there's no way for stream to determine, if results had ended
    fn get_followers(&self, id: UserId) -> PageSearchStream<Self::Auth, FollowerUser>
    where
        Self: Sized,
    {
        // TODO prettify this
        PageSearchStream::create(self, move |page| {
            let id_ = id.clone();
            Box::pin(async move { self.get_followers_page(&id_, page).await })
        })
    }

    /// Get replies to a comment.
    ///
    /// # Implementation
    /// Expected to GET `/api/articles/ARTICLE_ID/comments/COMMENT_ID/replies`,
    /// but at the time of me writing this, result is independent of `ARTICLE_ID` part, and thus can be retrieved with it being just zeroes:
    ///
    /// GET `/api/articles/000000000000000000000000/comments/COMMENT_ID/replies`
    async fn get_replies(&self, comment: &CommentId) -> Res<Vec<ReplyComment>>;

    /// Get a single feed page.
    ///
    /// # Implementation
    /// Expected to GET to `/api/preferences/feed?page=PAGE`
    async fn feed_page(&self, page: NonZeroUsize) -> Res<Vec<FeedArticle>>;

    /// Get articles at feed.
    ///
    /// # Returns
    /// A stream of result pages.
    ///
    /// # Implementation details
    /// This function should not be reimplemented.
    ///
    /// Currently, the underlying stream consequently calls for result pages,
    /// although this might be changed in the future for more optimized approach.
    ///
    /// # Note
    /// Stream ends after first error, since after the error, there's no way for stream to determine, if results had ended
    fn feed(&self) -> PageSearchStream<Self::Auth, FeedArticle>
    where
        Self: Sized,
    {
        PageSearchStream::create(self, |page| self.feed_page(page))
    }

    /// Logs in a Drukarnia user.
    ///
    /// Currently, should not be used and/or implemented
    async fn login(&self, _credentials: Credentials) -> Res<Self::Auth>
    where
        Self::Auth: AuthDrukarnia,
    {
        unimplemented!("Unstable");
    }
}

/// Represents Drukarnia API caller that currently has a valid authenticated user
///
/// It's expected to log user out, once dropped
// TODO make these docs pretty
#[async_trait]
pub trait AuthDrukarnia: Deref<Target = Self::Downgrade> {
    /// Represents a type `AuthDrukarnia` should dereference to
    ///
    /// It's intended to be the type one uses to obtain `AuthDrukarnia` in the first place
    type Downgrade;

    /// Returns authorized user data
    fn authorized_user(&self) -> &AuthorizedUser;

    /// # Implementation details
    /// a request to `/api/relationships/subscribe/{USER_ID}`
    /// POST means "follow"
    /// DELETE means "unfollow"
    async fn user_set_following(&self, id: &UserId, follow: bool) -> Res;

    /// GET to `/api/articles/bookmarks/lists`
    /// respond: List of
    /// {
    ///     "_id":LIST_ID?,
    ///     "name":LIST_NAME,
    ///     "articlesNum":usize,
    ///     "owner":USER_ID,
    ///     "__v":0
    /// }
    async fn get_bookmark_lists(&self) -> Res<Vec<FullList>>;

    /// - bookmark: POST to `/api/articles/bookmarks` with json body
    ///     {
    ///         "article": ARTICLE_ID,
    ///         "list": LIST_ID
    ///     }
    /// responds with
    ///     {
    ///         "article": ARTICLE_ID,
    ///         "owner":OWNER_ID,
    ///         "list":LIST_ID,
    ///         "_id": BOOKMARK_ID?,
    ///         "createdAt": DateTime,
    ///         "__v":??
    ///     }
    ///
    /// - un-bookmark: DELETE to `/api/articles/{ARTICLE_ID}/bookmarks`
    /// responds with the same, I guess
    async fn bookmark_article(&self, list: &ListId, article: &ArticleId) -> Res<FullBookmark>;

    ///
    async fn unbookmark_article(&self, article: &ArticleId) -> Res<FullBookmark>;

    /// GET to `/api/articles/bookmarks/lists/{LIST_ID}`
    /// respond: List of
    /// {
    ///     "_id":ArticleId,
    ///     "title":ArticleTitle,
    ///     "description":ArticleDescription,
    ///     "mainTag":TagName,
    ///     "readTime":Duration,
    ///     "slug":ArticleSlug,
    ///     "mainTagSlug":TagSlug,
    ///     "mainTagId":TagId,
    ///     "createdAt":DateTime,
    ///     "isBookmarked":bool
    /// }
    async fn get_list_articles(&self, list: &ListId) -> Res<Vec<ListArticle>>;

    /// - like: POST to `/api/articles/{ARTICLE_ID}/like`
    /// with json body
    /// {
    ///     likes: LIKES_NUM????
    /// }
    async fn like_article(&self, article: &ArticleId, likes: usize) -> Res;

    // Postponed for future revisions
    /*
    /// POST to `/api/articles/{ARTICLE_ID}/comments`
    /// with json body
    /// {
    ///     comment: HTML-LIKE?
    /// }
    /// response: plain text `COMMENT_ID`
    async fn post_comment(
        &self,
        content: HtmlLike,
    ) -> Res<CommentId>;
    */

    /// -like: POST to `/api/articles/{ARTICLE_ID}/comments/{COMMENT_ID}/likes`
    /// with empty json body
    ///
    /// - unlike: DELETE to same endpoint
    async fn set_comment_liked(&self, article: &ArticleId, comment: &CommentId, liked: bool)
        -> Res;

    // Actual interface for this needs some thinking
    // Reserved for future revisions
    /*
    /// POST to `/api/articles/{ARTICLE_ID}/comments/{COMMENT_ID}/replies`
    /// with json body
    /// {
    ///     "comment":HMTL-LIKE,
    ///     "rootComment":CommentId,
    ///     "rootCommentOwner":UserId,
    ///     "replyToUser":UserId,
    ///     "replyToComment":CommentId
    /// }
    async fn post_reply(&self);
    */

    // Postponed for future revisions
    /*
    /// - DELETE to `/api/articles/{ARTICLE_ID}/comments/{COMMENT_ID}`
    async fn delete_comment(&self, article: &ArticleId, comment: &CommentId) -> Res;
    */

    // TODO
    // As you can see, notification contains a so-called `type`, and I feel like there's no guarantee on them having constant structure
    // To actually find that out, I'd probably need to analyze site's scripts to figure out exactly what each of these does
    // I'll leave it for future revisions
    /*
    /// GET to `/api/notifications`
    /// response: List of
    /// {
    ///     "_id":NOTIFICATION_ID?,
    ///     "owner":USER_ID?,
    ///     "type":TYPE(usize?),
    ///     "details":{
    ///         "actionOwner":{
    ///             "_id":USER_ID?,
    ///             "name":USER_DISPLAY_NAME,
    ///             "username":USER_NAME,
    ///             "avatar":Url?
    ///         }
    ///     },
    ///     "seen": bool,
    ///     "createdAt": DateTime,
    ///     "__v":0,
    ///     "isLiked": bool? // what does that even mean? can you "like" a notification???
    /// },
    /// {
    /// {
    ///     "_id":NOTIFICATION_ID?,
    ///     "owner":USER_ID?,
    ///     "type":TYPE(usize?),
    ///     "seen": bool,
    ///     "createdAt": DateTime,
    ///     "__v":0,
    ///     "isLiked": bool?
    ///     // yep! there's no "details" field here!
    /// }
    /* async fn get_notifications_page(&self, page: usize); */

    /// TODO
    /* fn get_notifications(&self); */
    */
}
