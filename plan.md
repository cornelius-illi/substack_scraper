# Implementation Plan: Substack Scraper Enhancement

This document outlines the plan to enhance the `substack_scraper` tool with the following features:

-   Markdown output instead of plain text.
-   Image scraping and local storage.
-   Incremental writing for each post.
-   Idempotency to avoid re-scraping existing content.

## 1. Project Setup & Dependency Management

-   **Add new dependencies to `Cargo.toml`**:
    -   `html2md`: For converting post HTML into Markdown.
    -   `md5`: To hash image URLs for unique local filenames.
    -   We will also make sure that `tokio`'s `fs` feature is enabled for async file operations.

## 2. Implement Idempotency and Incremental Scraping

-   **Refactor the `scrape` function in `src/main.rs`**:
    -   The main loop will iterate over each `post_url` obtained from `get_post_urls`.
    -   **Check for existing file**: Inside the loop, for each `post_url`, construct the final markdown file path (`blogs/<blog_host>/p/<post_slug>.md`).
    -   Use `tokio::fs::metadata` to check if the file already exists. If it does, log a message and `continue` to the next URL. This makes the scraping process idempotent.
    -   If the file does not exist, proceed with fetching the content. This ensures we write to disk after processing each article.

## 3. HTML to Markdown Conversion and Image Handling

-   **Update `get_post_content` function**:
    -   Change the return type to `eyre::Result<String>`.
    -   Instead of selecting only `<p>` tags, the selector will be broadened to grab the entire article body, for example, by selecting the element with class `.post-body`. This will give us the full HTML content, including image tags.
    -   The `cleanup_content` function will be removed, as `html2md` will handle the conversion.

-   **Create a new `process_and_save_post` async function**:
    -   This function will take the `post_url`, the `homepage_url`, and the raw HTML `content` of the post as input.
    -   **Image Scraping**:
        -   It will parse the `content` HTML with the `scraper` crate.
        -   It will find all `<img>` tags within the content.
        -   For each `<img>` tag:
            1.  Extract the `src` URL.
            2.  Create the `blogs/<blog_host>/attachments` directory if it doesn't exist.
            3.  Generate a unique local filename for the image by hashing its URL (e.g., `md5(image_url)`).
            4.  Construct the local image path: `blogs/<blog_host>/attachments/<hashed_filename>.<extension>`.
            5.  Download the image from its `src` URL using `reqwest`.
            6.  Save the image to the local path using `tokio::fs`.
            7.  Replace the `src` attribute in the HTML with the new relative path to the local image (e.g., `../attachments/<hashed_filename>.<extension>`).
    -   **Markdown Conversion**:
        -   After updating all image `src` attributes, convert the modified HTML content to Markdown using the `html2md::parse_html` function.
    -   **Save Markdown File**:
        -   Construct the final output path: `blogs/<blog_host>/p/<post_slug>.md`.
        -   Save the generated Markdown to this file.

## 4. Refactor `main` and `scrape` functions

-   **Modify the `scrape` function**:
    -   The loop that currently calls `get_post_content` for all posts and stores them in a `Vec` will be removed.
    -   Instead, it will loop through the `post_urls`, and for each URL, it will:
        1.  Perform the idempotency check.
        2.  Call `get_post_content`.
        3.  Call the new `process_and_save_post` function.
    -   This change facilitates incremental writing.

-   **Error Handling**:
    -   Ensure robust error handling for network requests, file I/O, and parsing throughout the new implementation.

By following these steps, the scraper will be updated to produce Markdown files with locally saved images, and it will be able to resume scraping without re-downloading content.
