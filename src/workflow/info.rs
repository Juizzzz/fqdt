use crate::api::Client;
use crate::types::{ChapterRange, Config};
use crate::util;

pub fn run(book_id: &str, range: Option<&str>, show: bool, verbose: bool, cfg: &Config) {
    let vb = verbose || cfg.verbose;
    let api = Client::from_config(cfg, vb);
    print!("  获取目录... ");
    flush();
    let chs = match api.fetch_catalog(book_id) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("\n  err {}", e);
            return;
        }
    };
    let r = range.and_then(ChapterRange::parse);
    let v = util::filter_by_range(&chs, r.as_ref());
    println!("{} 章", chs.len());

    if show {
        for c in &v {
            println!("\n  \x1b[1;36m{:04} {}\x1b[0m", c.index, c.title);
            match api.fetch_content(&c.item_id) {
                Ok(text) => {
                    for line in text.lines().take(40) {
                        println!("  {}", line);
                    }
                    if text.lines().count() > 40 {
                        println!("  \x1b[2m... (共{}行)\x1b[0m", text.lines().count());
                    }
                }
                Err(e) => println!("  err {}", e),
            }
        }
    } else {
        for c in &v {
            println!("  {:04}  {}", c.index, c.title);
        }
        println!("\n  共 {} 章", v.len());
    }
}

fn flush() {
    use std::io::Write;
    std::io::stdout().flush().unwrap();
}
