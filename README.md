# Brief
`type_matrux` allows you to search and query data on [`Drukarnia`](https://drukarnia.com.ua/) website.
Although it's not currently implemented, there's a plan to make it also possible to authorize on a website,
and perform any operation one would do with on the actual website (post articles and comments, and interact with other users).
I aimed for the most accurate fetched data representation,
so that there would be no surprizes, like missing/redundant fields, or unexpected field format. To test for that, there are "reinforcement tests" in the test suite -
these attempt to deserialize a large returned data amount to prove that it's representation is indeed correct.
## Please note
I'm **NOT** affiliated with Drukarnia's developes in any way. In fact, they are **not happy** about people creating their own
API adapters and do not provide any sort of API documentation. That said, please consider two things:
- The crate can be easily abused to perform illegal actions and I'm not responsible for anything you do.
Please, respect Drukarnia's policies, as well as privacy of other users.
- I don't know full Drukarnia's API capabilities - this crate only allows you to send requests explicitly used by their front-end
(since these are the only ones I could capture).
# Details
You might be overwhelmed by the number of data structures defined. This is caused by Drukarnia returning just a little different fields list for each type of the request.
Since I already claimed that my data representation is accurate, I'd need to define a separate data structure for each request type (most of the time).
Also, this crate features simple newtypes for almost any field. While this might be annoying at first, It saved me a couple of times from passing wrong id types for a request.
Right now, authorized operations are implemented, but not yet tested and proved to work correctly.
To prevent crate users utilizing it, [`DrukarniaApi::login`] implementation panics now. Hope to change that soon.
Also, be careful to not make Drukarnia suspicious
(while testing, I got to the point of Drukarnia denying authorization, claiming that I've done it too many times already).
The actual API currently can only be accessed with a [`reqwest::Client`] (also reexported as [`ReqwestApi`]).
Any API accessor should implement [`DrukarniaApi`] trait. This allows one to mock the accessor, or create your own accessor extensions.
# Examples
You may refer to [`DrukarniaApi`] documentation as well as [`crate::object`] module documentation for a full list
of requests you may send and data received from them.
## Feed fetching
Lets say, we want to get some articles from a Feed. To do that, you may use [`DrukarniaApi::feed_page`] or [`DrukarniaApi::feed`].
As you might guess, the first one returns you a single feed page:
```rust
# use reqwest::Client;
# use type_matrux::client::DrukarniaApi;
# use std::num::NonZeroUsize;
# #[tokio::test]
# async fn test() {
let client = ReqwestApi::new();
let page = client.feed_page(NonZeroUsize::new(1).expect("1 != 0")).await.unwrap();
println!("first article is {:#?}", page[0]);
# }
```
While second one creates a stream of pages:
```rust
# use reqwest::Client;
# use type_matrux::client::DrukarniaApi;
# use std::num::NonZeroUsize;
# #[tokio::test]
# async fn test() {
let client = ReqwestApi::new();
let page_stream = client.feed();
let eleventh_page = page_stream.skip(10).next().await.unwrap().unwrap();
println!("first article on eleventh page is {:#?}", eleventh_page[0]);
# }
```
There's also an option to flatten the stream of pages into a stream of separate articles:
```rust
# use reqwest::Client;
# use type_matrux::client::DrukarniaApi;
# use std::num::NonZeroUsize;
# #[tokio::test]
# async fn test() {
let client = ReqwestApi::new();
let article_stream = client.feed().flat();
//                                 ^^^^^^ note this call
let hundredth_article = article_stream.skip(99).next().await.unwrap().unwrap();
println!("hundredth article is {:#?}", hundredth_article);
# }
```
Full code is available at `examples/feed.rs` for a closer inspection
## Averages
Let's say we want to find an average number of likes, max number of comments and average number of author reads for all
articles searched as having "Дія" in their title.
To do that, we can use [`DrukarniaApi::search_article_page`]/[`DrukarniaApi::search_article`] (having same sort of behavior, as discussed above with feed):
```rust
# use futures::StreamExt;
# use reqwest::Client;
# use type_matrux::DrukarniaApi;
# #[tokio::test]
# async fn test() {
let client = ReqwestApi::new();
let search_name = "Дія".parse().unwrap();
let mut articles = client.search_article(search_name).flat().take(500);
let mut total_likes = 0;
let mut max_comments = 0;
let mut total_reads = 0;
let mut total_articles = 0;
while let Some(Ok(article)) = articles.next().await {
    total_articles += 1;
    total_likes += article.like_num();
    max_comments = std::cmp::max(max_comments, *article.comment_num());
    total_reads += article.owner().read_num();
}
println!("{} articles processed", total_articles);
println!(
    "average like num: {}",
    (total_likes as f64) / (total_articles as f64)
);
println!("max comments: {}", max_comments);
println!(
    "average author reads: {}",
    (total_reads as f64) / (total_articles as f64)
);
# }
```
Full code is available at `examples/averages.rs` for closer inspection
# On further developing
This crate was created as my Rust learning project, and thus might not be updated in the future. These tests take a long time to run, so they are ignored by default.
As I mentioned, there's no contact between me and the developers, and thus these data representations might become invalid after some time.
In this case, I'd like to be notified on the issue tracker (if not stated otherwise).
For now, developed feature list is as follows:
- [ ] Add article content typing (it's returned in a sort of weird form, that feels like JSON-serialized HTML).
- [ ] Add validation to String newtypes. This requires some real effort,
as there's little known about limitations on things like Descriptions and DisplayNames
- [ ] Add procedural macro for compile-time verification of object ids, slugs, etc.
This will help user to know that id/slug does not exist at compile-time!
- [ ] Implement auth operations.
- [ ] Add API implementations for other popular HTTP clients like `isahc` and `surf`.