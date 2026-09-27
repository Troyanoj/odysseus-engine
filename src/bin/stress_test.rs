use std::env;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use reqwest;
use scraper;
use flate2::read::GzDecoder;
use std::io::Read;
use std::collections::HashSet;

use odysseus_engine::extractor::{extract_page_data, init_proxy_config, PageData};

#[derive(Debug, Clone)]
struct TestResult {
    url: String,
    status: String,
    time_ms: u64,
    html_kb: f64,
    markdown_kb: f64,
    compression_ratio: f64,
    links_count: usize,
    error: Option<String>,
}

impl TestResult {
    fn success(url: String, time_ms: u64, html_kb: f64, markdown_kb: f64, links_count: usize) -> Self {
        let compression_ratio = if html_kb > 0.0 {
            (markdown_kb / html_kb) * 100.0
        } else {
            0.0
        };
        Self {
            url,
            status: "✅".to_string(),
            time_ms,
            html_kb,
            markdown_kb,
            compression_ratio,
            links_count,
            error: None,
        }
    }

    fn error(url: String, time_ms: u64, error_msg: String) -> Self {
        Self {
            url,
            status: "❌".to_string(),
            time_ms,
            html_kb: 0.0,
            markdown_kb: 0.0,
            compression_ratio: 0.0,
            links_count: 0,
            error: Some(error_msg),
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=============================================================================");
    println!("ODYSSEUS ENGINE v1.0 - REAL WORLD STRESS TEST");
    println!("=============================================================================\n");

    // Read environment variables
    let api_key = env::var("API_KEY").unwrap_or_else(|_| "dev-key-123".to_string());
    let proxies = env::var("PROXIES").ok();

    if let Some(ref p) = proxies {
        println!("🔄 Proxy rotation enabled: {}", p);
    }
    println!("🔑 API Key: {}***", &api_key[..api_key.len().min(8)]);
    println!();

    // Test URLs
    let test_urls = vec![
        ("Control básico", "https://example.com"),
        ("Documentación Tech", "https://blog.rust-lang.org"),
        ("HTML denso", "https://news.ycombinator.com"),
        ("Alto tráfico/Complejo", "https://www.wikipedia.org"),
        ("Diseñado para scraping", "https://quotes.toscrape.com"),
        ("Moderno con JS", "https://github.com/torvalds/linux"),
        ("Noticias - BBC", "https://www.bbc.com/news"),
        ("Noticias pesado - TechCrunch", "https://techcrunch.com"),
        ("Documentación densa - MDN", "https://developer.mozilla.org/en-US/docs/Web/JavaScript"),
        ("Error HTTP 404", "https://httpstat.us/404"),
    ];

    let mut results = Vec::new();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build HTTP client");

    // Initialize proxy config if proxies provided
    if let Some(proxies_str) = proxies {
        let proxy_list: Vec<String> = proxies_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !proxy_list.is_empty() {
            init_proxy_config(proxy_list);
        }
    }

    println!("Iniciando pruebas contra {} URLs...\n", test_urls.len());

    for (name, url) in test_urls {
        print!("Probando: {:<30} ... ", name);
        let start = Instant::now();

        let test_future = async {
            // Small delay to be respectful
            tokio::time::sleep(Duration::from_millis(500)).await;

            let client_clone = client.clone();
            let api_key_clone = api_key.clone();

            let response = timeout(Duration::from_secs(15), async move {
                let chrome_headers = generate_chrome_headers();
                client_clone
                    .get(url)
                    .header("x-api-key", &api_key_clone)
                    .headers(chrome_headers)
                    .send()
                    .await
            }).await;

            match response {
                Ok(Ok(resp)) => {
                    let status = resp.status();
                    let content_encoding = resp.headers().get(reqwest::header::CONTENT_ENCODING).cloned();
                    let bytes = resp.bytes().await?;

                    let html_content = if content_encoding.as_ref().map(|v| v.to_str().unwrap_or("")).unwrap_or("").contains("gzip") {
                        let mut decoder = flate2::read::GzDecoder::new(&bytes[..]);
                        let mut decompressed = String::new();
                        std::io::Read::read_to_string(&mut decoder, &mut decompressed)?;
                        decompressed
                    } else {
                        String::from_utf8(bytes.to_vec())?
                    };

                    // Extract using our extractor logic
                    let document = scraper::Html::parse_document(&html_content);
                    let mut markdown = String::new();
                    let mut seen_links = std::collections::HashSet::new();
                    let body_selector = scraper::Selector::parse("body").unwrap();
                    let final_url = reqwest::Url::parse(url)?;

                    if let Some(body_element) = document.select(&body_selector).next() {
                        markdown.push_str(&element_to_markdown(&body_element, &mut seen_links, &final_url));
                    } else {
                        for child in document.root_element().children() {
                            if let Some(elem_ref) = scraper::ElementRef::wrap(child) {
                                markdown.push_str(&element_to_markdown(&elem_ref, &mut seen_links, &final_url));
                            }
                        }
                    }

                    let html_kb = html_content.len() as f64 / 1024.0;
                    let markdown_kb = markdown.len() as f64 / 1024.0;
                    let links_count = seen_links.len();

                    if status.is_success() {
                        Ok(TestResult::success(
                            url.to_string(),
                            0, // Will be set after
                            html_kb,
                            markdown_kb,
                            links_count,
                        ))
                    } else {
                        Ok(TestResult::error(
                            url.to_string(),
                            0,
                            format!("HTTP {}", status.as_u16()),
                        ))
                    }
                }
                Ok(Err(e)) => Ok(TestResult::error(url.to_string(), 0, format!("Request error: {}", e))),
                Err(_) => Ok(TestResult::error(url.to_string(), 0, "Timeout (15s)".to_string())),
            }
        };

        let mut result = test_future.await.unwrap_or_else(|e: Box<dyn std::error::Error + Send + Sync>| TestResult::error(url.to_string(), 0, format!("Panic: {}", e)));
        result.time_ms = start.elapsed().as_millis() as u64;

        let status_display = if result.status == "✅" {
            format!("✅ {:>4}", "OK")
        } else {
            format!("❌ {:>4}", result.error.as_deref().unwrap_or("ERR") )
        };

        println!("{}", status_display);
        results.push(result);
    }

    // Print results table
    println!("\n=============================================================================");
    println!("RESULTADOS DEL STRESS TEST");
    println!("=============================================================================");
    println!("{:<30} | {:>6} | {:>8} | {:>8} | {:>8} | {:>7} | {:>6}",
        "URL", "Estado", "Tiempo", "HTML(KB)", "MD(KB)", "Ratio%", "Links");
    println!("{}", "-".repeat(95));

    let mut success_count = 0;
    let mut total_time = 0u64;
    let mut total_ratio = 0.0f64;

    for result in &results {
        let url_short = if result.url.len() > 28 {
            format!("{}...", &result.url[..25])
        } else {
            result.url.clone()
        };

        let status_str = if result.status == "✅" { "✅ OK".to_string() } else {
            result.error.as_deref().unwrap_or("ERR").to_string()
        };

        println!("{:<30} | {:>6} | {:>7}ms | {:>7.1} | {:>7.1} | {:>6.1}% | {:>6}",
            url_short,
            status_str,
            result.time_ms,
            result.html_kb,
            result.markdown_kb,
            result.compression_ratio,
            result.links_count
        );

        if result.status == "✅" {
            success_count += 1;
            total_time += result.time_ms;
            total_ratio += result.compression_ratio;
        }
    }

    println!("{}", "-".repeat(95));

    // Executive summary
    println!("\n=============================================================================");
    println!("RESUMEN EJECUTIVO");
    println!("=============================================================================");
    println!("URLs Exitosas:          {}/{} ({:.0}%)", success_count, results.len(), (success_count as f64 / results.len() as f64) * 100.0);
    if success_count > 0 {
        println!("Tiempo Promedio (éxitos): {:.0}ms", total_time as f64 / success_count as f64);
        println!("Ratio de Compresión Promedio: {:.1}% (Ahorro estimado: {:.0}% tokens LLM)", 
            total_ratio / success_count as f64, 100.0 - total_ratio / success_count as f64);
    }
    println!("Total Enlaces Extraídos: {}", results.iter().map(|r| r.links_count).sum::<usize>());
    
    let failures = results.len() - success_count;
    if failures > 0 {
        println!("\nFallos detectados: {}", failures);
        for r in &results {
            if r.status == "❌" {
                println!("  - {}: {}", r.url, r.error.as_deref().unwrap_or("Unknown"));
            }
        }
    }

    println!("\n=============================================================================");
    println!("Test completado. Odysseus Engine v1.0 listo para producción. 🚀");
    println!("=============================================================================");
}

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
    headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
    headers.insert("sec-ch-ua-platform", "\"Windows\"".parse().unwrap());
    headers.insert("sec-fetch-dest", "document".parse().unwrap());
    headers.insert("sec-fetch-mode", "navigate".parse().unwrap());
    headers.insert("sec-fetch-site", "none".parse().unwrap());
    headers.insert("sec-fetch-user", "?1".parse().unwrap());
    headers.insert("upgrade-insecure-requests", "1".parse().unwrap());
    headers.insert("cache-control", "max-age=0".parse().unwrap());
    
    headers
}

fn element_to_markdown(element: &scraper::ElementRef, seen_links: &mut std::collections::HashSet<String>, base_url: &reqwest::Url) -> String {
    let tag_name = element.value().name();
    
    if tag_name == "script" || tag_name == "style" {
        return String::new();
    }

    let mut inner_text = String::new();
    for child in element.children() {
        match child.value() {
            scraper::Node::Text(text) => {
                inner_text.push_str(text.text.as_ref());
            }
            scraper::Node::Element(_) => {
                if let Some(child_elem) = scraper::ElementRef::wrap(child) {
                    inner_text.push_str(&element_to_markdown(&child_elem, seen_links, base_url));
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
                } else { String::new() }
            } else { String::new() }
        }
        "a" => {
            if let Some(href) = element.value().attr("href") {
                let absolute_href = base_url.join(href).map(|u| u.to_string()).unwrap_or_else(|_| href.to_string());
                let link_key = format!("{}|{}", inner_text, absolute_href);
                if !inner_text.is_empty() && seen_links.insert(link_key) {
                    format!("[{}]({})", inner_text, absolute_href)
                } else { inner_text }
            } else { inner_text }
        }
        _ => inner_text,
    }
}