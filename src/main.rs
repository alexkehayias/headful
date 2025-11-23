use std::env;
use futures_util::StreamExt;
use tokio::task;
use htmd::HtmlToMarkdown;
use chromiumoxide::browser::{Browser, BrowserConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let url = args.first().unwrap();

    // Create a headful chromium browser and the handler to drive the
    // browser via websocket
    let (mut browser, mut handler) =
        Browser::launch(BrowserConfig::builder().with_head().build()?).await?;
    let handle = task::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    let page = browser.new_page(url).await?;
    let html = page.wait_for_navigation().await?.content().await?;
    browser.close().await?;
    let _ = handle.await;

    let converter = HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style", "footer", "img", "svg", "iframe", "head", "link"])
        .build();
    let markdown_content = converter.convert(&html).unwrap();

    println!("{}", markdown_content);
    Ok(())
}
