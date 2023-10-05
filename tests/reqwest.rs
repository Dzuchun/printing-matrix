use std::num::NonZeroUsize;

use once_cell::sync::Lazy;
use reqwest::Client;
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{fmt::MakeWriter, prelude::__tracing_subscriber_SubscriberExt, Registry};
use type_matrux::{client::AuthDrukarnia, object::Credentials, DrukarniaApi};

macro_rules! auth_guard {
    () => {
        if std::env::var("TEST_AUTH").is_err() {
            eprintln!("Skipped");
            return;
        }
    };
}

fn nonzero_one() -> NonZeroUsize {
    NonZeroUsize::new(1).expect("1 != 0")
}

fn get_credentials() -> Credentials {
    toml::from_str(
        &std::fs::read_to_string("credentials.toml")
            .expect("Should be able to read credentials file"),
    )
    .expect("Credentials should be a valid toml")
}

// TODO add verification for these "static claims"
fn get_existing_user_id() -> type_matrux::object::UserId {
    static EXISTING_USER_ID_BYTES: [u8; 12] = [
        0x64, 0x3a, 0xf9, 0xfc, 0x12, 0x72, 0xbd, 0x90, 0x66, 0xa1, 0xff, 0xdb,
    ];

    unsafe { std::mem::transmute(EXISTING_USER_ID_BYTES) }
}

fn get_existing_article_id() -> type_matrux::object::ArticleId {
    static EXISTING_ARTICLE_ID_BYTES: [u8; 12] = [
        0x65, 0x11, 0xe0, 0x36, 0x28, 0x0f, 0x44, 0x21, 0x02, 0x5f, 0x09, 0xfd,
    ];

    unsafe { std::mem::transmute(EXISTING_ARTICLE_ID_BYTES) }
}

fn get_existing_comment_id() -> type_matrux::object::CommentId {
    // 651ae7dc280f4421026b12c5
    static EXISTING_COMMENT_ID_BYTES: [u8; 12] = [
        0x65, 0x1a, 0xe7, 0xdc, 0x28, 0x0f, 0x44, 0x21, 0x02, 0x6b, 0x12, 0xc5,
    ];

    unsafe { std::mem::transmute(EXISTING_COMMENT_ID_BYTES) }
}

fn get_non_existing_comment_id() -> type_matrux::object::CommentId {
    static NON_EXISTING_COMMENT_ID_BYTES: [u8; 12] = [
        0x65, 0x1a, 0x00, 0xdc, 0x28, 0x0f, 0x00, 0x21, 0x02, 0x00, 0x12, 0xc5,
    ];

    unsafe { std::mem::transmute(NON_EXISTING_COMMENT_ID_BYTES) }
}

async fn get_auth() -> type_matrux::client::ReqwestAuth {
    let valid_credentials = get_credentials();
    let client = Client::new();
    client
        .login(valid_credentials)
        .await
        .expect("Should get logged in")
}

async fn get_auth_list_id(auth: &type_matrux::client::ReqwestAuth) -> type_matrux::object::ListId {
    let lists = auth
        .get_bookmark_lists()
        .await
        .expect("Should be able to get lists");
    lists[0].id().clone()
}

pub fn create_logger<Sink>(sink: Sink) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // create filter from environment variable, or use info level filtering
    // formatting layer
    let formatting = BunyanFormattingLayer::new("testing".to_owned(), sink);
    // actual logger
    Registry::default()
        // stores additional info is json for easy-use
        .with(JsonStorageLayer)
        .with(formatting)
}

pub fn set_logger(logger: impl Subscriber + Send + Sync) {
    set_global_default(logger).expect("Failed to set logger");
}

static LOGGER: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let logger = create_logger(std::io::stdout);
        set_logger(logger);
    } else {
        let logger = create_logger(std::io::sink);
        set_logger(logger);
    }
});

fn setup_log() {
    Lazy::force(&LOGGER);
}

/// This group of tests attempts to query site API and parse response into JSON format
///
/// The intent is to make sure that data representation is indeed correct
///
/// None is expecting any sort of error
mod data_representation {

    use futures::{StreamExt, TryStreamExt};
    use reqwest::Client;
    use type_matrux::{client::AuthDrukarnia, DrukarniaApi};

    use crate::{
        get_auth, get_auth_list_id, get_credentials, get_existing_article_id,
        get_existing_comment_id, get_existing_user_id, nonzero_one, setup_log,
    };

    #[tokio::test]
    async fn popular_tags_should_succeed() {
        setup_log();
        // Arrange
        let client = Client::new();

        // Act
        let popular_tags = client.popular_tags().await;

        // Assert
        assert!(
            popular_tags.is_ok(),
            "Should be able to query popular tags: {}",
            popular_tags.unwrap_err()
        );
    }

    #[tokio::test]
    async fn get_user_should_succeed() {
        setup_log();
        // Arrange
        static EXISTING_USER: &str = "OstanniyCapitalist";
        let client = Client::new();

        // Act
        let user_name = EXISTING_USER.parse().expect("Should be a valid username");
        let user = client.get_user(&user_name).await;

        // Assert
        assert!(
            user.is_ok(),
            "Should be able to query user by username: {}",
            user.unwrap_err()
        );
    }

    #[tokio::test]
    async fn search_users_should_succeed() {
        setup_log();
        // Arrange
        static VALID_USER_QUERY: &str = "Poroshenko";
        let client = Client::new();

        // Act
        let user_query = VALID_USER_QUERY
            .parse()
            .expect("Should be a valid username");
        let users = client.search_user_page(&user_query, nonzero_one()).await;

        // Assert
        assert!(
            users.is_ok(),
            "Should be able to search users by username: {}",
            users.unwrap_err()
        );
    }

    #[tokio::test]
    async fn search_lots_of_users_should_succeed() {
        setup_log();
        // Arrange
        static POPULAR_USER_QUERY: &str = "Іван";
        let client = Client::new();

        // Act
        let user_query = POPULAR_USER_QUERY
            .parse()
            .expect("Should be a valid username");
        let users = client.search_user(user_query);

        // Assert
        users
            .for_each(|page| async {
                assert!(
                    page.is_ok(),
                    "Should be able to search users by username: {}",
                    page.unwrap_err()
                );
            })
            .await;
    }

    #[tokio::test]
    async fn get_tag_should_succeed() {
        setup_log();
        // Arrange
        static EXISTING_TAG_SLUG: &str = "istoriya";
        let client = Client::new();

        // Act
        let tag_slug = EXISTING_TAG_SLUG.parse().expect("Should be valid tag slug");
        let tag = client.get_tag(&tag_slug).await;

        // Assert
        assert!(
            tag.is_ok(),
            "Should be able to get tag by slug: {}",
            tag.unwrap_err()
        );
    }

    #[tokio::test]
    async fn get_article_should_succeed() {
        setup_log();
        // Arrange
        static EXISTING_ARTICLE_SLUG: &str =
            "gitlab-istoriya-uspikhu-ukrayinskogo-konkurenta-github-t7agP";
        let client = Client::new();

        // Act
        let article_slug = EXISTING_ARTICLE_SLUG
            .parse()
            .expect("Should be valid article slug");
        let article = client.get_article(&article_slug).await;

        // Assert
        assert!(
            article.is_ok(),
            "Should be able to get article by slug: {}",
            article.unwrap_err()
        );
    }

    // FIXME MISSING FIELD _id
    #[tokio::test]
    async fn search_article_should_succeed() {
        setup_log();
        // Arrange
        static VALID_ARTICLE_NAME: &str = "Дія";
        let client = Client::new();

        // Act
        let article_name = VALID_ARTICLE_NAME
            .parse()
            .expect("Should be valid article title");
        let articles = client
            .search_article_page(&article_name, nonzero_one())
            .await;

        // Assert
        assert!(
            articles.is_ok(),
            "Should be able to search articles by title: {}",
            articles.unwrap_err()
        );
    }

    #[tokio::test]
    async fn get_followers_should_succeed() {
        setup_log();
        // Arrange
        let client = Client::new();
        let existing_user_id = get_existing_user_id();

        // Act
        let articles = client
            .get_followers_page(&existing_user_id, nonzero_one())
            .await;

        // Assert
        assert!(
            articles.is_ok(),
            "Should be able to load followers by user id: {}",
            articles.unwrap_err()
        );
    }

    #[tokio::test]
    async fn get_replies_should_succeed() {
        setup_log();
        // Arrange
        let client = Client::new();
        let existing_comment_id = get_existing_comment_id();

        // Act
        let comments = client.get_replies(&existing_comment_id).await;

        // Assert
        assert!(
            comments.is_ok(),
            "Should be able to load comments: {}",
            comments.unwrap_err()
        );
    }

    #[tokio::test]
    async fn get_feed_should_succeed() {
        setup_log();
        // Arrange
        let client = Client::new();

        // Act
        let feed_articles = client.feed_page(nonzero_one()).await;

        // Assert
        assert!(
            feed_articles.is_ok(),
            "Should be able to get feed: {}",
            feed_articles.unwrap_err()
        );
    }

    #[tokio::test]
    async fn get_feed_flat_100_should_succeed() {
        setup_log();
        // Arrange
        let client = Client::new();

        // Act
        let feed = client.feed().flat();
        let feed_articles: Result<Vec<_>, type_matrux::client::Error> =
            feed.take(100).try_collect().await;

        // Assert
        assert!(
            feed_articles.is_ok(),
            "Should be able to get feed: {}",
            feed_articles.unwrap_err()
        );
    }

    #[tokio::test]
    async fn login_should_succeed() {
        setup_log();
        auth_guard!();
        // Arrange
        let valid_credentials = get_credentials();
        let client = Client::new();

        // Act
        let auth = client.login(valid_credentials).await;

        // Assert
        assert!(
            auth.is_ok(),
            "Should be able to login: {}",
            auth.unwrap_err()
        );
    }

    #[tokio::test]
    // TODO make a coherence test
    async fn follow_should_succeed() {
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;
        let existing_user_id = get_existing_user_id();

        // Act
        let res = auth.user_set_following(&existing_user_id, true).await;
        // tokio::time::sleep(Duration::from_secs(2)).await;

        // Assert
        assert!(
            res.is_ok(),
            "Should be able to follow: {:?}",
            res.unwrap_err()
        );
    }

    #[tokio::test]
    // TODO make a coherence test
    async fn nollow_should_succeed() {
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;
        let existing_user_id = get_existing_user_id();

        // Act
        let res = auth.user_set_following(&existing_user_id, false).await;

        // Assert
        assert!(
            res.is_ok(),
            "Should be able to unfollow: {:?}",
            res.unwrap_err()
        );
    }

    #[tokio::test]
    async fn get_bookmarks_should_succeed() {
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;

        // Act
        let res = auth.get_bookmark_lists().await;

        // Assert
        assert!(
            res.is_ok(),
            "Should be able to get bookmarks: {:?}",
            res.unwrap_err()
        );
    }

    #[tokio::test]
    async fn bookmark_should_succeed() {
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;
        let list_id = get_auth_list_id(&auth).await;
        let existing_article_id = get_existing_article_id();

        // Act
        let res = auth.bookmark_article(&list_id, &existing_article_id).await;

        // Assert
        assert!(
            res.is_ok(),
            "Should be able to bookmark an article: {:?}",
            res.unwrap_err()
        );
    }

    #[tokio::test]
    async fn unbookmark_should_succeed() {
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;
        let existing_article_id = get_existing_article_id();

        // Act
        let res = auth.unbookmark_article(&existing_article_id).await;

        // Assert
        assert!(
            res.is_ok(),
            "Should be able to unbookmark an article: {:?}",
            res.unwrap_err()
        );
    }

    #[tokio::test]
    async fn get_list_articles_should_succeed() {
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;
        let list_id = get_auth_list_id(&auth).await;

        // Act
        let res = auth.get_list_articles(&list_id).await;

        // Assert
        assert!(
            res.is_ok(),
            "Should be able to get listed articles: {:?}",
            res.unwrap_err()
        );
    }

    #[tokio::test]
    async fn like_article_should_succeed() {
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;
        let existing_article_id = get_existing_article_id();

        // Act
        let res = auth.like_article(&existing_article_id, 1).await;

        // Assert
        assert!(
            res.is_ok(),
            "Should be able to like an article: {:?}",
            res.unwrap_err()
        );
    }

    #[tokio::test]
    async fn unlike_article_should_succeed() {
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;
        let existing_article_id = get_existing_article_id();

        // Act
        let res = auth.like_article(&existing_article_id, 0).await; // TODO not really sure what this like num thing means

        // Assert
        assert!(
            res.is_ok(),
            "Should be able to unlike an article: {:?}",
            res.unwrap_err()
        );
    }

    #[tokio::test]
    async fn like_comment_should_succeed() {
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;
        let existing_comment_id = get_existing_comment_id();
        let existing_article_id = get_existing_article_id();

        // Act
        let res = auth
            .set_comment_liked(&existing_article_id, &existing_comment_id, true)
            .await;

        // Assert
        assert!(
            res.is_ok(),
            "Should be able to like a comment: {:?}",
            res.unwrap_err()
        );
    }

    #[tokio::test]
    async fn unlike_comment_should_succeed() {
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;
        let existing_comment_id = get_existing_comment_id();
        let existing_article_id = get_existing_article_id();

        // Act
        let res = auth
            .set_comment_liked(&existing_article_id, &existing_comment_id, false)
            .await;

        // Assert
        assert!(
            res.is_ok(),
            "Should be able to like a comment: {:?}",
            res.unwrap_err()
        );
    }
}

/// This group of tests attempts various malformed queries to site API and check that they indeed return an error
///
/// The intent is to guarantee certain responses in various situations
///
/// All of them expect to get a specific type of error
mod error_representation {
    use reqwest::Client;
    use type_matrux::{
        client::{AuthDrukarnia, Error},
        object::Credentials,
        DrukarniaApi,
    };

    use crate::{
        get_auth, get_auth_list_id, get_existing_article_id, get_non_existing_comment_id, setup_log,
    };

    #[tokio::test]
    async fn get_non_existing_user_should_error() {
        setup_log();
        // Arrange
        static NON_EXITING_USER: &str = "efgyb5h644un5teyvg5346y7gb564cfytg54";
        let client = Client::new();

        // Act
        let username = NON_EXITING_USER
            .parse()
            .expect("Should be a valid username");
        let user = client.get_user(&username).await;

        // Assert
        let real = user.expect_err("Should not allow getting a non-existent user");
        let exp = Error::NoObject;
        assert!(
            std::mem::discriminant(&real) == std::mem::discriminant(&exp),
            "Error type is not correct. Real: {}, Exp: {}",
            real,
            exp
        );
    }

    #[tokio::test]
    async fn get_non_existing_tag_should_error() {
        setup_log();
        // Arrange
        static NON_EXISTING_TAG_SLUG: &str = "f3ervtg4rf3ced";
        let client = Client::new();

        // Act
        let tag_slug = NON_EXISTING_TAG_SLUG
            .parse()
            .expect("Should be valid tag slug");
        let tag = client.get_tag(&tag_slug).await;

        // Assert
        let real = tag.expect_err("Should not allow getting a non-existent tag");
        let exp = Error::NoObject;
        assert!(
            std::mem::discriminant(&real) == std::mem::discriminant(&exp),
            "Error type is not correct. Real: {}, Exp: {}",
            real,
            exp
        );
    }

    #[tokio::test]
    async fn get_non_existing_article_should_error() {
        setup_log();
        // Arrange
        static NON_EXISTING_ARTICLE_SLUG: &str = "34uih8r34n897rgtn9837gt9r87ns3tg29re876g2";
        let client = Client::new();

        // Act
        let article_slug = NON_EXISTING_ARTICLE_SLUG
            .parse()
            .expect("Should be valid article slug");
        let article = client.get_article(&article_slug).await;

        // Assert
        let real = article.expect_err("Should not allow getting a non-existent article");
        let exp = Error::NoObject;
        assert!(
            std::mem::discriminant(&real) == std::mem::discriminant(&exp),
            "Error type is not correct. Real: {}, Exp: {}",
            real,
            exp
        );
    }

    #[tokio::test]
    #[ignore = "seems like there's no way to actually check, if comment with a provided id exists"]
    async fn get_non_existing_comment_replies_should_error() {
        setup_log();
        // Arrange
        let client = Client::new();
        let non_exiting_comment_id = get_non_existing_comment_id();

        // Act
        let comments = client.get_replies(&non_exiting_comment_id).await;

        // Assert
        let real =
            comments.expect_err("Should not allow getting a non-existent article comment reply");
        let exp = Error::NoObject;
        assert!(
            std::mem::discriminant(&real) == std::mem::discriminant(&exp),
            "Error type is not correct. Real: {}, Exp: {}",
            real,
            exp
        );
    }

    #[tokio::test]
    async fn login_bad_credentials_should_error() {
        setup_log();
        auth_guard!();
        // Arrange
        let client = Client::new();
        let bad_credentials =
            Credentials::create("e235326643634523421452354@gmail.com", "random_password")
                .expect("Should a valid credentials");

        // Act
        let res = client.login(bad_credentials).await;

        // Assert
        let real = res.expect_err("Should not allow login under arbitrary credentials");
        let exp = Error::BadCredentials;
        assert!(
            std::mem::discriminant(&real) == std::mem::discriminant(&exp),
            "Error type is not correct. Real: {}, Exp: {}",
            real,
            exp
        );
    }

    #[tokio::test]
    async fn non_exiting_article_bookmark_should_error() {
        static NON_EXISTING_ARTICLE_ID_BYTES: [u8; 12] = [
            0x88, 0x11, 0xe0, 0x36, 0x00, 0x0f, 0x44, 0x21, 0x11, 0x5f, 0x09, 0xfd,
        ];
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;
        let list_id = get_auth_list_id(&auth).await;
        let non_exiting_article_id = unsafe { std::mem::transmute(NON_EXISTING_ARTICLE_ID_BYTES) };

        // Act
        let res = auth
            .bookmark_article(&list_id, &non_exiting_article_id)
            .await;

        // Assert
        let real = res.expect_err("Should not allow bookmark a non-exiting article");
        let exp = Error::NoObject;
        assert!(
            std::mem::discriminant(&real) == std::mem::discriminant(&exp),
            "Error type is not correct. Real: {}, Exp: {}",
            real,
            exp
        );

        // Act 2
        let res = auth.unbookmark_article(&non_exiting_article_id).await;

        // Assert 2
        let real = res.expect_err("Should not allow unbookmark a non-exiting article");
        let exp = Error::NoObject;
        assert!(
            std::mem::discriminant(&real) == std::mem::discriminant(&exp),
            "Error type is not correct. Real: {}, Exp: {}",
            real,
            exp
        );
    }

    #[tokio::test]
    async fn non_exiting_list_bookmark_should_error() {
        static NON_EXISTING_LIST_ID_BYTES: [u8; 12] = [
            0x88, 0x11, 0xe0, 0x36, 0x00, 0x0f, 0x44, 0x21, 0x11, 0x5f, 0x09, 0xfd,
        ];
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;
        let exiting_article_id = get_existing_article_id();
        let non_exiting_list_id = unsafe { std::mem::transmute(NON_EXISTING_LIST_ID_BYTES) };

        // Act
        let res = auth
            .bookmark_article(&non_exiting_list_id, &exiting_article_id)
            .await;

        // Assert
        let real = res.expect_err("Should not allow bookmark article into a non-exiting list");
        let exp = Error::NoObject;
        assert!(
            std::mem::discriminant(&real) == std::mem::discriminant(&exp),
            "Error type is not correct. Real: {}, Exp: {}",
            real,
            exp
        );

        // Act 2
        let res = auth.unbookmark_article(&exiting_article_id).await;

        // Assert 2
        let real = res.expect_err("Should not allow unbookmark article from non-existing list");
        let exp = Error::NoObject;
        assert!(
            std::mem::discriminant(&real) == std::mem::discriminant(&exp),
            "Error type is not correct. Real: {}, Exp: {}",
            real,
            exp
        );
    }

    #[tokio::test]
    async fn get_non_exiting_list_articles_should_error() {
        static NON_EXISTING_LIST_ID_BYTES: [u8; 12] = [
            0x88, 0x11, 0xe0, 0x36, 0x00, 0x0f, 0x44, 0x21, 0x11, 0x5f, 0x09, 0xfd,
        ];
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;
        let non_exiting_list_id = unsafe { std::mem::transmute(NON_EXISTING_LIST_ID_BYTES) };

        // Act
        let res = auth.get_list_articles(&non_exiting_list_id).await;

        // Assert
        let real = res.expect_err("Should not allow unbookmark article from non-existing list");
        let exp = Error::NoObject;
        assert!(
            std::mem::discriminant(&real) == std::mem::discriminant(&exp),
            "Error type is not correct. Real: {}, Exp: {}",
            real,
            exp
        );
    }

    #[tokio::test]
    async fn like_non_existing_article_should_error() {
        static NON_EXISTING_ARTICLE_ID_BYTES: [u8; 12] = [
            0x88, 0x11, 0xe0, 0x36, 0x00, 0x0f, 0x44, 0x21, 0x11, 0x5f, 0x09, 0xfd,
        ];
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;
        let non_existing_article_id = unsafe { std::mem::transmute(NON_EXISTING_ARTICLE_ID_BYTES) };

        // Act
        let res = auth.like_article(&non_existing_article_id, 1).await;

        // Assert
        let real = res.expect_err("Should not allow like non existing article");
        let exp = Error::NoObject;
        assert!(
            std::mem::discriminant(&real) == std::mem::discriminant(&exp),
            "Error type is not correct. Real: {}, Exp: {}",
            real,
            exp
        );
    }

    #[tokio::test]
    async fn unlike_non_existing_article_should_error() {
        static NON_EXISTING_ARTICLE_ID_BYTES: [u8; 12] = [
            0x88, 0x11, 0xe0, 0x36, 0x00, 0x0f, 0x44, 0x21, 0x11, 0x5f, 0x09, 0xfd,
        ];
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;
        let non_existing_article_id = unsafe { std::mem::transmute(NON_EXISTING_ARTICLE_ID_BYTES) };

        // Act
        let res = auth.like_article(&non_existing_article_id, 0).await;

        // Assert
        let real = res.expect_err("Should not allow unlike non existing article");
        let exp = Error::NoObject;
        assert!(
            std::mem::discriminant(&real) == std::mem::discriminant(&exp),
            "Error type is not correct. Real: {}, Exp: {}",
            real,
            exp
        );
    }

    #[tokio::test]
    async fn like_non_existing_comment_should_error() {
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;
        let existing_article_id = get_existing_article_id();
        let non_existing_comment_id = get_non_existing_comment_id();

        // Act
        let res = auth
            .set_comment_liked(&existing_article_id, &non_existing_comment_id, true)
            .await;

        // Assert
        let real = res.expect_err("Should not allow like a non-exiting comment");
        let exp = Error::NoObject;
        assert!(
            std::mem::discriminant(&real) == std::mem::discriminant(&exp),
            "Error type is not correct. Real: {}, Exp: {}",
            real,
            exp
        );
    }

    #[tokio::test]
    async fn unlike_non_existing_comment_should_error() {
        setup_log();
        auth_guard!();
        // Arrange
        let auth = get_auth().await;
        let existing_article_id = get_existing_article_id();
        let non_existing_comment_id = get_non_existing_comment_id();

        // Act
        let res = auth
            .set_comment_liked(&existing_article_id, &non_existing_comment_id, false)
            .await;

        // Assert
        let real = res.expect_err("Should not allow like a non-exiting comment");
        let exp = Error::NoObject;
        assert!(
            std::mem::discriminant(&real) == std::mem::discriminant(&exp),
            "Error type is not correct. Real: {}, Exp: {}",
            real,
            exp
        );
    }
}

/// This group of tests aim to query as much data as possible and attempt parsing it
///
/// The idea is to ensure that chosen data representation is actually universal. For example, fields might be absent sometimes
///
/// None is expecting any sort of error
///
/// # WARNING
/// These tests can be genuinely harmful for your personal IP to run
/// as Drukarnia might not be happy with you querying for lots of data at once
mod structure_enforcement {
    use std::{collections::HashSet, time::Duration};

    use futures::StreamExt;
    use rand::seq::IteratorRandom;
    use reqwest::Client;
    use type_matrux::DrukarniaApi;

    use crate::setup_log;

    fn get_safety_break() -> impl FnMut() -> tokio::time::Sleep {
        let mut beep_count = 0;
        let mut rng = rand::thread_rng();
        move || {
            let pause = 500..1500;
            let duration = Duration::from_millis(pause.choose(&mut rng).unwrap());
            beep_count += 1;
            println!("Beep no. {}", beep_count);
            tokio::time::sleep(duration)
        }
    }

    /// Tested objects:
    /// - RecommendedArticle
    /// - ArticleUser
    /// - FullTag
    /// - TagArticle
    /// - FullArticle
    /// - ArticleTag
    /// - ArticleComment
    /// - CommentUser
    #[tokio::test]
    #[ignore = "takes a loooong time, intended to be run manually"]
    async fn search_enforcement() -> Result<(), type_matrux::client::Error> {
        setup_log();
        let mut safety_break = get_safety_break();
        let mut error_count = 0;

        let client = Client::new();
        let article_name = "Дія".parse().unwrap();
        let mut articles = client.search_article(article_name).flat();

        let mut tags = HashSet::new();
        while let Some(page) = articles.next().await {
            match page {
                Ok(article) => {
                    tags.insert(article.main_tag_slug().clone());
                }
                Err(err) => {
                    error_count += 1;
                    eprintln!("Error at tags searching: {}", err);
                }
            }
            safety_break().await;
        }
        eprintln!("Finished searching tags, {} total", tags.len());

        let mut articles = HashSet::new();
        for tag_slug in tags {
            let full_tag = client.get_tag(&tag_slug).await;
            match full_tag {
                Ok(full_tag) => {
                    articles.extend(full_tag.articles().into_iter().map(|a| a.slug().clone()))
                }
                Err(err) => {
                    error_count += 1;
                    eprintln!("Error at articles searching: {}", err);
                }
            }
            safety_break().await;
        }
        eprintln!("Finished searching articles, {} total", articles.len());

        let mut total_likes = 0;
        for slug in articles {
            let article = client.get_article(&slug).await;
            match article {
                Ok(article) => {
                    total_likes += article.like_num();
                }
                Err(err) => {
                    error_count += 1;
                    eprintln!("Error at likes counting: {}", err);
                }
            }
            safety_break().await;
        }

        eprintln!("Finished, {} likes total", total_likes);
        println!("{} errors total", error_count);
        Ok(())
    }

    #[tokio::test]
    #[ignore = "takes a loooong time, intended to be run manually"]
    async fn feed_enforcement() -> Result<(), type_matrux::client::Error> {
        setup_log();
        let mut safety_break = get_safety_break();
        let mut error_count = 0;

        let client = Client::new();
        let mut feed = client.feed().flat().take(500);
        while let Some(maybe_article) = feed.next().await {
            safety_break().await;
            let article = match maybe_article {
                Ok(article) => article,
                Err(err) => {
                    error_count += 1;
                    eprintln!("Error while fetching a feed article {}", err);
                    continue;
                }
            };
            let full_article = match client.get_article(article.slug()).await {
                Ok(full_article) => full_article,
                Err(err) => {
                    error_count += 1;
                    eprintln!(
                        "Error while fetching full article {}, slug: {}",
                        err,
                        article.slug()
                    );
                    continue;
                }
            };
            let full_user = match client.get_user(full_article.owner().username()).await {
                Ok(full_user) => full_user,
                Err(err) => {
                    error_count += 1;
                    eprintln!(
                        "Error while fetching full user {}. Username: {}",
                        err,
                        full_article.owner().username()
                    );
                    continue;
                }
            };
            let mut followers = client
                .get_followers(full_user.id().clone())
                .flat()
                .take(100);
            while let Some(res) = followers.next().await {
                safety_break().await;
                if let Err(err) = res {
                    error_count += 1;
                    eprintln!(
                        "Error while fetching user followers {}. Username: {}",
                        err,
                        full_article.owner().username()
                    );
                }
            }
            let full_tag = match client.get_tag(full_article.main_tag_slug()).await {
                Ok(full_tag) => full_tag,
                Err(err) => {
                    error_count += 1;
                    eprintln!(
                        "Error while fetching full tag: {}, tag slug: {}",
                        err,
                        full_article.main_tag_slug()
                    );
                    continue;
                }
            };
            println!(
                "Main tag of the article is {}, author's name is {}. Has {} comments",
                full_tag.name(),
                full_user.name(),
                full_article.comments().len() // yeah, there's a better field for that, I don't care
            );
        }
        println!("{} errors total", error_count);
        Ok(())
    }
}

// TODO Unfortunately, cannot test it yet, as any change introduced to the site, must be done by authorized user
// That has some complications
/// This group of tests aim to check that changes caused by one call to API,
/// indeed has expected effect and can be observed by another API call
///
/// The idea is to prevent calls that take unexpected effect, or better yet -- take no effect despite success responses
///
/// None is expecting any sort of error
mod coherence {}

/// This group of tests aim to ensure that returned data is interpreted correctly
///
/// These tests use some concrete site objects that are unlikely to change and ensure that
/// their parsed representations match expected locally-stored values
///
/// None is expecting any sort of error
mod correctness {
    use reqwest::Client;
    use time::{Date, Duration, Month};
    use type_matrux::{
        object::{ArticleSlug, TagSlug},
        DrukarniaApi,
    };

    use crate::setup_log;

    #[tokio::test]
    async fn get_article_should_be_correct() {
        // Arrange
        setup_log();
        let article_slug: ArticleSlug = "otrimaite-groshi-za-pereglyad-video-na-youtube-fMcYj"
            .parse()
            .unwrap();
        let client = Client::new();

        // Act
        let article = client.get_article(&article_slug).await.unwrap();

        // Assert
        assert!(*article.comment_num() > 0);
        assert_eq!(
            article.created_at().date(),
            Date::from_calendar_date(2023, Month::October, 2).unwrap()
        );
        assert_eq!(article.description().as_ref(), "Приготуйтеся стати найлінивішим мільйонером у світі! У цій статті я розкрию секретну стратегію, як заробляти гроші на PayPal, переглядаючи відео на YouTube. Саме так, тепер ви можете заробляти $219000 на рік, просто переглядаючи улюблені відео про котів.");
        // assert_eq!(article.like_num(), 0); // there are literally zero likes there, lol
        assert_eq!(article.main_tag().as_ref(), "Заробіток З Нуля");
        assert!(article.get_age() < Duration::SECOND); // this object was just fetched
        assert_eq!(article.owner().name().as_ref(), "Бізнес. Ідеї. Стартапи");
        assert_eq!(
            article.title().as_ref(),
            "Отримайте гроші за перегляд відео на YouTube"
        );
    }

    #[tokio::test]
    async fn get_tag_should_be_correct() {
        // Arrange
        setup_log();
        let tag_slug: TagSlug = "igri".parse().unwrap();
        let client = Client::new();

        // Act
        let tag = client.get_tag(&tag_slug).await.unwrap();

        // Assert
        assert_eq!(tag.name().as_ref(), "Ігри");
        assert!(*tag.mentions_num() >= 369);
        assert!(tag.get_age() < Duration::SECOND); // this object was just fetched
    }

    #[tokio::test]
    async fn get_user_should_be_correct() {
        // Arrange
        setup_log();
        let user_display_name: type_matrux::object::UserName = "drukarnia".parse().unwrap();
        let client = Client::new();

        // Act
        let user = client.get_user(&user_display_name).await.unwrap();

        // Assert
        assert_eq!(user.name().as_ref(), "Друкарня");
        assert!(user.articles().len() >= 5);
        assert_eq!(user.description().as_ref().unwrap().as_ref(), "Корисні довгочити, оновлення та поради по користуванню платформою. Основний профіль адміністрації Друкарні.");
        // assert!(*user.read_num() >= 2400);
        assert!(*user.followers_num() > 390);
        assert_eq!(
            user.created_at().date(),
            Date::from_calendar_date(2023, Month::April, 14).unwrap()
        );
        assert!(user.get_age() < Duration::SECOND); // this object was just fetched
    }
}

/// Other sort of tests I couldn't categorize
mod other {}
