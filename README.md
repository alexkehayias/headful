# Headful HTML to Markdown Converter

A Rust application that converts web pages to Markdown by launching a visible Chrome browser, navigating to a URL, and extracting content.

## Overview

This tool uses Chrome DevTools Protocol to control a headful (visible) Chrome browser instance. It navigates to the specified URL, waits for page load completion, extracts content, and converts it to clean Markdown format.

The converter supports two modes:
- **HTML mode** (default): Uses the `htmd` library to convert raw HTML to Markdown
- **AXTree mode** (`--axtree`): Uses Chrome's accessibility tree for more semantic, structured output

## Features

- Launches a visible Chrome browser for debugging/verification
- Waits for page navigation to complete before extracting content
- Converts HTML (or accessibility tree) to clean Markdown format
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

### Basic HTML conversion (default)

```bash
# Run directly with cargo
cargo run -- https://example.com

# Or run the compiled binary
./target/release/headful https://example.com
```

### AXTree (Accessibility Tree) mode

```bash
# Use Chrome's accessibility tree for semantic markdown extraction
cargo run -- https://example.com --axtree

# Or with the short flag
cargo run -- https://example.com -a
```

The accessibility tree mode extracts content based on the page's semantic structure (headings, links, paragraphs, lists, etc.) rather than raw HTML. This often produces cleaner output for complex websites.

Use an LLM to cleanup the content (good for news sites).

```bash
cargo run --features llm -- https://example.com --llm-endpoint https://api.openai.com/v1/chat/completions --api-key mykey
```

The converted Markdown content will be printed to stdout.

## How It Works

1. **Browser Launch**: Creates a headful Chrome browser instance using chromiumoxide
2. **Page Navigation**: Navigates to the provided URL and waits for navigation completion
3. **Content Extraction**:
   - HTML mode: Retrieves the complete HTML content of the page
   - AXTree mode: Uses Chrome's accessibility tree API to get semantic content
4. **HTML Filtering** (HTML mode only): Removes unwanted elements (scripts, styles, etc.)
5. **Markdown Conversion**:
   - HTML mode: Converts the filtered HTML to Markdown using htmd
   - AXTree mode: Parses accessibility tree nodes and converts semantic roles to Markdown
6. **[Optional] LLM Cleanup**: Clean up the markdown using a language model
7. **Output**: Prints the resulting Markdown to stdout

## AXTree Mode

When using `--axtree`, the converter uses Chrome's accessibility tree which represents the page's semantic structure. This mode handles:

- **Headings**: Converts to Markdown headers (h1-h6)
- **Links**: Preserves link text and URLs as `[text](url)`
- **Paragraphs**: Converts to plain text blocks
- **Lists**: Converts `<ul>` and `<ol>` elements to Markdown bullet lists
- **Buttons**: Renders as `[button text](button)`
- **Images**: Preserves alt text (when available)
- **Articles/Main content**: Extracts main content areas
- **Footers**: Marks footer sections

AXTree mode gracefully handles unknown node types by processing their children.

## Skipped HTML Elements (HTML mode)

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
- [clap](https://crates.io/crates/clap) - Command-line argument parsing

## License

MIT