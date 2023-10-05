use futures::StreamExt;
use type_matrux::client::ReqwestApi;
use type_matrux::DrukarniaApi;

#[tokio::main(flavor = "current_thread")]
pub async fn main() {
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
}
