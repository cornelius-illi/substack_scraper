# Gemini Code Assistant Context

This document provides context for the Gemini code assistant to understand the `substack_scraper` project.

## Project Overview

`substack_scraper` is a command-line utility written in Rust designed to scrape all posts from one or more Substack blogs. It downloads the content of each post and saves it as a separate plain-text file. The primary goal of this project was to generate training data for neural networks.

The application works by:
1.  Taking a space-separated list of Substack homepage URLs as command-line arguments.
2.  For each blog, it queries the public `/api/v1/archive` endpoint to fetch a complete list of all post URLs.
3.  It then visits each post URL, parses the HTML, and extracts the main article content.
4.  The extracted content is cleaned by stripping HTML tags and other artifacts.
5.  Finally, the clean text is saved to the local filesystem under a `blogs/` directory, with a subdirectory for each blog, mirroring the original URL structure.

The project uses the `tokio` runtime for asynchronous network requests, `clap` for command-line argument parsing, `reqwest` as the HTTP client, and `scraper` for HTML parsing.

**Limitation**: The scraper cannot access content behind subscriber-only paywalls. It will only save the publicly available preview text.

## Building and Running

The project is a standard Rust application managed with Cargo.

### Build

To build the project:
```bash
cargo build
```

For an optimized release build:
```bash
cargo build --release
```

### Run

To run the scraper, use `cargo run` with the `-w` or `--websites` flag, followed by a space-separated list of Substack URLs.

```bash
# Example
cargo run -- -w "https://blog.bytebytego.com/ https://astralcodexten.substack.com/"
```

### Debug Logging

To enable verbose debug logging, set the `RUST_LOG` environment variable to `debug`.

```bash
RUST_LOG=debug cargo run -- -w "https://blog.bytebytego.com/"
```

## Development Conventions

### Testing

There are currently no automated tests in the project. The standard command to run tests would be:
```bash
cargo test
```

### Linting & Formatting

The project follows standard Rust conventions. Use `cargo clippy` for linting and `cargo fmt` for formatting.

To run the linter:
```bash
cargo clippy
```

To format the code:
```bash
cargo fmt
```
