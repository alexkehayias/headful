# Headful HTML to Markdown Converter

A Rust application that converts web pages to Markdown by launching a visible Chrome browser, navigating to a URL, and extracting content.

## Overview

This tool uses Chrome DevTools Protocol to control a headful (visible) Chrome browser instance. It navigates to the specified URL, waits for page load completion, extracts the HTML content, and converts it to clean Markdown format.

## Features

- Launches a visible Chrome browser for debugging/verification
- Waits for page navigation to complete before extracting content
- Converts HTML to clean Markdown format
- Filters out unwanted elements (scripts, styles, etc.)
- Simple command-line interface

## Prerequisites

- Rust toolchain (latest stable version recommended)
- Chrome/Chromium browser installed on your system

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd headful

# Build the project
cargo build --release
```

## Usage

```bash
# Run directly with cargo
cargo run -- https://example.com

# Or run the compiled binary
./target/release/headful https://example.com
```

Use an LLM to cleanup the content (good for news sites).

```bash
cargo run --features llm -- https://example.com --llm-endpoint https://api.openai.com/v1/chat/completions --api-key mykey
```

The converted Markdown content will be printed to stdout.

## How It Works

1. **Browser Launch**: Creates a headful Chrome browser instance using chromiumoxide
2. **Page Navigation**: Navigates to the provided URL and waits for navigation completion
3. **Content Extraction**: Retrieves the complete HTML content of the page
4. **HTML Filtering**: Removes unwanted elements (scripts, styles, etc.)
5. **Markdown Conversion**: Converts the filtered HTML to Markdown using htmd
6. **[Optional] LLM Cleanup**: Clean up the markdown using a language model
7. **Output**: Prints the resulting Markdown to stdout

## Skipped HTML Elements

The following HTML elements are automatically filtered out during conversion:
- `<script>` - JavaScript code
- `<style>` - CSS styles
- `<footer>` - Page footers
- `<img>` - Images
- `<svg>` - SVG graphics
- `<iframe>` - Inline frames
- `<head>` - Document head
- `<link>` - External resource links

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Running Tests

```bash
cargo test
```

## Dependencies

- [tokio](https://crates.io/crates/tokio) - Async runtime
- [chromiumoxide](https://github.com/alexkehayias/chromiumoxide) - Chrome DevTools Protocol client
- [futures-util](https://crates.io/crates/futures-util) - Async utilities
- [htmd](https://crates.io/crates/htmd) - HTML to Markdown converter

## License

MIT
