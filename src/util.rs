use crate::types::{sanitize_filename, Chapter};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
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

pub fn bar_style(colors: &str) -> ProgressStyle {
    ProgressStyle::default_bar()
        .template(&format!(
            "[{{elapsed_precise}}] {{bar:28.{colors}}} {{pos}}/{{len}} {{msg}}"
        ))
        .unwrap()
        .progress_chars("━▶")
}

pub fn json_esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn with_retry<T>(
    f: impl Fn() -> Result<T, String>,
    max: u32,
) -> Result<T, String> {
    let mut last_err = String::new();
    for i in 0..max {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => {
                last_err = e;
                if i + 1 < max {
                    thread::sleep(Duration::from_secs(1 << i));
                }
            }
        }
    }
    Err(format!("重试{}次后失败: {}", max, last_err))
}

pub fn with_verbose<T>(f: impl FnOnce() -> T, label: &str, enabled: bool) -> T {
    if enabled {
        eprintln!("  [verbose] {} 开始", label);
    }
    let result = f();
    if enabled {
        eprintln!("  [verbose] {} 完成", label);
    }
    result
}

pub fn with_progress<T, U>(
    items: Vec<T>,
    total: usize,
    concurrent: usize,
    colors: &str,
    work: impl Fn(&T, &ProgressBar) -> Result<U, String> + Send + Sync + 'static,
) -> usize
where
    T: Send + Sync + 'static,
    U: Send + 'static,
{
    if items.is_empty() {
        return 0;
    }

    let pb = ProgressBar::new(total as u64);
    pb.set_style(bar_style(colors));

    let skipped = total - items.len();
    pb.inc(skipped as u64);

    let failed = Arc::new(AtomicUsize::new(0));
    let items = Arc::new(items);
    let work = Arc::new(work);
    let n = items.len();
    let count = concurrent.max(1);
    let mut handles = vec![];

    for w in 0..count {
        let items = items.clone();
        let pb = pb.clone();
        let fl = failed.clone();
        let work = work.clone();

        handles.push(thread::spawn(move || {
            for i in (w..n).step_by(count) {
                let item = &items[i];
                match work(item, &pb) {
                    Ok(_) => {}
                    Err(_) => {
                        fl.fetch_add(1, Ordering::SeqCst);
                    }
                }
                pb.inc(1);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    pb.finish_and_clear();
    failed.load(Ordering::SeqCst)
}


