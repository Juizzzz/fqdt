use crate::api::Client;
use crate::config;
use crate::types::{Config, DownloadParams};
use std::io::Write;
use std::time::Instant;

#[allow(clippy::too_many_arguments)]
pub fn run(keyword: &str, page: usize, output: Option<&str>, concurrent: Option<usize>,
           range: Option<&str>, format: Option<&str>, verbose: bool, auto: Option<usize>,
           no_download: bool, _interval: u64, cfg: &Config) {
    let start = Instant::now();
    let vb = verbose || cfg.verbose;
    let api = Client::from_config(cfg, vb);

    print!("  搜索 \"{}\" (第{}页)... ", keyword, page);
    std::io::stdout().flush().unwrap();
    let books = match api.search(keyword, page) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("\n  err {}", e);
            return;
        }
    };
    if books.is_empty() {
        println!("无结果");
        return;
    }
    let elapsed = start.elapsed().as_secs();
    println!("{} 本 ({}s)", books.len(), elapsed);
    println!();

    let mut list_lines = 2;
    for (i, b) in books.iter().enumerate() {
        let status = b.status_text();
        let abs: String = b.abstract_.chars().take(60).collect();
        let abs = if b.abstract_.chars().count() > 60 {
            format!("{}...", abs)
        } else {
            b.abstract_.clone()
        };
        println!("  \x1b[1;36m{:>2}.\x1b[0m {}", i + 1, b.title);
        println!(
            "      {} \x1b[33m{}\x1b[0m | {} | \x1b[35m{}\x1b[0m \x1b[2m#{}\x1b[0m",
            b.author, b.category, status, b.score, b.book_id
        );
        list_lines += 2;
        if !abs.is_empty() {
            println!("      {}", abs);
            list_lines += 1;
        }
        println!();
        list_lines += 1;
    }

    if no_download {
        println!("\n  \x1b[2m使用 info <book_id> 查看目录, download <book_id> 下载\x1b[0m");
        println!("  \x1b[2m翻页: fqdt search \"{}\" -p {}\x1b[0m", keyword, page + 1);
        return;
    }

    let idx = auto.map_or_else(
        || {
            print!("  \x1b[2m输入序号 (1-{}, 0=取消): \x1b[0m", books.len());
            std::io::stdout().flush().unwrap();
            list_lines += 1;
            let mut inp = String::new();
            std::io::stdin().read_line(&mut inp).unwrap();
            match inp.trim().parse::<usize>() {
                Ok(n) if n >= 1 && n <= books.len() => n - 1,
                _ => {
                    println!("  \x1b[2m取消\x1b[0m");
                    books.len()
                }
            }
        },
        |n| {
            if n >= 1 && n <= books.len() {
                n - 1
            } else {
                println!("  err 无效序号");
                books.len()
            }
        },
    );

    if idx >= books.len() {
        return;
    }
    let book = &books[idx];

    if auto.is_none() {
        print!("\x1b[{}F\x1b[J", list_lines);
    }
    println!(
        "  \x1b[1;36m{}\x1b[0m  {} \x1b[33m{}\x1b[0m | {} | \x1b[35m{}\x1b[0m",
        book.title,
        book.author,
        book.category,
        book.status_text(),
        book.score
    );
    config::add_bookmark(&book.book_id, &book.title).ok();
    super::download::run(
        &book.book_id, &DownloadParams {
            output: output.map(|s| s.to_string()),
            range: range.map(|s| s.to_string()),
            format: format.map(|s| s.to_string()),
            concurrent,
            audio: false, tone: 1, abr: 0,
            lrc: "external".into(), force: false,
            verbose: vb, book_title: Some(book.title.clone()),
        }, cfg,
    );
}
