use reqwest;
use serde::{Deserialize};
use regex::Regex;

use log::{debug};
use env_logger::Builder; // Added back

use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use std::thread::sleep;
use chrono::Local;
use scraper::{Html, Selector};

use tokio::fs; // Added back

use clap::{Parser};
use color_eyre::eyre;
use env_logger::Target::Stdout;

use reqwest::Url;
use md5;
use html2md;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// A space-delimited list of substack sites to scrape, such as "https://blog.bytebytego.com/ https://astralcodexten.substack.com/"
    #[clap(short, long, use_value_delimiter = true, value_delimiter = ' ')]
    websites: Vec<String>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> eyre::Result<()> {
    Builder::from_default_env()
        .format(|buf, record| {
            writeln!(buf,
                     "{} [{}] - {}",
                     Local::now().format("%Y-%m-%dT%H:%M:%S"),
                     record.level(),
                     record.args()
            )
        })
        .target(Stdout)
        .init();

    let cli = Cli::parse();

    debug!("Websites are {:?}", cli.websites);
    // Convert to Url type
    // Remove websites that are empty.
    let websites = cli.websites.iter().filter(|x| !x.is_empty());

    let websites = websites.into_iter().map(|s|
        Url::parse(&s).unwrap())
            .collect::<Vec<Url>>();

    // let join_handle = tokio::spawn(async move {
    for website in websites {
        scrape(&website).await.expect(&*format!("Failed to scrape {}", website));
    }
    // });

    // Wait for the async functions to complete.
    // join_handle.await.unwrap();
    Ok(())
}

#[derive(Deserialize)]
#[derive(Debug)]
struct CanonicalUrl {
    canonical_url: Url,
}

async fn scrape(homepage_url: &Url) -> eyre::Result<()> {
    let post_urls = get_post_urls(homepage_url).await?;

    let blog_folder_path = Path::new("blogs").join(Path::new(&homepage_url.host_str().unwrap()));

    for post_url in &post_urls {
        let path = Path::new(post_url.path());
        let path = path.strip_prefix("/").unwrap_or(path);
        let path_with_md_extension = blog_folder_path.join(path).with_extension("md");

        if let Ok(metadata) = fs::metadata(&path_with_md_extension).await { // Use tokio::fs::metadata
            if metadata.is_file() {
                debug!("Skipping already scraped post: {}", post_url);
                continue;
            }
        }

        let post_content = get_post_content(&post_url).await?;
        process_and_save_post(&post_url, homepage_url, post_content).await?;
    }
    Ok(())
}

/// Get the text content of a post.
async fn get_post_content(url: &Url) -> eyre::Result<String> {
    debug!("Fetching post content for URL: {}", url);

    loop {
        let headers = reqwest::get(url.clone()).await?;
        let body = headers.text().await?;

        let fragment = Html::parse_fragment(&body);
        // Broaden the selector to capture the entire content div.
        // Based on analysis of Substack HTML, '.available-content' often contains the main post.
        let selector = Selector::parse(".available-content").unwrap();

        if let Some(main_content_element) = fragment.select(&selector).next() {
            return Ok(main_content_element.inner_html());
        }

        debug!("No content found with selector. Retrying...");
        sleep(std::time::Duration::from_secs(1));
    }
}



async fn process_and_save_post(
    post_url: &Url,
    homepage_url: &Url,
    raw_html_content: String,
) -> eyre::Result<()> {
    debug!("Processing and saving post: {}", post_url);

    let blog_host = homepage_url.host_str().unwrap_or("unknown_host");
    let blog_folder_path = Path::new("blogs").join(blog_host);
    let attachments_dir = blog_folder_path.join("attachments");

    tokio::fs::create_dir_all(&attachments_dir).await?;

    let fragment = Html::parse_fragment(&raw_html_content);
    let mut img_replacements = Vec::new();
    let img_selector = Selector::parse("img").unwrap();

    for element_ref in fragment.select(&img_selector) {
        if let Some(src) = element_ref.value().attr("src") {
            let img_url = Url::parse(src)?;
            let img_bytes = reqwest::get(img_url.clone()).await?.bytes().await?;

            let img_filename = format!("{:x}", md5::compute(src.as_bytes()));
            let img_extension = img_url
                .path_segments()
                .and_then(|segments| segments.last())
                .and_then(|name| name.rsplit('.').next())
                .unwrap_or("bin");

            let local_img_path = attachments_dir.join(format!("{}.{}", img_filename, img_extension));

            tokio::fs::write(&local_img_path, img_bytes).await?;
            debug!("Downloaded image: {} to {:?}", img_url, local_img_path);

            let relative_img_path = Path::new("../attachments")
                .join(format!("{}.{}", img_filename, img_extension))
                .to_str()
                .unwrap()
                .to_string();

            // Store original src and new relative path for replacement
            img_replacements.push((src.to_string(), relative_img_path));
        }
    }

    // Apply replacements to the HTML content string
    let mut modified_html_content = raw_html_content;
    for (original_src, new_src) in img_replacements {
        // This is a simple string replacement; a more robust solution might involve re-rendering the DOM
        // or using a more sophisticated HTML manipulation library if complex scenarios arise.
        // For now, this assumes 'src' attributes are quoted and distinct enough.
        modified_html_content = modified_html_content.replace(&format!("src=\"{}\"", original_src), &format!("src=\"{}\"", new_src));
    }


    // Convert modified HTML to Markdown
    let markdown_content = html2md::parse_html(&modified_html_content);

    // Post-process markdown to fix image links
    let re = Regex::new(r#"\[\s*<img[^>]*src="([^"]+)"[^>]*>\s*\]\([^)]+\)"#).unwrap();
    let markdown_content = re.replace_all(&markdown_content, "![]($1)").to_string();

    // Save Markdown file
    let post_file_path = blog_folder_path.join(post_url.path().strip_prefix("/").unwrap_or(post_url.path())).with_extension("md");
    tokio::fs::create_dir_all(post_file_path.parent().unwrap()).await?;
    tokio::fs::write(&post_file_path, markdown_content.as_bytes()).await?;
    debug!("Saved markdown for post: {} to {:?}", post_url, post_file_path);

    Ok(())
}

async fn get_post_urls(homepage_url: &Url) -> eyre::Result<HashSet<Url>> {
    debug!("Scraping {}", homepage_url);

    // Current page number.
    let mut page_offset = 0;
    // Pages to request on each iteration.
    let page_limit = 12;

    // Contains the hashset of article URLs.
    let mut seen_urls = HashSet::new();

    loop {
        // Get content. The api url may be subject to change from Substack.
        let current_request_url = format!("{}api/v1/archive?sort=new&search=&offset={}&limit={}", homepage_url, page_offset, page_limit);
        debug!("current_request_url = {}", &current_request_url);

        let post_urls = reqwest::get(&current_request_url)
            .await?.
            json::<Vec<CanonicalUrl>>()
            .await?;

        // Add page URLs.
        // Exit on empty query.
        if post_urls.is_empty() {
            break;
        }
        seen_urls.extend(post_urls.into_iter().map(|it| it.canonical_url));

        page_offset += page_limit;
    }
    debug!("seen_urls = {seen_urls:?}");

    debug!("Finished scraping {}", homepage_url);
    Ok(seen_urls)
}
