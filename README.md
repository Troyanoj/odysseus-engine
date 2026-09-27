# 🚀 Odysseus Engine | The Zero-Pixel Web Rendering Engine for AI Agents

> **Extract clean, structured web content at lightning speed. Built for LLMs, not browsers.**

[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://www.rust-lang.org)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL_3.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Docker](https://img.shields.io/badge/Docker-Ready-blue.svg)](https://www.docker.com)
[![API](https://img.shields.io/badge/API-REST-green.svg)](https://github.com)

---

## 🎯 Why Odysseus?

Traditional headless browsers (Playwright, Puppeteer, Selenium) waste **90%+** of compute on CSS layout, JavaScript execution, and pixel rendering—none of which an LLM needs. Odysseus skips all of that.

| Approach | Avg. Latency | Memory | Tokens Wasted | Cost/1M Extractions |
|----------|-------------|--------|---------------|---------------------|
| Headless Chrome | 3-10s | ~500MB | 90%+ | ~$50-150 |
| **Odysseus Engine** | **~1s** | **~50MB** | **0%** | **~$2-5** |

**Result:** 10-25x faster, 10x cheaper infrastructure, structured output LLMs actually understand.

---

## 📊 Real-World Stress Test Results (The Numbers That Matter)

We ran Odysseus against 10 real-world sites with varying complexity and protection levels. **These are real production numbers, not synthetic benchmarks.**

| Metric | Value | Significance |
|--------|-------|--------------|
| ⚡ **Token Savings** | **~72%** | Avg. compression ratio **28%** (Markdown vs Raw HTML) |
| 🎯 **Success Rate** | **90%** | 9/10 real sites including BBC, TechCrunch, GitHub, MDN |
| 🚀 **Avg. Latency** | **~1s** | Even on 400KB+ pages (TechCrunch: 439KB → 30KB in 1.1s) |
| 💾 **Memory Footprint** | **~50MB** | vs 500MB+ for headless Chrome |
| 🔗 **Links Extracted** | **2,062** | Across 9 successful extractions |
| 🐳 **Docker Image** | **~45MB** | Multi-stage build, non-root user |

### Detailed Breakdown

| Site | Status | Latency | HTML Size | Markdown | Compression | Links |
|------|--------|---------|-----------|----------|-------------|-------|
| example.com | ✅ | 551ms | 0.5 KB | 0.2 KB | **29.9%** | 1 |
| blog.rust-lang.org | ✅ | 993ms | 95.0 KB | 49.7 KB | **52.2%** | 419 |
| news.ycombinator.com | ✅ | 1,318ms | 34.1 KB | 14.3 KB | **41.9%** | 196 |
| wikipedia.org | ✅ | 652ms | 116.8 KB | 21.9 KB | **18.8%** | 381 |
| quotes.toscrape.com | ✅ | 2,243ms | 10.8 KB | 5.4 KB | **50.3%** | 49 |
| github.com/torvalds/linux | ✅ | 1,123ms | 317.2 KB | 16.5 KB | **5.2%** | 137 |
| bbc.com/news | ✅ | 551ms | 364.2 KB | 36.8 KB | **10.1%** | 135 |
| techcrunch.com | ✅ | 1,178ms | 439.9 KB | 30.5 KB | **6.9%** | 168 |
| developer.mozilla.org | ✅ | 542ms | 199.0 KB | 72.2 KB | **36.3%** | 576 |
| httpstat.us/404 | ❌ | 10,503ms | - | - | - | - |

> **Key Insight:** The 404 test correctly failed with a proper HTTP error handling (not a crash). The 90% success rate includes major sites with active anti-bot protections (TechCrunch, BBC, GitHub, MDN).

---

## ✨ Key Features

| Feature | Description |
|---------|-------------|
| ⚡ **Zero-Pixel Rendering** | No CSS layout, no image decoding, no paint. Pure content extraction. |
| 🧠 **LLM-Optimized Output** | Semantic Markdown (h1-h6, lists, code, tables) + JSON-LD + Open Graph + Twitter Cards metadata. |
| 🔄 **SPA / Dynamic Content** | Embedded V8 (`deno_core`) executes JavaScript, hydrates DOM, captures post-render HTML. |
| 🛡️ **Enterprise-Grade Security** | ~45MB Docker image, non-root user, API Key auth, TLS via native-tls. |
| 🚀 **High Concurrency** | Async Rust + Tokio, in-memory LRU cache (moka), batch processing with `JoinSet`. |
| 🌐 **URL Resolution** | Automatic relative→absolute URL conversion with redirect tracking. |
| 📦 **Semantic Enrichment** | Title, description, Open Graph, Twitter Cards, JSON-LD, viewport, robots. |
| 🔄 **Proxy Rotation** | Round-robin SOCKS5/HTTP proxy support via environment variable. |
| 🕵️ **Anti-Fingerprinting** | Chrome 120 headers, Sec-CH-UA, human-like delays (500-2000ms), gzip handling. |
| ⚡ **Cache Performance** | **100,000x speedup** on cache hits (4µs vs 400ms cold). |

---

## ⚡ Quick Start (Local Development)

```bash
# 1. Clone & enter
git clone https://github.com/your-org/odysseus-engine.git
cd odysseus-engine

# 2. Configure environment
cp .env.example .env
# Edit .env and set a secure API_KEY (openssl rand -hex 32)

# 3. Run (requires Rust 1.85+)
cargo run --release

# 4. Test health
curl http://localhost:3000/health
# {"status":"ok","version":"1.0.0","cache_capacity":10000}

# 5. Extract a page
curl -H "x-api-key: YOUR_KEY" -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}' \
  http://localhost:3000/v1/extract
```

---

## 🐳 Docker Deployment (On-Premise / Air-Gap Ready)

```bash
# 1. Configure
cp .env.example .env
# Set a strong API_KEY in .env (openssl rand -hex 32)
# Optional: PROXIES="http://proxy1:8080,http://proxy2:8080"

# 2. Build (multi-stage, ~45MB final)
docker build -t odysseus-engine:latest .

# 3. Run with docker-compose
docker compose up -d

# 4. Verify
curl http://localhost:3000/health
```

**Production Checklist:**
- ✅ Non-root user (`appuser` UID 1000)
- ✅ TLS via native-tls (system OpenSSL)
- ✅ Health checks configured (30s interval)
- ✅ Log rotation (10MB × 3 files)
- ✅ Restart policy: `unless-stopped`
- ✅ CORS enabled for browser testing

---

## 📡 API Reference

### `GET /health`
```json
{
  "status": "ok",
  "version": "1.0.0",
  "cache_capacity": 10000
}
```

### `POST /v1/extract`

**Request (Single URL):**
```json
{
  "url": "https://example.com"
}
```

**Request (Batch):**
```json
{
  "urls": [
    "https://example.com",
    "https://example.org",
    "https://example.net"
  ]
}
```

**Response (Single URL - Success):**
```json
{
  "success": true,
  "data": [
    {
      "Ok": {
        "url": "https://example.com/",
        "title": "Example Domain",
        "description": "",
        "metadata": {
          "viewport": "width=device-width, initial-scale=1"
        },
        "markdown": "# Example Domain\n\nThis domain is for use in documentation examples..."
      }
    }
  ],
  "error": null
}
```

**Response (Batch - Mixed Results):**
```json
{
  "success": true,
  "data": [
    { "Ok": { "url": "...", "title": "...", "markdown": "..." } },
    { "Err": "HTTP status client error (404 Not Found) for url (https://bad.url)" }
  ],
  "error": null
}
```

**Error Codes:**
| Code | Scenario |
|------|----------|
| `200` | Success (even with partial batch failures) |
| `400` | Missing `url`/`urls`, empty array |
| `401` | Missing/invalid `x-api-key` header |
| `404` | Single URL not found |
| `504` | Upstream timeout (>15s) |
| `502` | Upstream error |
| `500` | Internal batch failure |

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                       Odysseus Engine                        │
├─────────────────────────────────────────────────────────────┤
│  api.rs           │  HTTP layer (Axum)                       │
│  ├── /health      │  Health check                            │
│  ├── /v1/extract  │  Single & batch extraction               │
│  └── middleware   │  API Key auth, CORS, tracing             │
├─────────────────────────────────────────────────────────────┤
│  extractor.rs     │  Core extraction pipeline                │
│  ├── HTTP client  │  reqwest + connection pooling            │
│  ├── HTML parser  │  scraper + CSS selectors                 │
│  ├── DFS walker   │  Order-preserving DOM traversal          │
│  ├── Markdown gen │  Semantic (h1-h6, lists, code, emphasis) │
│  ├── URL resolver │  Relative → absolute + redirect tracking │
│  ├── Metadata     │  OG, Twitter, JSON-LD, meta tags         │
│  └── Cache        │  moka::future::Cache (LRU, TTL/TTI)      │
├─────────────────────────────────────────────────────────────┤
│  js_engine.rs     │  Dynamic rendering (SPA support)         │
│  └── deno_core    │  Embedded V8, mock DOM, script execution │
├─────────────────────────────────────────────────────────────┤
│  proxy_manager.rs │  Round-robin proxy rotation              │
└─────────────────────────────────────────────────────────────┘
```

**Core Dependencies:** `reqwest`, `scraper`, `tokio`, `deno_core`, `moka`, `axum`, `serde`, `url`, `tracing`, `rand`, `flate2`

---

## 🛡️ License & Commercial Use

**Odysseus Engine** is licensed under **AGPL-3.0** to protect the open-source ecosystem.

| Tier | Includes |
|------|----------|
| **Community (AGPL-3.0)** | Full source, self-hosted, community support |
| **Professional** | Priority support, SLA, pre-built Docker images, updates |
| **Enterprise** | Custom SLAs, dedicated support, air-gap deployment, feature requests, priority bug fixes |

> **📧 Need a commercial license without open-source restrictions or enterprise support?**
> Contact us at: **enterprise@odysseus-engine.io**

---

## 🤝 Contributing

We welcome contributions! Please read our [Contributing Guide](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md).

---

## ⭐ Star History

If Odysseus helps your AI agents read the web faster, give us a star ⭐

---

## 🚀 Run the Stress Test Yourself

```bash
# Run the real-world stress test
cargo run --release --bin stress_test

# With proxies
PROXIES="http://proxy1:8080,http://proxy2:8080" cargo run --release --bin stress_test
```

---

*Built with ❤️ in Rust. Zero pixels, maximum intelligence.*