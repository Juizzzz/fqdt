use crate::audio;
use crate::types::{Chapter, Config};

pub fn run(input: &str, lrc: bool, cover: Option<String>, cfg: &Config) {
    let vb = cfg.verbose;
    let p = std::path::Path::new(input);
    if !p.exists() {
        eprintln!("  err {} 不存在", input);
        return;
    }

    if lrc {
        embed_lrc(p, vb);
    }

    if let Some(cover_path) = cover {
        let cp = std::path::Path::new(&cover_path);
        if !cp.exists() {
            eprintln!("  err 封面不存在: {}", cover_path);
        } else {
            embed_cover(p, cp, vb);
        }
    }
}

fn embed_lrc(p: &std::path::Path, vb: bool) {
    if p.is_dir() {
        for e in std::fs::read_dir(p).unwrap().flatten() {
            let path = e.path();
            if path.extension().map(|x| x == "mp3").unwrap_or(false) {
                let ch = Chapter {
                    index: 0,
                    title: path.file_stem().unwrap().to_string_lossy().to_string(),
                    item_id: String::new(),
                };
                audio::embed_lrc(&path, &ch, vb, None);
                println!("  ok {}", path.display());
            }
        }
    } else if p.extension().map(|x| x == "mp3").unwrap_or(false) {
        let ch = Chapter {
            index: 0,
            title: p.file_stem().unwrap().to_string_lossy().to_string(),
            item_id: String::new(),
        };
        audio::embed_lrc(p, &ch, vb, None);
        println!("  ok {}", p.display());
    } else {
        eprintln!("  err 不是 MP3 文件");
    }
}

fn embed_cover(p: &std::path::Path, cp: &std::path::Path, vb: bool) {
    if p.is_dir() {
        for e in std::fs::read_dir(p).unwrap().flatten() {
            let path = e.path();
            if path.extension().map(|x| x == "mp3").unwrap_or(false) {
                audio::embed_cover(&path, cp, vb);
                println!("  ok {} ← {}", path.display(), cp.display());
            }
        }
    } else if p.extension().map(|x| x == "mp3").unwrap_or(false) {
        audio::embed_cover(p, cp, vb);
        println!("  ok {} ← {}", p.display(), cp.display());
    } else {
        eprintln!("  err 不是 MP3 文件");
    }
}
