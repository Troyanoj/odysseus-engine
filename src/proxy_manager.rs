use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ProxyConfig {
    proxies: Vec<String>,
    current_index: Arc<AtomicUsize>,
}

impl ProxyConfig {
    pub fn new(proxy_list: Vec<String>) -> Self {
        Self {
            proxies: proxy_list,
            current_index: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn get_next_proxy(&self) -> Option<String> {
        if self.proxies.is_empty() {
            return None;
        }
        let index = self.current_index.fetch_add(1, Ordering::SeqCst);
        Some(self.proxies[index % self.proxies.len()].clone())
    }

    pub fn has_proxies(&self) -> bool {
        !self.proxies.is_empty()
    }
}