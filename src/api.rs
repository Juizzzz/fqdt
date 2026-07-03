use crate::types::{Book, Chapter};
use crate::util;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime};

#[derive(Clone)]
pub struct Client {
    pub cache_dir: PathBuf,
    pub cache_enabled: bool,
    pub cache_ttl: u64,
    pub search_urls: Vec<String>,
    pub catalog_url: String,
    pub content_urls: Vec<String>,
    pub batch_urls: Vec<String>,
    pub detail_url: String,
    pub audio_content_urls: Vec<String>,
    pub verbose: bool,
    pub reqwest_client: reqwest::Client,
    pub source_stats: Arc<Mutex<HashMap<String, SourceStats>>>,
}

#[derive(Clone, Default)]
pub struct SourceStats {
    pub ok_count: u32,
    pub fail_count: u32,
    pub total_time_ms: u64,
}

impl SourceStats {
    fn score(&self) -> f64 {
        let total = self.ok_count + self.fail_count;
        if total == 0 { return 1.0; }
        let rate = self.ok_count as f64 / total as f64;
        let avg_time = if self.ok_count > 0 { self.total_time_ms as f64 / self.ok_count as f64 } else { 9999.0 };
        rate * 100.0 - avg_time * 0.1
    }
}

fn sort_urls(urls: &[String], stats: &HashMap<String, SourceStats>) -> Vec<String> {
    let mut scored: Vec<(f64, &String)> = urls.iter()
        .map(|u| {
            let s = stats.get(u).cloned().unwrap_or_default();
            (-s.score(), u)
        })
        .collect();
    scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.iter().map(|(_, u)| (*u).clone()).collect()
}

fn maybe_reset_stats(stats: &Arc<Mutex<HashMap<String, SourceStats>>>, counter: &AtomicU64) {
    let c = counter.fetch_add(1, Ordering::Relaxed);
    if c > 0 && c.is_multiple_of(500) {
        if let Ok(mut s) = stats.lock() {
            s.clear();
        }
    }
}

impl Client {
    #[allow(clippy::too_many_arguments)]
    pub fn new(cache_dir: PathBuf, cache_enabled: bool, cache_ttl: u64,
               search_urls: Vec<String>, catalog_url: String, content_urls: Vec<String>,
               batch_urls: Vec<String>, detail_url: String,
               audio_content_urls: Vec<String>, verbose: bool) -> Self {
        Client {
            cache_dir, cache_enabled, cache_ttl, search_urls, catalog_url,
            content_urls, batch_urls, detail_url, audio_content_urls,
            verbose,
            reqwest_client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Linux; Android 14) AppleWebKit/537.36")
                .timeout(std::time::Duration::from_secs(15))
                .build().unwrap(),
            source_stats: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn from_config(cfg: &crate::types::Config, verbose: bool) -> Self {
        Client::new(
            cfg.cache_dir.clone(), cfg.cache_enabled, cfg.cache_ttl,
            cfg.search_urls.clone(), cfg.catalog_url.clone(), cfg.content_urls.clone(),
            cfg.batch_urls.clone(), cfg.detail_url.clone(),
            cfg.audio_content_urls.clone(), verbose,
        )
    }

    pub async fn search(&self, keyword: &str, page: usize) -> Result<Vec<Book>, String> {
        static REQ_COUNTER: AtomicU64 = AtomicU64::new(0);
        maybe_reset_stats(&self.source_stats, &REQ_COUNTER);
        let offset = (page.saturating_sub(1)) * 10;
        let kw = urlencode(keyword);
        let mut last_err = String::new();
        let sorted = {
            let stats = self.source_stats.lock().unwrap();
            sort_urls(&self.search_urls, &stats)
        };
        for tmpl in &sorted {
            let url = tmpl.replacen("{}", &kw, 1).replacen("{}", &offset.to_string(), 1);
            let start = Instant::now();
            let result = util::with_verbose_async(|| async {
                util::with_retry_async(|| self.http_get(&url), 2).await
            }, &format!("search {}", &url[..url.len().min(60)]), self.verbose).await;
            let elapsed = start.elapsed().as_millis() as u64;
            if let Ok(mut stats) = self.source_stats.lock() {
                let entry = stats.entry(url.clone()).or_default();
                match &result {
                    Ok(_) => { entry.ok_count += 1; entry.total_time_ms += elapsed; }
                    Err(_) => { entry.fail_count += 1; }
                }
            }
            match result {
                Ok(text) => {
                    match Self::parse_search_results(&text, self.verbose) {
                        Ok(books) if !books.is_empty() => return Ok(books),
                        Ok(_) => { last_err = "无结果".into(); continue; }
                        Err(e) => { last_err = e; continue; }
                    }
                }
                Err(e) => { last_err = e; continue; }
            }
        }
        Err(last_err)
    }

    fn parse_search_results(text: &str, verbose: bool) -> Result<Vec<Book>, String> {
        let root: Value = serde_json::from_str(text).map_err(|e| {
            if verbose { eprintln!("  [verbose] JSON解析失败: {}\n  原始响应:\n{}", e, text); }
            format!("JSON: {}", e)
        })?;
        let mut books = vec![];
        if let Some(Value::Array(items)) = root.pointer("/data/ret_data") {
            for item in items {
                let (id, title, author, score, category, abstract_) = (
                    item["book_id"].as_str().unwrap_or(""),
                    item["title"].as_str().unwrap_or(""),
                    item["author"].as_str().unwrap_or(""),
                    item["score"].as_str().unwrap_or("-"),
                    item["category"].as_str().unwrap_or(""),
                    item["abstract"].as_str().unwrap_or(""),
                );
                let status = item["creation_status"].as_i64().unwrap_or(0);
                if !id.is_empty() {
                    books.push(Book {
                        book_id: id.into(), title: title.into(), author: author.into(),
                        score: score.into(), category: category.into(), abstract_: abstract_.into(),
                        status,
                    });
                }
            }
        }
        if books.is_empty() { Err("无结果".into()) } else { Ok(books) }
    }

    pub async fn fetch_catalog(&self, book_id: &str) -> Result<Vec<Chapter>, String> {
        let url = self.catalog_url.replacen("{}", book_id, 1);
        let text = util::with_verbose_async(|| async {
            self.get_cached(&url, book_id).await
        }, &format!("catalog {}", book_id), self.verbose).await?;
        let root: Value = serde_json::from_str(&text).map_err(|e| {
            if self.verbose { eprintln!("  [verbose] JSON解析失败: {}\n  原始响应:\n{}", e, text); }
            format!("JSON: {}", e)
        })?;
        let mut chapters = vec![];
        let mut idx = 1;
        if let Some(Value::Array(vlist)) = root.pointer("/data/chapterListWithVolume") {
            for vol in vlist {
                if let Value::Array(items) = vol {
                    for item in items {
                        let item_id = item["itemId"].as_str().unwrap_or("").to_string();
                        let title = item["title"].as_str().unwrap_or("未知章节").to_string();
                        if !item_id.is_empty() {
                            chapters.push(Chapter { index: idx, item_id, title });
                            idx += 1;
                        }
                    }
                }
            }
        }
        if chapters.is_empty() { return Err("未获取到章节".into()); }
        Ok(chapters)
    }

    pub async fn fetch_content(&self, item_id: &str) -> Result<String, String> {
        static REQ_COUNTER: AtomicU64 = AtomicU64::new(0);
        maybe_reset_stats(&self.source_stats, &REQ_COUNTER);
        let sorted = {
            let stats = self.source_stats.lock().unwrap();
            sort_urls(&self.content_urls, &stats)
        };
        let mut last_err = String::new();
        for tmpl in &sorted {
            let url = tmpl.replacen("{}", item_id, 1);
            let start = Instant::now();
            let result = self.fetch_content_from(&url, item_id).await;
            let elapsed = start.elapsed().as_millis() as u64;
            if let Ok(mut stats) = self.source_stats.lock() {
                let entry = stats.entry(url.clone()).or_default();
                match &result {
                    Ok(_) => { entry.ok_count += 1; entry.total_time_ms += elapsed; }
                    Err(_) => { entry.fail_count += 1; }
                }
            }
            if result.is_ok() { return result; }
            last_err = result.unwrap_err();
        }
        Err(last_err)
    }

    async fn fetch_content_from(&self, url: &str, item_id: &str) -> Result<String, String> {
        let raw = util::with_verbose_async(|| async {
            util::with_retry_async(|| self.http_get(url), 3).await
        }, &format!("content {}", &url[..url.len().min(60)]), self.verbose).await?;
        let root: Value = serde_json::from_str(&raw).map_err(|e| {
            if self.verbose { eprintln!("  [verbose] JSON解析失败: {}\n  原始响应:\n{}", e, raw); }
            format!("JSON: {}", item_id)
        })?;
        let content = root.pointer("/data/content").and_then(|v| v.as_str()).unwrap_or("");
        if self.cache_enabled {
            let cp = self.cache_dir.join(format!("c_{}.json", item_id));
            fs::write(&cp, &raw).ok();
        }
        Ok(strip_html(content))
    }

    async fn get_cached(&self, url: &str, book_id: &str) -> Result<String, String> {
        if !self.cache_enabled { return self.http_get(url).await; }
        let cp = self.cache_dir.join(format!("cat_{}.json", book_id));
        if cp.exists() {
            if let Ok(meta) = fs::metadata(&cp) {
                if let Ok(mtime) = meta.modified() {
                    let age = SystemTime::now().duration_since(mtime).unwrap_or_default().as_secs();
                    if age < self.cache_ttl {
                        if let Ok(c) = fs::read_to_string(&cp) { return Ok(c); }
                    }
                }
            }
        }
        let text = self.http_get(url).await?;
        if serde_json::from_str::<Value>(&text).is_ok() {
            fs::write(&cp, &text).ok();
        }
        Ok(text)
    }

    pub async fn http_get(&self, url: &str) -> Result<String, String> {
        if self.verbose { eprintln!("  [verbose] GET {}", url); }
        let resp = self.reqwest_client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;
        let status = resp.status().as_u16();
        let text = resp.text().await.map_err(|e| format!("读取响应: {}", e))?;
        if status == 200 { return Ok(text); }
        Err(format!("HTTP {}: {}..", status, &text[..text.len().min(200)]))
    }

    pub async fn fetch_audio_url(&self, item_id: &str, tone_id: usize) -> Result<String, String> {
        static REQ_COUNTER: AtomicU64 = AtomicU64::new(0);
        maybe_reset_stats(&self.source_stats, &REQ_COUNTER);
        let sorted = {
            let stats = self.source_stats.lock().unwrap();
            sort_urls(&self.audio_content_urls, &stats)
        };
        let mut last_err = String::new();
        for tmpl in &sorted {
            let url = tmpl.replacen("{}", item_id, 1).replacen("{}", &tone_id.to_string(), 1);
            let start = Instant::now();
            let result = self.fetch_audio_from(&url, item_id).await;
            let elapsed = start.elapsed().as_millis() as u64;
            if let Ok(mut stats) = self.source_stats.lock() {
                let entry = stats.entry(url.clone()).or_default();
                match &result {
                    Ok(_) => { entry.ok_count += 1; entry.total_time_ms += elapsed; }
                    Err(_) => { entry.fail_count += 1; }
                }
            }
            if result.is_ok() { return result; }
            last_err = result.unwrap_err();
        }
        Err(last_err)
    }

    async fn fetch_audio_from(&self, url: &str, item_id: &str) -> Result<String, String> {
        let raw = util::with_verbose_async(|| async {
            util::with_retry_async(|| self.http_get(url), 2).await
        }, &format!("audio_url {}", &item_id[..item_id.len().min(16)]), self.verbose).await?;
        let root: Value = serde_json::from_str(&raw).map_err(|e| {
            if self.verbose { eprintln!("  [verbose] JSON解析失败: {}\n  原始响应:\n{}", e, raw); }
            format!("JSON: {}", item_id)
        })?;
        let audio_url = root.pointer("/data/content").and_then(|v| v.as_str()).unwrap_or("");
        if !audio_url.is_empty() {
            if self.verbose { eprintln!("  [verbose] audio URL: {}b", audio_url.len()); }
            return Ok(audio_url.to_string());
        }
        let msg = root.pointer("/message").and_then(|v| v.as_str()).unwrap_or("");
        Err(if msg.is_empty() { "无可听音频".into() } else { msg.to_string() })
    }

    pub async fn fetch_detail(&self, book_id: &str) -> Result<String, String> {
        let url = self.detail_url.replacen("{}", book_id, 1);
        let raw = util::with_verbose_async(|| async {
            util::with_retry_async(|| self.http_get(&url), 2).await
        }, &format!("detail {}", book_id), self.verbose).await?;
        let root: Value = serde_json::from_str(&raw).map_err(|e| {
            if self.verbose { eprintln!("  [verbose] JSON: {}\n  {}", e, raw); }
            format!("JSON: {}", e)
        })?;
        let title = root.pointer("/data/data/book_name")
            .or_else(|| root.pointer("/data/data/bookName"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if title.is_empty() { return Err("未找到书籍信息".into()); }
        let author = root.pointer("/data/data/author").and_then(|v| v.as_str()).unwrap_or("");
        let intro = root.pointer("/data/data/abstract").and_then(|v| v.as_str()).unwrap_or("");
        Ok(format!("{}|{}|{}", title, author, intro).trim_end_matches('|').to_string())
    }

    pub async fn fetch_content_batch(&self, book_id: &str, item_ids: &[&str]) -> Result<HashMap<String, String>, String> {
        static REQ_COUNTER: AtomicU64 = AtomicU64::new(0);
        maybe_reset_stats(&self.source_stats, &REQ_COUNTER);
        let batch = item_ids.join(",");
        let sorted = {
            let stats = self.source_stats.lock().unwrap();
            sort_urls(&self.batch_urls, &stats)
        };
        let mut last_err = String::new();
        for tmpl in &sorted {
            let url = tmpl.replacen("{}", book_id, 1).replacen("{}", &batch, 1);
            let start = Instant::now();
            let result = async {
                let raw = util::with_verbose_async(|| async {
                    util::with_retry_async(|| self.http_get(&url), 2).await
                }, &format!("batch {} chapters", item_ids.len()), self.verbose).await?;
                let root: Value = serde_json::from_str(&raw).map_err(|e| {
                    if self.verbose { eprintln!("  [verbose] JSON: {}\n  {}", e, raw); }
                    format!("JSON: {}", e)
                })?;
                let mut map = HashMap::new();
                if let Some(arr) = root.pointer("/data").and_then(|v| v.as_array()) {
                    for item in arr {
                        let id = item["item_id"].as_str()
                            .or_else(|| item["itemId"].as_str())
                            .unwrap_or("");
                        let content = item["content"].as_str().unwrap_or("");
                        if !id.is_empty() && !content.is_empty() {
                            map.insert(id.to_string(), strip_html(content));
                        }
                    }
                }
                if !map.is_empty() { return Ok(map); }
                Err("批量API未返回数据".into())
            }.await;
            let elapsed = start.elapsed().as_millis() as u64;
            if let Ok(mut stats) = self.source_stats.lock() {
                let entry = stats.entry(url).or_default();
                match &result {
                    Ok(_) => { entry.ok_count += 1; entry.total_time_ms += elapsed; }
                    Err(_) => { entry.fail_count += 1; }
                }
            }
            if result.is_ok() { return result; }
            last_err = result.unwrap_err();
        }
        Err(last_err)
    }
}

fn urlencode(s: &str) -> String {
    let mut r = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => r.push(b as char),
            b' ' => r.push_str("%20"),
            _ => r.push_str(&format!("%{:02X}", b)),
        }
    }
    r
}

pub fn strip_html(s: &str) -> String {
    let mut out = String::new();
    let mut tag = false;
    let mut tag_name = String::new();
    let mut ent = false;
    let mut buf = String::new();
    let mut is_close = false;
    for c in s.chars() {
        if tag {
            if c == '>' {
                tag = false;
                if is_close && (tag_name == "p" || tag_name == "div" || tag_name == "h1" || tag_name == "h2" || tag_name == "h3") {
                    out.push_str("\n\n");
                }
                if tag_name == "br" {
                    out.push('\n');
                }
                tag_name.clear();
                is_close = false;
                continue;
            }
            if c == '/' && tag_name.is_empty() { is_close = true; continue; }
            if !c.is_ascii_whitespace() { tag_name.push(c.to_ascii_lowercase()); }
            continue;
        }
        if ent {
            if c == ';' {
                match buf.as_str() {
                    "lt" => out.push('<'), "gt" => out.push('>'),
                    "amp" => out.push('&'), "nbsp" => out.push(' '),
                    "quot" => out.push('"'), _ => {}
                }
                ent = false; buf.clear();
            } else { buf.push(c); }
            continue;
        }
        if c == '<' { tag = true; tag_name.clear(); is_close = false; continue; }
        if c == '&' { ent = true; buf.clear(); continue; }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html_paragraphs() {
        let s = "<article><p idx=\"0\">第一段</p><p idx=\"1\">第二段</p></article>";
        let out = strip_html(s);
        assert_eq!(out, "第一段\n\n第二段\n\n");
    }

    #[test]
    fn test_strip_html_br() {
        let s = "第一行<br>第二行";
        let out = strip_html(s);
        assert_eq!(out, "第一行\n第二行");
    }

    #[test]
    fn test_strip_html_entities() {
        let s = "&lt;tag&gt; &amp; &quot;hello&quot;";
        let out = strip_html(s);
        assert_eq!(out, "<tag> & \"hello\"");
    }

    #[test]
    fn test_strip_html_heading() {
        let s = "<h2>标题</h2><p>内容</p>";
        let out = strip_html(s);
        assert_eq!(out, "标题\n\n内容\n\n");
    }
}
