use std::io;
use std::io::Write;
use futures_util::StreamExt;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tokio::task;
use htmd::HtmlToMarkdown;
use chromiumoxide::{Command, Method, browser::{Browser, BrowserConfig}};
use clap::Parser;

mod axtree;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetFullAxTree;

impl Command for GetFullAxTree {
    type Response = Value;
}

impl Method for GetFullAxTree {
    fn identifier(&self) -> chromiumoxide::types::MethodId {
        chromiumoxide::types::MethodId::Borrowed("Accessibility.getFullAXTree")
    }
}


#[cfg(feature = "llm")]
use reqwest::Client;
#[cfg(feature = "llm")]
use serde::{Deserialize, Serialize};


fn wait_for_enter(prompt: &str) -> io::Result<()> {
    print!("{prompt}");
    io::stdout().flush()?;          // Propagate any flushing error
    let mut line = String::new();
    io::stdin().read_line(&mut line)?; // Propagate read errors
    Ok(())
}

#[cfg(feature = "llm")]
#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
}

#[cfg(feature = "llm")]
#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[cfg(feature = "llm")]
#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[cfg(feature = "llm")]
#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[cfg(feature = "llm")]
#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

#[cfg(feature = "llm")]
async fn cleanup_with_llm(
    markdown: &str,
    endpoint: &str,
    api_key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let request_builder = client.post(endpoint)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key));

    let openai_request = OpenAIRequest {
        model: "openai/gpt-oss-120b".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant that cleans up and formats markdown content extracted from web pages. Remove any extraneous whitespace, fix broken links if possible, and ensure proper markdown formatting. Return only the cleaned markdown without any additional commentary.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: format!("Clean up this markdown that was converted from a website HTML, removing navigation and extraneous content:\n\n{}", markdown),
            },
        ],
    };

    let response = request_builder
        .json(&openai_request)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(format!("LLM API request failed with status {}: {}", status, error_text).into());
    }

    let openai_response: OpenAIResponse = response.json().await?;
    Ok(openai_response.choices
        .first()
        .ok_or("No choices in LLM response")?
        .message
        .content
        .clone())
}

/// Convert HTML web pages to Markdown format using a headful Chrome browser.
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
struct Cli {
    /// The URL to fetch and convert to Markdown
    url: String,

    /// Experimental: Use accessibility tree instead of HTML for
    /// markdown conversion
    #[arg(short, long)]
    axtree: bool,

    #[cfg(feature = "llm")]
    /// LLM API endpoint for markdown cleanup
    #[arg(short, long)]
    llm_endpoint: String,

    #[cfg(feature = "llm")]
    /// LLM API key
    #[arg(short, long)]
    api_key: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

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
    let page = browser.new_page(&cli.url).await?;
    let html = page.wait_for_navigation().await?.content().await?;
    let axt_value = page.execute(GetFullAxTree).await?;

    // Clean up
    browser.close().await?;
    let _ = handle.await;

    // Convert to markdown using accessibility tree or HTML
    let markdown_content = if cli.axtree {
        // Parse the accessibility tree from JSON value
        let axt_json = serde_json::to_string(&axt_value.result)?;
        let axt: axtree::AxTree = serde_json::from_str(&axt_json)?;
        eprintln!("Converted accessibility tree with {} nodes", axt.nodes.len());
        axtree::axtree_to_markdown(&axt)
    } else {
        // Convert HTML to markdown
        let converter = HtmlToMarkdown::builder()
            .skip_tags(vec!["script", "style", "footer", "img", "svg", "iframe", "head", "link"])
            .build();
        let markdown = converter.convert(&html)?;

        // Naive captcha detection and wait for the user to indicate they
        // completed it (only for HTML conversion)
        let markdown = if markdown.contains("CAPTCHA") {
            // This is blocking!
            wait_for_enter("Please complete the CAPTCHA and press return to continue")?;
            let html_after_captcha = page.wait_for_navigation().await?.content().await?;
            converter.convert(&html_after_captcha)?
        } else {
            markdown
        };

        markdown
    };

    // Clean up with LLM if feature is enabled
    #[cfg(feature = "llm")]
    {
        markdown_content = cleanup_with_llm(&markdown_content, &cli.llm_endpoint, &cli.api_key).await?;
    }

    println!("{}", markdown_content);
    Ok(())
}
