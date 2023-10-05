use std::num::NonZeroUsize;

use futures::StreamExt;
use type_matrux::client::ReqwestApi;
use type_matrux::DrukarniaApi;

#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    let client = ReqwestApi::new();
    let page = client
        .feed_page(NonZeroUsize::new(1).expect("1 != 0"))
        .await
        .unwrap();
    println!("first acticle is {:#?}", page[0]);

    let page_stream = client.feed();
    let eleventh_page = page_stream.skip(10).next().await.unwrap().unwrap();
    println!("first article on eleventh page is {:#?}", eleventh_page[0]);

    let article_stream = client.feed().flat();
    let hundredth_article = article_stream.skip(99).next().await.unwrap().unwrap();
    println!("hundredth article is {:#?}", hundredth_article);
}
