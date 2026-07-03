use crate::types::{sanitize_filename, Chapter};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub fn has_chapter_prefix(title: &str) -> bool {
    title.starts_with('第') || title.starts_with(|c: char| c.is_ascii_digit())
}

pub fn chapter_heading(ch: &Chapter) -> String {
    if has_chapter_prefix(&ch.title) {
        ch.title.clone()
    } else {
        format!("第{}章 {}", ch.index, ch.title)
    }
}

pub fn format_filename(template: &str, ch: &Chapter) -> String {
    template
        .replace("{idx04}", &format!("{:04}", ch.index))
        .replace("{idx}", &ch.index.to_string())
        .replace("{title}", &sanitize_filename(&ch.title))
}

pub fn filter_by_range<'a>(
    chapters: &'a [Chapter],
    range: Option<&crate::types::ChapterRange>,
) -> Vec<&'a Chapter> {
    chapters
        .iter()
        .filter(|c| range.is_none_or(|r| r.contains(c.index)))
        .collect()
}

pub fn bar_style(colors: &str, concurrent: usize) -> ProgressStyle {
    ProgressStyle::default_bar()
        .template(&format!(
            "[{{elapsed_precise}}] {{bar:28.{colors}}} {{pos}}/{{len}} ({}j) {{msg}}", concurrent
        ))
        .unwrap()
        .progress_chars("━▶")
}

pub fn json_esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub async fn with_retry_async<F, Fut, T>(f: F, max: u32) -> Result<T, String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    let mut last_err = String::new();
    for i in 0..max {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                last_err = e;
                if i + 1 < max {
                    tokio::time::sleep(Duration::from_secs(1 << i)).await;
                }
            }
        }
    }
    Err(format!("重试{}次后失败: {}", max, last_err))
}

pub async fn with_verbose_async<F, Fut, T>(f: F, label: &str, enabled: bool) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    if enabled {
        eprintln!("  [verbose] {} 开始", label);
    }
    let result = f().await;
    if enabled {
        eprintln!("  [verbose] {} 完成", label);
    }
    result
}

pub async fn with_progress_async<T, U, F, Fut>(
    items: Vec<T>,
    total: usize,
    concurrent: usize,
    colors: &str,
    work: F,
) -> usize
where
    T: Send + 'static,
    U: Send + 'static,
    F: Fn(T, ProgressBar) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<U, String>> + Send,
{
    if items.is_empty() {
        return 0;
    }

    let pb = ProgressBar::new(total as u64);
    pb.set_style(bar_style(colors, concurrent));

    let skipped = total - items.len();
    pb.inc(skipped as u64);

    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrent.max(1)));
    let failed = Arc::new(AtomicUsize::new(0));
    let work = Arc::new(work);
    let mut handles = Vec::with_capacity(items.len());

    for item in items {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let pb = pb.clone();
        let fl = failed.clone();
        let work = work.clone();

        handles.push(tokio::spawn(async move {
            let _permit = permit;
            let result = work(item, pb.clone()).await;
            if result.is_err() {
                fl.fetch_add(1, Ordering::SeqCst);
            }
            pb.inc(1);
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    pb.finish_and_clear();
    failed.load(Ordering::SeqCst)
}


