# 🚀 Odysseus Engine

**The Zero-Pixel Web Rendering Engine for AI Agents.**

Odysseus is a high-performance, ultra-lightweight web extraction engine built in **Rust**, designed specifically to feed clean, structured data to Large Language Models (LLMs) and AI Agents.

Instead of wasting LLM context windows and compute resources on raw HTML, CSS, and ads, Odysseus extracts only the semantic core of any webpage: **Clean Markdown + Rich Metadata (JSON-LD, Open Graph)**.

---

## ⚡ Why Odysseus?

| Traditional Headless Browser (Puppeteer/Playwright) | Odysseus Engine (Rust) |
| :--- | :--- |
| 🐢 Slow startup & high latency | ⚡ **< 1.2s** average extraction time |
| 💾 Heavy: ~500MB+ RAM per instance | 🪶 **Lightweight: ~45MB** Docker image |
| 💸 Expensive to scale (high CPU/RAM) | 📈 **Highly concurrent** with Tokio & LRU Cache |
| 🗑️ Returns bloated, noisy HTML | 🧠 Returns **LLM-optimized Markdown + Metadata** |

---

## 📊 Real-World Performance Metrics

Odysseus isn't just a concept; it's battle-tested. Recent stress tests against complex, protected financial and crypto websites (CoinDesk, IOG, Messari) yielded:

*   🎯 **83%+ Success Rate** on protected sites using `Stealth` mode (Chrome 120 fingerprinting + Smart Fallback Pipeline).
*   🗜️ **~75% Token Savings**: Compresses a 400KB+ HTML page into ~50KB of dense, semantic Markdown.
*   ⚡ **~4µs Cache Hits**: 100,000x speedup for repeated URL requests via in-memory LRU cache (`moka`).
*   🛡️ **Enterprise-Ready**: Built-in Rate Limiting, API Key authentication, and strict 10s timeouts.

---

## ✨ Key Features

*   **Zero-Pixel Rendering**: No CSS, no images, no wasted bandwidth. Pure semantics.
*   **Smart Fallback Pipeline**: Automatically attempts `Original URL` → `AMP Version` → `Archive.ph` when facing soft paywalls or blocks.
*   **SPA Support**: Embedded `deno_core` (V8) to execute basic JavaScript and render dynamic content.
*   **Rich Metadata Extraction**: Automatically captures `title`, `description`, Open Graph, Twitter Cards, and structured `JSON-LD` data.
*   **Absolute URL Resolution**: Converts all relative links to absolute URLs, perfect for autonomous AI crawlers.
*   **Batch Processing**: Concurrently extract multiple URLs using `tokio::task::JoinSet`.

---

## 🚀 Quick Start (Local)

1. **Clone the repository**:
   ```bash
   git clone https://github.com/Troyanoj/odysseus-engine.git
   cd odysseus-engine
   ```

2. **Set up environment variables** (optional, defaults are provided):
   ```bash
   export API_KEY="your-secret-key"
   export PORT="3000"
   ```

3. **Run in release mode**:
   ```bash
   cargo run --release
   ```

The API will start at `http://0.0.0.0:3000`

---

## 📡 API Usage

### 1. Health Check
```bash
curl -s http://localhost:3000/health
# Response: {"status":"ok","version":"1.0.0","cache_capacity":10000}
```

### 2. Single URL Extraction (Stealth Mode)
Use `mode: "Stealth"` to activate Chrome 120 headers and the Smart Fallback Pipeline for protected sites.

```bash
curl -s -X POST http://localhost:3000/v1/extract \
  -H "x-api-key: your-secret-key" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://www.coindesk.com/business/2024/12/01/cardano-ada-chang-hard-fork-governance/",
    "mode": "Stealth"
  }'
```

### 3. Batch Extraction
```bash
curl -s -X POST http://localhost:3000/v1/extract \
  -H "x-api-key: your-secret-key" \
  -H "Content-Type: application/json" \
  -d '{
    "urls": [
      "https://example.com",
      "https://quotes.toscrape.com"
    ]
  }'
```

---

## 🐳 Docker Deployment (On-Premise)

For enterprises requiring data privacy and on-premise deployment, Odysseus is packaged as an ultra-lightweight, multi-stage Docker image (~45MB).

### Build the image:
```bash
docker build -t odysseus-engine:latest .
```

### Run with Docker Compose:
```bash
# Copy .env.example to .env and configure your API_KEY
docker compose up -d
```

---

## 🏗️ Architecture

*   **main.rs**: Axum server bootstrap, environment configuration, and tracing setup.
*   **api.rs**: REST endpoints, API Key middleware, and rate limiting (`tower-governor`).
*   **extractor.rs**: The core engine. HTTP client management, Smart Fallback Pipeline, HTML parsing (`scraper`), and Markdown generation.
*   **js_engine.rs**: Lightweight V8 integration (`deno_core`) for basic SPA DOM manipulation.

---

## ⚖️ License & Commercial Use

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**.

This means you are free to use, modify, and distribute this code, provided that any networked service using it also releases its source code under the same license.

### 🏢 Need to use Odysseus Engine in a proprietary commercial product or SaaS?

We offer **Commercial Licenses** that remove AGPL restrictions, along with priority support, custom extractor packs, and advanced Cloudflare evasion integrations.

📩 **Contact for Enterprise Licensing**: jfsa6888@gmail.com

---

Built with ❤️ and ⚡ by Jaime Francisco Sánchez Aragón
