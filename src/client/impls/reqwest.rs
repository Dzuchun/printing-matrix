use std::num::NonZeroUsize;

use async_trait::async_trait;
use derive_more::Deref;
use reqwest::{
    header::{self},
    Client, Response, StatusCode,
};
use secrecy::{ExposeSecret, SecretString};
use tracing::info;

use crate::{
    client::{
        ArticleId, ArticleSlug, ArticleTitle, AuthDrukarnia, AuthorizedUser, CommentId,
        DrukarniaApi, Error, FullArticle, FullTag, FullUser, PopularTag, Res, ShortUser, TagSlug,
        UserName,
    },
    object::{
        FeedArticle, FollowerUser, FullBookmark, FullList, ListArticle, ListId, RecommendedArticle,
        ReplyComment, UserId,
    },
};

static USER_AGENT: &str = "type-matrux/0.1.0";

/// Super-angry message explaining that url arithmetic is correct
///
/// Should not be shown to the end-user, if crate was tested properly
static ANGRY_URL: &str = "Should be able to append endpoint to base url";

/// A convenience macro to set user agent header, send a request, await it and map-return any request error
///
/// Not intended to be used outside of this module, as it's tied to `reqwest` crate functions
macro_rules! send_ok {
    ($req:expr) => {
        $req.header(header::USER_AGENT, USER_AGENT)
            .header(header::HOST, "drukarnia.com.ua")
            .send()
            .await
            .map_err(|err| super::super::Error::OnExecution(Box::new(err)))?
    };
}

static CONTEXT_SIZE: usize = 30;

/// A convenience macro to parse a response to json, await for a result and map-return any error
///
/// Not intended to be used outside of this module, as it's tied to `reqwest` crate functions
macro_rules! json_ok {
    ($res:expr, $tp:ty) => {{
        let text: String = $res
            .text()
            .await
            .map_err(|err| super::super::Error::OnExecution(Box::new(err)))?;
        serde_json::from_str::<$tp>(text.as_str()).map_err(|err| {
            let line = err.line();
            let line = text
                .lines()
                .nth(line - 1)
                .expect("Line number should be valid");
            let column = err.column();
            let cause = line[column.saturating_sub(CONTEXT_SIZE)
                ..std::cmp::min(column + CONTEXT_SIZE, line.len())]
                .to_owned();
            super::super::Error::BadJson(err, cause)
        })?
    }};
}

#[allow(unused)]
fn extract_token(res: &Response) -> Option<SecretString> {
    res.headers()
        .into_iter()
        .find_map(|(key, value)| {
            let v = value.to_str().ok()?;
            (key.as_str() == header::SET_COOKIE && v.starts_with("token=")).then_some(v)
        })
        .map(|v| SecretString::new(v.to_owned()))
}

#[derive(Debug, serde::Deserialize)]
#[allow(unused)]
struct AuthResponse {
    user: AuthorizedUser,
}

#[async_trait]
impl DrukarniaApi for Client {
    type Auth = Auth;

    #[tracing::instrument(name = "Fetching popular tags")]
    async fn popular_tags(&self) -> Res<Vec<PopularTag>> {
        const ENDPOINT: &str = "/api/articles/tags/popular";
        let url = self.base_url().join(ENDPOINT).expect(ANGRY_URL);
        let response = send_ok!(self.get(url));
        let tag = json_ok!(response, Vec<PopularTag>);
        Ok(tag)
    }

    #[tracing::instrument(name = "Loading user")]
    async fn get_user(&self, name: &UserName) -> Res<FullUser> {
        const ENDPOINT: &str = "/api/users/profile/";
        let url = self
            .base_url()
            .join(ENDPOINT)
            .and_then(|endpoint| endpoint.join(name.as_ref()))
            .expect(ANGRY_URL);
        let response = send_ok!(self.get(url));

        if response.status() == StatusCode::NOT_FOUND {
            // User does not exist
            return Err(Error::NoObject);
        }

        let user = json_ok!(response, FullUser);
        Ok(user)
    }

    #[tracing::instrument(name = "Searching user")]
    async fn search_user_page(&self, name: &UserName, page: NonZeroUsize) -> Res<Vec<ShortUser>> {
        const ENDPOINT: &str = "/api/users/info";
        let mut url = self.base_url().join(ENDPOINT).expect(ANGRY_URL);
        url.query_pairs_mut()
            .append_pair("name", name.as_ref())
            .append_pair("page", &page.to_string())
            .append_pair("withRelationships", "true");
        let response = send_ok!(self.get(url));
        let users_page = json_ok!(response, Vec<ShortUser>);
        Ok(users_page)
    }

    #[tracing::instrument(name = "Loading tag")]
    async fn get_tag(&self, slug: &TagSlug) -> Res<FullTag> {
        const ENDPOINT: &str = "/api/articles/tags/";
        let mut url = self
            .base_url()
            .join(ENDPOINT)
            .and_then(|endpoint| endpoint.join(slug.as_ref()))
            .expect(ANGRY_URL);
        // FIXME not really sure why should I add this here,
        // but the site returns 404 otherwise :idk:
        url.query_pairs_mut().append_pair("page", "1");
        let response = send_ok!(self.get(url));

        if response.status() == StatusCode::NOT_FOUND {
            // Tag does not exist
            return Err(Error::NoObject);
        }

        let tag = json_ok!(response, FullTag);
        Ok(tag)
    }

    #[tracing::instrument(name = "Loading article")]
    async fn get_article(&self, slug: &ArticleSlug) -> Res<FullArticle> {
        const ENDPOINT: &str = "/api/articles/";
        let url = self
            .base_url()
            .join(ENDPOINT)
            .and_then(|endpoint| endpoint.join(slug.as_ref()))
            .expect(ANGRY_URL);
        let response = send_ok!(self.get(url));
        if response.status() == StatusCode::NOT_FOUND {
            // Article does not exist
            return Err(Error::NoObject);
        }

        let article = json_ok!(response, FullArticle);
        Ok(article)
    }

    #[tracing::instrument(name = "Searching for article")]
    async fn search_article_page(
        &self,
        name: &ArticleTitle,
        page: NonZeroUsize,
    ) -> Res<Vec<RecommendedArticle>> {
        const ENDPOINT: &str = "/api/articles/search";
        let mut url = self.base_url().join(ENDPOINT).expect(ANGRY_URL);
        url.query_pairs_mut()
            .append_pair("name", name.as_ref())
            .append_pair("page", &page.to_string());
        let response = send_ok!(self.get(url));
        let articles = json_ok!(response, Vec<RecommendedArticle>);
        Ok(articles)
    }

    #[tracing::instrument(name = "Loading followers")]
    async fn get_followers_page(&self, id: &UserId, page: NonZeroUsize) -> Res<Vec<FollowerUser>> {
        const ENDPOINT: &str = "/api/relationships/";
        let mut url = self
            .base_url()
            .join(ENDPOINT)
            .and_then(|endpoint| endpoint.join(&format!("{}/followers", id)))
            .expect(ANGRY_URL);
        url.query_pairs_mut().append_pair("page", &page.to_string());
        let response = send_ok!(self.get(url));
        let followers = json_ok!(response, Vec<FollowerUser>);
        Ok(followers)
    }

    #[tracing::instrument(name = "Getting replies")]
    async fn get_replies(&self, comment: &CommentId) -> Res<Vec<ReplyComment>> {
        const ENDPOINT: &str = "/api/articles/000000000000000000000000/comments/";
        let url = self
            .base_url()
            .join(ENDPOINT)
            .and_then(|article_comments| article_comments.join(&format!("{}/replies", comment)))
            .expect(ANGRY_URL);
        let response = send_ok!(self.get(url));

        if response.status() == StatusCode::UNAUTHORIZED {
            return Err(Error::NoObject);
        }
        // TODO add assertions for expected response code in all of the functions
        assert_eq!(response.status(), StatusCode::OK, "Unexpected status code");

        let comments = json_ok!(response, Vec<ReplyComment>);
        Ok(comments)
    }

    #[tracing::instrument(name = "Loading feed page")]
    async fn feed_page(&self, page: NonZeroUsize) -> Res<Vec<FeedArticle>> {
        const ENDPOINT: &str = "/api/preferences/feed";
        let url = self
            .base_url()
            .join(ENDPOINT)
            .map(|mut endpoint| {
                endpoint
                    .query_pairs_mut()
                    .append_pair("page", &page.to_string());
                endpoint
            })
            .expect(ANGRY_URL);
        let response = send_ok!(self.get(url));

        // TODO add assertions for expected response code in all of the functions
        assert_eq!(response.status(), StatusCode::OK, "Unexpected status code");

        let feed_articles = json_ok!(response, Vec<FeedArticle>);
        Ok(feed_articles)
    }

    /*
    #[tracing::instrument(name = "Authenticating user")]
    #[allow(unused)]
    async fn login(&self, credentials: Credentials) -> Res<Self::Auth> {
        static ENDPOINT: &str = "/api/users/login";
        let url = self.base_url().join(ENDPOINT).expect(ANGRY_URL);
        let body = format!(
            r#"
            {{
                "email": "{}",
                "password": "{}"
            }}
            "#,
            credentials.email(),
            credentials.password().expose_secret()
        );
        let response = send_ok!(self
            .post(url)
            .body(body)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.essence_str()));

        if response.status() == StatusCode::NOT_FOUND {
            // "Такого юзера не існує або невірний пароль"
            return Err(Error::BadCredentials);
        }

        let token = extract_token(&response).ok_or(Error::NoToken)?;
        let auth_user = json_ok!(response, AuthResponse).user;
        let new_client = Client::builder()
            .default_headers(HeaderMap::new())
            .build()
            .expect("Should be able to build new client");
        Ok(Auth(new_client, auth_user, token))
    }
    */
}

/// [`reqwest::Client`] wrapper, that's currently authorized on the site
#[derive(Debug, Deref)]
pub struct Auth(#[deref] Client, AuthorizedUser, SecretString);

macro_rules! auth_send_ok {
    ($req:expr, $t:expr) => {
        send_ok!($req
            .header(header::COOKIE, $t.expose_secret())
            .header(
                header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED.essence_str()
            )
            .header(header::CONTENT_LENGTH, 0))
    };
}

#[async_trait]
impl AuthDrukarnia for Auth {
    type Downgrade = Client;

    fn authorized_user(&self) -> &AuthorizedUser {
        &self.1
    }

    // FIXME IT JUST DOES NOT WORK
    // I send LITERALLY THE SAME REQUEST AS THEIR WEBSITE CURL AND POSTMAN, BUT NOTHING CHANGES AFTER MINE
    // I HAVE NO IDEA WHAT AM I DOING WRONG
    // Also, server's response is 201, meaning has accepted the change; expect it did not, that's a lie
    #[allow(unreachable_code)]
    #[tracing::instrument(name = "Following user")]
    async fn user_set_following(&self, id: &UserId, follow: bool) -> Res {
        static ENDPOINT: &str = "/api/relationships/subscribe/";

        let url = self
            .base_url()
            .join(ENDPOINT)
            .and_then(|endpoint| endpoint.join(&id.to_string()))
            .expect(ANGRY_URL);

        let request = if follow {
            self.post(url)
        } else {
            self.delete(url)
        };
        let response = auth_send_ok!(request, self.2);
        assert_eq!(
            response.status(),
            if follow {
                StatusCode::CREATED
            } else {
                StatusCode::OK
            },
            "Response was not successful: {:?}",
            response
        );

        let body = response
            .text()
            .await
            .expect("Should be able to decode a response");
        info!(body, "Response body");
        Ok(())
    }

    #[tracing::instrument(name = "Loading bookmarks")]
    async fn get_bookmark_lists(&self) -> Res<Vec<FullList>> {
        static ENDPOINT: &str = "/api/articles/bookmarks/lists";
        let url = self.base_url().join(ENDPOINT).expect(ANGRY_URL);
        let response = auth_send_ok!(self.get(url), self.2);
        let lists = json_ok!(response, Vec<FullList>);
        Ok(lists)
    }

    #[tracing::instrument(name = "Bookmarking an article")]
    async fn bookmark_article(&self, list: &ListId, article: &ArticleId) -> Res<FullBookmark> {
        static ENDPOINT: &str = "/api/articles/bookmarks";
        let url = self.base_url().join(ENDPOINT).expect(ANGRY_URL);
        let body = format!(
            r#"
            {{
                "article": {},
                "list": {}
            }}
            "#,
            article, list
        );
        let response = auth_send_ok!(
            self.post(url)
                .body(body)
                .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.essence_str()),
            self.2
        );
        let bookmark = json_ok!(response, FullBookmark);
        Ok(bookmark)
    }

    #[tracing::instrument(name = "unBookmarking an article")]
    async fn unbookmark_article(&self, article: &ArticleId) -> Res<FullBookmark> {
        static ENDPOINT: &str = "/api/articles/";
        let url = self
            .base_url()
            .join(ENDPOINT)
            .and_then(|endpoint| endpoint.join(&format!("{}/bookmarks", article)))
            .expect(ANGRY_URL);
        let response = auth_send_ok!(self.delete(url), self.2);
        let bookmark = json_ok!(response, FullBookmark);
        Ok(bookmark)
    }

    #[tracing::instrument(name = "Loading article list")]
    async fn get_list_articles(&self, list: &ListId) -> Res<Vec<ListArticle>> {
        static ENDPOINT: &str = "/api/articles/bookmarks/lists/";
        let url = self
            .base_url()
            .join(ENDPOINT)
            .and_then(|endpoint| endpoint.join(&list.to_string()))
            .expect(ANGRY_URL);
        let response = auth_send_ok!(self.get(url), self.2);

        let list = json_ok!(response, Vec<ListArticle>);
        Ok(list)
    }

    #[tracing::instrument(name = "Liking article")]
    async fn like_article(&self, article: &ArticleId, likes: usize) -> Res {
        static ENDPOINT: &str = "/api/articles/";
        let url = self
            .base_url()
            .join(ENDPOINT)
            .and_then(|endpoint| endpoint.join(&format!("{}/like", article)))
            .expect(ANGRY_URL);
        let body = format!(
            r#"
            {{
                "likes": {}
            }}
            "#,
            likes
        );
        let _ = auth_send_ok!(
            self.post(url)
                .body(body)
                .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.essence_str()),
            self.2
        );
        Ok(())
    }

    #[tracing::instrument(name = "Liking comment")]
    async fn set_comment_liked(
        &self,
        article: &ArticleId,
        comment: &CommentId,
        liked: bool,
    ) -> Res {
        static ENDPOINT: &str = "/api/articles/";
        let url = self
            .base_url()
            .join(ENDPOINT)
            .and_then(|endpoint| endpoint.join(&format!("{}/comments/{}/likes", article, comment)))
            .expect(ANGRY_URL);
        let request = if liked {
            self.post(url)
        } else {
            self.delete(url)
        };
        let _ = auth_send_ok!(request, self.2);
        Ok(())
    }

    // TODO
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

    // TODO
    // As you can see, notifications contains a so-called `type`, and I feel like there's no guarantee on them having constant structure
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

    /* fn get_notifications(&self); */
    */
}

#[tracing::instrument(name = "Logging user out")]
async fn log_out(auth: Client) {
    static ENDPOINT: &str = "/api/users/logout";
    let url = auth.base_url().join(ENDPOINT).expect(ANGRY_URL);
    auth.get(url).send().await.ok();
}

impl Drop for Auth {
    fn drop(&mut self) {
        // I tried REALLY HARD, but literally every solution
        // I came up with resulted in thread hanging indefinitely
        // (I suspect that's because of tokio::main macro runtime is single-threaded,
        // but CREATING SEPARATE THREADS DID NOT HELP)

        // Scoped thread with `futures::executor::block_on`
        // would not actually run the future, but just get blocked after first await point.
        // Scoped threads alongside it would run just fine, so something IS really fishy here

        // This all means, that there's literally no way to do the following without cloning `reqwest::Client`

        // Also, I don't really know if there's a guarantee that tokio task actually finishes before parent runtime drops
        let client = self.0.clone();
        tokio::spawn(log_out(client)); // also, this is the only place in my release code,
                                       // where I use tokio, making it a direct dependency :(
                                       // I guess, `reqwest` does that internally anyway, so it's a **big** problem?
    }
}
