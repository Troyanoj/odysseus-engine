use reqwest;
use scraper::{Html, ElementRef, Node, Selector};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::OnceLock;
use std::time::Duration;
use url::Url;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use moka::future::Cache;
use rand::Rng;
use tokio::time::sleep;
use flate2::read::GzDecoder;
use std::io::Read;

use crate::proxy_manager::ProxyConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageData {
    pub url: String,
    pub title: String,
    pub description: String,
    pub metadata: HashMap<String, String>,
    pub markdown: String,
}

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub fn get_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client")
    })
}

static PROXY_CONFIG: OnceLock<ProxyConfig> = OnceLock::new();

pub fn init_proxy_config(proxies: Vec<String>) {
    PROXY_CONFIG.get_or_init(|| ProxyConfig::new(proxies));
}

fn build_client_with_proxy() -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(10));
    
    // Si hay proxies configurados, usar el siguiente
    if let Some(proxy_config) = PROXY_CONFIG.get() {
        if let Some(proxy_url) = proxy_config.get_next_proxy() {
            if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                builder = builder.proxy(proxy);
            }
        }
    }
    
    builder.build().expect("Failed to build HTTP client")
}

fn get_client_or_proxy() -> reqwest::Client {
    if PROXY_CONFIG.get().map(|p| p.has_proxies()).unwrap_or(false) {
        build_client_with_proxy()
    } else {
        get_client().clone()
    }
}

/// Genera cabeceras HTTP que imitan a Chrome 120 en Windows
fn generate_chrome_headers() -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    
    headers.insert(
        reqwest::header::USER_AGENT,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".parse().unwrap()
    );
    headers.insert(
        reqwest::header::ACCEPT,
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8".parse().unwrap()
    );
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        "es-ES,es;q=0.9,en-US;q=0.8,en;q=0.7".parse().unwrap()
    );
    headers.insert(
        reqwest::header::ACCEPT_ENCODING,
        "gzip, deflate".parse().unwrap()
    );
    headers.insert(
        "sec-ch-ua",
        "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"".parse().unwrap()
    );
    headers.insert(
        "sec-ch-ua-mobile",
        "?0".parse().unwrap()
    );
    headers.insert(
        "sec-ch-ua-platform",
        "\"Windows\"".parse().unwrap()
    );
    headers.insert(
        "sec-fetch-dest",
        "document".parse().unwrap()
    );
    headers.insert(
        "sec-fetch-mode",
        "navigate".parse().unwrap()
    );
    headers.insert(
        "sec-fetch-site",
        "none".parse().unwrap()
    );
    headers.insert(
        "sec-fetch-user",
        "?1".parse().unwrap()
    );
    headers.insert(
        "upgrade-insecure-requests",
        "1".parse().unwrap()
    );
    headers.insert(
        "cache-control",
        "max-age=0".parse().unwrap()
    );
    
    headers
}

static PAGE_CACHE: OnceLock<Cache<String, PageData>> = OnceLock::new();

fn get_cache() -> &'static Cache<String, PageData> {
    PAGE_CACHE.get_or_init(|| {
        Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(300))
            .time_to_idle(Duration::from_secs(60))
            .build()
    })
}

pub async fn extract_page_data(url: &str) -> Result<PageData> {
    if let Some(cached_data) = get_cache().get(url).await {
        return Ok(cached_data);
    }
    
    // Delay aleatorio "humano" entre 500ms y 2000ms
    let delay_ms = rand::thread_rng().gen_range(500..=2000);
    sleep(Duration::from_millis(delay_ms)).await;
    
    let client = get_client_or_proxy();
    let chrome_headers = generate_chrome_headers();
    
    let response = client.get(url)
        .headers(chrome_headers)
        .send()
        .await?;
    
    let status = response.status();
    let content_encoding = response.headers().get(reqwest::header::CONTENT_ENCODING).cloned();
    tracing::info!("Response status: {}, content-encoding: {:?}", status, content_encoding);
    
    let response = response.error_for_status()?;
    
    let final_url = response.url().clone();
    
    // Leer bytes y descomprimir gzip manualmente si es necesario
    let bytes = response.bytes().await?;
    let html_content = if content_encoding.as_ref().map(|v| v.to_str().unwrap_or("")).unwrap_or("").contains("gzip") {
        let mut decoder = GzDecoder::new(&bytes[..]);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed)?;
        decompressed
    } else {
        String::from_utf8(bytes.to_vec())?
    };
    
    tracing::info!("HTML content length: {}", html_content.len());
    
    let result = parse_html_to_page_data(&html_content, &final_url)?;
    
    get_cache().insert(url.to_string(), result.clone()).await;
    
    Ok(result)
}

pub async fn extract_multiple_page_data(urls: Vec<String>) -> Result<Vec<Result<PageData>>> {
    let mut join_set = tokio::task::JoinSet::new();
    
    for url in urls {
        join_set.spawn(async move {
            match extract_page_data(&url).await {
                Ok(data) => Ok(data),
                Err(e) => Err(e),
            }
        });
    }
    
    let mut results = Vec::new();
    while let Some(res) = join_set.join_next().await {
        match res {
            Ok(task_result) => results.push(task_result),
            Err(e) => results.push(Err(anyhow::anyhow!("Task panicked: {}", e))),
        }
    }
    
    Ok(results)
}

fn parse_html_to_page_data(html_content: &str, final_url: &Url) -> Result<PageData> {
    let document = Html::parse_document(html_content);
    
    let title = extract_title(&document).unwrap_or_default();
    let description = extract_description(&document).unwrap_or_default();
    let metadata = extract_metadata(&document);
    
    let mut markdown = String::new();
    let mut seen_links = HashSet::new();
    let body_selector = Selector::parse("body").unwrap();
    
    if let Some(body_element) = document.select(&body_selector).next() {
        markdown.push_str(&element_to_markdown(body_element, &mut seen_links, final_url));
    } else {
        for child in document.root_element().children() {
            if let Some(elem_ref) = ElementRef::wrap(child) {
                markdown.push_str(&element_to_markdown(elem_ref, &mut seen_links, final_url));
            }
        }
    }
    
    Ok(PageData {
        url: final_url.to_string(),
        title,
        description,
        metadata,
        markdown,
    })
}

fn extract_title(document: &Html) -> Option<String> {
    let title_selector = Selector::parse("head > title").ok()?;
    document.select(&title_selector).next()
        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
}

fn extract_description(document: &Html) -> Option<String> {
    let desc_selector = Selector::parse("head > meta[name=\"description\"]").ok()?;
    document.select(&desc_selector).next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.trim().to_string())
}

fn extract_metadata(document: &Html) -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    let meta_selector = Selector::parse("head > meta").unwrap();
    
    for el in document.select(&meta_selector) {
        let value = el.value();
        
        if let Some(prop) = value.attr("property") {
            if prop.starts_with("og:") || prop.starts_with("twitter:") {
                if let Some(content) = value.attr("content") {
                    metadata.insert(prop.to_string(), content.to_string());
                }
            }
        }
        else if let Some(name) = value.attr("name") {
            if name.starts_with("twitter:") || name == "description" || name == "viewport" || name == "robots" {
                if let Some(content) = value.attr("content") {
                    metadata.insert(name.to_string(), content.to_string());
                }
            }
        }
    }
    
    let json_ld_selector = Selector::parse("script[type=\"application/ld+json\"]").unwrap();
    let mut json_lds = Vec::new();
    for el in document.select(&json_ld_selector) {
        let text = el.text().collect::<Vec<_>>().join(" ").trim().to_string();
        if !text.is_empty() {
            json_lds.push(text);
        }
    }
    if !json_lds.is_empty() {
        metadata.insert("json_ld".to_string(), json_lds.join("\n"));
    }
    
    metadata
}

fn element_to_markdown(element: ElementRef, seen_links: &mut HashSet<String>, base_url: &Url) -> String {
    let tag_name = element.value().name();
    
    if tag_name == "script" || tag_name == "style" {
        return String::new();
    }

    let mut inner_text = String::new();
    for child in element.children() {
        match child.value() {
            Node::Text(text) => {
                inner_text.push_str(text.text.as_ref());
            }
            Node::Element(_) => {
                if let Some(child_elem) = ElementRef::wrap(child) {
                    inner_text.push_str(&element_to_markdown(child_elem, seen_links, base_url));
                }
            }
            _ => {}
        }
    }

    let inner_text = inner_text.trim().to_string();

    match tag_name {
        "h1" => if !inner_text.is_empty() { format!("# {}\n\n", inner_text) } else { String::new() },
        "h2" => if !inner_text.is_empty() { format!("## {}\n\n", inner_text) } else { String::new() },
        "h3" => if !inner_text.is_empty() { format!("### {}\n\n", inner_text) } else { String::new() },
        "h4" => if !inner_text.is_empty() { format!("#### {}\n\n", inner_text) } else { String::new() },
        "h5" => if !inner_text.is_empty() { format!("##### {}\n\n", inner_text) } else { String::new() },
        "h6" => if !inner_text.is_empty() { format!("###### {}\n\n", inner_text) } else { String::new() },
        "p" => if !inner_text.is_empty() { format!("{}\n\n", inner_text) } else { String::new() },
        "br" => "\n".to_string(),
        "ul" | "ol" => if !inner_text.is_empty() { format!("{}\n", inner_text) } else { String::new() },
        "li" => if !inner_text.is_empty() { format!("- {}\n", inner_text) } else { String::new() },
        "strong" | "b" => if !inner_text.is_empty() { format!("**{}**", inner_text) } else { String::new() },
        "em" | "i" => if !inner_text.is_empty() { format!("*{}*", inner_text) } else { String::new() },
        "code" => if !inner_text.is_empty() { format!("`{}`", inner_text) } else { String::new() },
        "pre" => if !inner_text.is_empty() { format!("```\n{}\n```\n\n", inner_text) } else { String::new() },
        "img" => {
            if let Some(alt) = element.value().attr("alt") {
                if !alt.trim().is_empty() {
                    format!(" [Imagen: {}] ", alt.trim())
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        }
        "a" => {
            if let Some(href) = element.value().attr("href") {
                let absolute_href = base_url.join(href).map(|u| u.to_string()).unwrap_or_else(|_| href.to_string());
                let link_key = format!("{}|{}", inner_text, absolute_href);
                if !inner_text.is_empty() && seen_links.insert(link_key) {
                    format!("[{}]({})", inner_text, absolute_href)
                } else {
                    inner_text
                }
            } else {
                inner_text
            }
        }
        _ => inner_text,
    }
}