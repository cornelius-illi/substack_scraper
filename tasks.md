# Task List for Substack Scraper Enhancement

Based on the `plan.md`, here is a detailed task list for the implementation:

## 1. Project Setup & Dependency Management

-   [ ] **Task 1.1**: Add `html2md` dependency to `Cargo.toml`.
-   [ ] **Task 1.2**: Add `md5` dependency to `Cargo.toml`.
-   [ ] **Task 1.3**: Ensure `tokio`'s `fs` feature is enabled in `Cargo.toml`.

## 2. Implement Idempotency and Incremental Scraping

-   [ ] **Task 2.1**: Refactor `scrape` function in `src/main.rs` to iterate over `post_url`s individually.
-   [ ] **Task 2.2**: Inside the `scrape` loop, construct the target Markdown file path (`blogs/<blog_host>/p/<post_slug>.md`).
-   [ ] **Task 2.3**: Implement a check using `tokio::fs::metadata` to see if the Markdown file already exists.
-   [ ] **Task 2.4**: If the file exists, log a message and `continue` to the next `post_url`.

## 3. HTML to Markdown Conversion and Image Handling

-   [ ] **Task 3.1**: Modify `get_post_content` function:
    -   [ ] **Task 3.1.1**: Change its return type to `eyre::Result<String>`.
    -   [ ] **Task 3.1.2**: Broaden the `scraper` selector to capture the entire article body HTML (e.g., `.post-body`).
    -   [ ] **Task 3.1.3**: Remove the call to `cleanup_content`.
-   [ ] **Task 3.2**: Create a new `process_and_save_post` async function with parameters `post_url: &Url`, `homepage_url: &Url`, and `raw_html_content: String`.
    -   [ ] **Task 3.2.1**: Parse `raw_html_content` using `scraper::Html::parse_fragment`.
    -   [ ] **Task 3.2.2**: Find all `<img>` tags within the parsed HTML.
    -   [ ] **Task 3.2.3**: For each `<img>` tag:
        -   [ ] **Task 3.2.3.1**: Extract the `src` URL.
        -   [ ] **Task 3.2.3.2**: Create the `blogs/<blog_host>/attachments` directory if it doesn't exist using `tokio::fs::create_dir_all`.
        -   [ ] **Task 3.2.3.3**: Generate a unique local filename (e.g., using `md5(image_url)` and original file extension).
        -   [ ] **Task 3.2.3.4**: Construct the local image file path.
        -   [ ] **Task 3.2.3.5**: Download the image using `reqwest`.
        -   [ ] **Task 3.2.3.6**: Save the image to the local path using `tokio::fs::write`.
        -   [ ] **Task 3.2.3.7**: Update the `src` attribute of the `<img>` tag in the HTML fragment to point to the new relative local path (e.g., `../attachments/<hashed_filename>.<extension>`).
    -   [ ] **Task 3.2.4**: Convert the modified HTML content (after image `src` updates) to Markdown using `html2md::parse_html`.
    -   [ ] **Task 3.2.5**: Construct the final Markdown output file path (`blogs/<blog_host>/p/<post_slug>.md`).
    -   [ ] **Task 3.2.6**: Save the generated Markdown content to the file using `tokio::fs::write`.

## 4. Refactor `main` and `scrape` functions (Integration)

-   [ ] **Task 4.1**: Remove the old content collection and batch writing logic from the `scrape` function.
-   [ ] **Task 4.2**: Integrate `get_post_content` and `process_and_save_post` calls into the `scrape` function's loop, ensuring incremental processing.
-   [ ] **Task 4.3**: Review and enhance error handling throughout the modified code.
