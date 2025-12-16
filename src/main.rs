use std::env;
use std::io;
use std::io::Write;
use futures_util::StreamExt;
use tokio::task;
use htmd::HtmlToMarkdown;
use chromiumoxide::browser::{Browser, BrowserConfig};


fn wait_for_enter(prompt: &str) -> io::Result<()> {
    print!("{prompt}");
    io::stdout().flush()?;          // Propagate any flushing error
    let mut line = String::new();
    io::stdin().read_line(&mut line)?; // Propagate read errors
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let url = args.first().expect("Missing URL to fetch");

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

    // Fetch the page
    let page = browser.new_page(url).await?;
    let html = page.wait_for_navigation().await?.content().await?;

    // Convert HTML to markdown
    let converter = HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style", "footer", "img", "svg", "iframe", "head", "link"])
        .build();
    let mut markdown_content = converter.convert(&html)?;

    // Naive captcha detection and wait for the user to indicate they
    // completed it
    if markdown_content.contains("CAPTCHA") {
        // This is blocking!
        wait_for_enter("Please complete the CAPTCHA and press return to continue")?;
        let html_after_captcha = page.wait_for_navigation().await?.content().await?;
        markdown_content = converter.convert(&html_after_captcha)?;
    }

    // Clean up
    browser.close().await?;
    let _ = handle.await;

    println!("{}", markdown_content);
    Ok(())
}
