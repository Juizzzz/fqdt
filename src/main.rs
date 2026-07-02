mod api;
mod audio;
mod config;
mod download;
mod epub;
mod types;
mod util;
mod workflow;

use clap::{Parser, Subcommand};
use types::Config;

#[derive(Parser)]
#[command(name = "fqdt", version, about = "番茄小说下载器")]
struct Cli {
    #[arg(long, global = true, help = "搜索 API")]
    search_url: Option<String>,
    #[arg(long, global = true, help = "目录 API")]
    catalog_url: Option<String>,
    #[arg(long, global = true, help = "内容 API")]
    content_url: Option<String>,
    #[arg(long, global = true, help = "超时秒")]
    timeout: Option<u64>,
    #[arg(long, global = true, help = "HTTP auto/minreq/curl")]
    http: Option<String>,
    #[arg(long, global = true, help = "curl 参数")]
    curl_args: Option<String>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// 搜索并下载小说
    Search {
        /// 搜索关键词
        keyword: String,
        #[arg(short='p', long, default_value="1", help = "页码")]
        page: usize,
        #[arg(short='D', long, help = "自动下载第 N 本 (无需交互)")]
        auto: Option<usize>,
        #[arg(long, help = "仅搜索不下载")]
        dry_run: bool,
        #[arg(short='o', long, help = "输出目录")]
        output: Option<String>,
        #[arg(short='j', long, help = "并发数")]
        jobs: Option<usize>,
        #[arg(short='r', long, help = "章节范围 (1-50 / -5 / 10-)")]
        range: Option<String>,
        #[arg(short='t', long, help = "输出格式 (txt/epub)")]
        format: Option<String>,
        #[arg(short='i', long, default_value = "0", help = "请求间隔(ms)")]
        interval: u64,
        #[arg(short='v', long, help = "详细输出")]
        verbose: bool,
    },
    /// 查看目录或内容
    Info {
        /// 书籍 ID
        book_id: String,
        #[arg(short='r', long, help = "章节范围")]
        range: Option<String>,
        #[arg(short='s', long, help = "显示正文内容")]
        show: bool,
        #[arg(short='v', long, help = "详细输出")]
        verbose: bool,
    },
    /// 下载章节正文
    Download {
        /// 书籍 ID 或本地目录
        book_id: String,
        #[arg(short='o', long, help = "输出目录")]
        output: Option<String>,
        #[arg(short='j', long, help = "并发数")]
        jobs: Option<usize>,
        #[arg(short='r', long, help = "章节范围 (1-50 / -5 / 10-)")]
        range: Option<String>,
        #[arg(short='t', long, help = "输出格式 (txt/epub)")]
        format: Option<String>,
        #[arg(long, help = "同时下载音频")]
        audio: bool,
        #[arg(long, default_value = "1", help = "音色编号")]
        tone: usize,
        #[arg(long, default_value = "0", help = "压缩码率 (0=跳过)")]
        abr: u32,
        #[arg(long, default_value = "external", help = "歌词模式: external/embed/both/off")]
        lrc: String,
        #[arg(short='f', long, help = "强制覆盖已存在文件")]
        force: bool,
        #[arg(short='i', long, default_value = "0", help = "请求间隔(ms)")]
        interval: u64,
        #[arg(short='v', long, help = "详细输出")]
        verbose: bool,
    },
    /// 增量更新（只下载新章节）
    Update {
        /// 书籍 ID 或本地目录
        book_id: Option<String>,
        #[arg(short='o', long, help = "输出目录")]
        output: Option<String>,
        #[arg(short='j', long, help = "并发数")]
        jobs: Option<usize>,
        #[arg(short='r', long, help = "章节范围")]
        range: Option<String>,
        #[arg(short='f', long, help = "强制覆盖")]
        force: bool,
        #[arg(long, help = "同时更新音频")]
        audio: bool,
        #[arg(short='v', long, help = "详细输出")]
        verbose: bool,
        #[arg(short='i', long, default_value = "0", help = "请求间隔(ms)")]
        interval: u64,
    },
    /// 书架管理
    Shelf {
        #[arg(short='a', long)]
        add: Option<String>,
        #[arg(short='d', long)]
        delete: Option<usize>,
        #[arg(short='D', long)]
        dl: Option<usize>,
    },
    /// 生成默认配置
    Init,
    /// 测试 API 连接
    #[command(name = "test-api")]
    TestApi,
    /// 下载语音或 TTS 转语音
    Audio {
        /// 书籍 ID
        book_id: Option<String>,
        #[arg(short='o', long, help = "输出目录")]
        output: Option<String>,
        #[arg(short='r', long, help = "章节范围 (1-50 / -5 / 10-)")]
        range: Option<String>,
        #[arg(long, default_value = "1", help = "音色编号")]
        tone: usize,
        #[arg(short='t', long, help = "TTS 文本文件或目录路径 (代替 book_id)")]
        tts: Option<String>,
        #[arg(long, default_value = "zh-CN-XiaoxiaoNeural", help = "TTS 语音")]
        voice: String,
        #[arg(long, help = "TTS 语速 (+0% / -50% / +100%)")]
        rate: Option<String>,
        #[arg(long, help = "TTS 音量 (+0% / -50%)")]
        volume: Option<String>,
        #[arg(long, help = "TTS 音调 (+0Hz / -20Hz / +20Hz)")]
        pitch: Option<String>,
        #[arg(long, help = "压缩码率 kbps (0=跳过压缩, 32/64/128)")]
        abr: Option<u32>,
        #[arg(long, help = "变速播放 (0.5=半速, 2.0=双倍)")]
        speed: Option<f32>,
        #[arg(long, help = "归一化音量 (均衡响度)")]
        normalize: bool,
        #[arg(short='e', long, help = "后处理命令模板 ({input} {output})")]
        exec: Option<String>,
        #[arg(long, default_value = "external", help = "歌词模式: external / embed / both / off")]
        lrc: String,
        #[arg(short='f', long, help = "强制重新下载已存在的文件")]
        force: bool,
        #[arg(short='j', long, help = "并发数")]
        jobs: Option<usize>,
        #[arg(short='i', long, default_value = "0", help = "请求间隔(ms)")]
        interval: u64,
        #[arg(short='v', long, help = "详细输出")]
        verbose: bool,
    },
    /// 小工具集 + 函数管道。可用函数: embed, process, fetch-catalog, fetch-content, fetch-audio-url, search, strip-html, compress, embed-lrc, embed-cover, read-info, read-audio-info。管道: fn1 args \; fn2 {} args
    Function {
        #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
        args: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let mut cfg = Config::load();
    cfg.apply_cli_overrides(
        cli.search_url.as_deref(),
        cli.catalog_url.as_deref(),
        cli.content_url.as_deref(),
    );
    if let Some(to) = cli.timeout {
        cfg.timeout = to;
    }
    if let Some(h) = cli.http {
        cfg.http_method = h;
    }
    if let Some(c) = cli.curl_args {
        cfg.curl_args = c;
    }
    cfg.ensure_dirs();
    Config::save_default().ok();

    match cli.cmd {
        Cmd::Search { keyword, page, auto, dry_run, output, jobs, range, format, interval, verbose } =>
            workflow::search::run(
                &keyword, page, output.as_deref(), jobs, range.as_deref(),
                format.as_deref(), verbose, auto, dry_run, interval, &cfg,
            ),
        Cmd::Info { book_id, range, show, verbose } =>
            workflow::info::run(&book_id, range.as_deref(), show, verbose, &cfg),
        Cmd::Download { book_id, output, jobs, range, format, audio, tone, abr, lrc, force, interval, verbose } =>
            workflow::download::run(
                &book_id, output.as_deref(), jobs, range.as_deref(), format.as_deref(),
                audio, tone, abr, &lrc, force, verbose, interval, &cfg, None,
            ),
        Cmd::Update { book_id, output, jobs, range, force, audio, verbose, interval } =>
            workflow::update::run(
                book_id.as_deref(), output.as_deref(), jobs, range.as_deref(),
                force, audio, verbose, interval, &cfg,
            ),
        Cmd::Shelf { add, delete, dl } =>
            workflow::shelf::run(add, delete, dl, &cfg),
        Cmd::Init => {
            Config::save_default().ok();
            println!("  ok ~/.config/fqdt/config.ini");
        }
        Cmd::TestApi => test_api(&cfg),
        Cmd::Audio { book_id, output, range, tone, tts, voice, rate, volume, pitch, abr, speed, normalize, exec, lrc, force, jobs, interval, verbose } =>
            workflow::audio::run(
                book_id.as_deref(), output.as_deref(), range.as_deref(), tone, verbose,
                tts.as_deref(), &voice, rate, volume, pitch, abr, speed, normalize,
                exec, &lrc, force, jobs, interval, &cfg,
            ),
        Cmd::Function { args } => run_function(args, &cfg),
    }
}

fn test_api(cfg: &Config) {
    fn test(api: &api::Client, label: &str, url: &str, desc: &str) {
        print!("  {} {} ... ", label, desc);
        flush();
        match api.http_get(url) {
            Ok(text) if !text.is_empty() => {
                let snippet: String = text.chars().take(120).collect();
                println!("\x1b[32m✓\x1b[0m {}b", text.len());
                println!("    {}", snippet);
            }
            Ok(_) => println!("\x1b[33m⚠\x1b[0m 空响应"),
            Err(e) => println!("\x1b[31m✗ {}\x1b[0m", e),
        }
    }

    let mut api = api::Client::from_config(cfg, cfg.verbose);
    api.cache_enabled = false;
    api.cache_ttl = 0;

    println!("\n  📡 API 测试\n");

    println!("  ── 搜索 ──");
    for tmpl in &cfg.search_urls {
        let url = tmpl.replacen("{}", "凡人", 1).replacen("{}", "0", 1);
        test(&api, "", &url, "search?q=凡人");
    }

    println!("\n  ── 目录 ──");
    let url = cfg.catalog_url.replacen("{}", "7481975434217786393", 1);
    test(&api, "", &url, "catalog?bookId=...");

    println!("\n  ── 内容 ──");
    for tmpl in &cfg.content_urls {
        let url = tmpl.replacen("{}", "7481975434217786393", 1);
        test(&api, "", &url, "content?item_id=...");
    }

    println!();
}

fn run_function(args: Vec<String>, cfg: &Config) {
    if args.is_empty() {
        eprintln!("  err 需要函数名或管道\n  用法:\n  \
            fqdt function embed <file> [--lrc] [--cover <img>]\n  \
            fqdt function process <file> [--abr N] [--speed X] [--normalize] [--cmd <cmd>]\n  \
            fqdt function fetch-catalog <book_id>\n  \
            fqdt function fetch-content <item_id>\n  \
            fqdt function fetch-audio-url <item_id> [tone]\n  \
            fqdt function search <keyword> [-p <page>]\n  \
            fqdt function strip-html <file|->\n  \
            fqdt function compress <file> [--abr N]\n  \
            fqdt function embed-lrc <file>\n  \
            fqdt function embed-cover <file> <cover>\n  \
            fqdt function read-info <dir>\n  \
            fqdt function read-audio-info <dir>\n  \
            fqdt function ... \\; ... \\; ...  (chaining with ;)");
        return;
    }

    let sep_positions: Vec<usize> = args.iter().enumerate()
        .filter(|(_, s)| *s == ";").map(|(i, _)| i).collect();

    if sep_positions.is_empty() {
        let out = dispatch_one(&args, cfg, "");
        if !out.is_empty() { println!("{}", out); }
    } else {
        let mut segments: Vec<&[String]> = vec![];
        let mut start = 0usize;
        for &pos in &sep_positions {
            if pos > start {
                segments.push(&args[start..pos]);
            }
            start = pos + 1;
        }
        if start < args.len() {
            segments.push(&args[start..]);
        }

        let mut piped = String::new();
        for seg in &segments {
            if seg.is_empty() { continue; }
            let resolved: Vec<String> = seg.iter()
                .map(|a| a.replace("{}", &piped))
                .collect();
            piped = dispatch_one(&resolved, cfg, &piped);
            if !piped.is_empty() {
                println!("{}", piped);
            }
        }
    }
}

fn dispatch_one(args: &[String], cfg: &Config, _piped: &str) -> String {
    fn err(e: String) -> String { eprintln!("  err {}", e); String::new() }

    if args.is_empty() { return String::new(); }
    let func = &args[0];
    let rest: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();

    match func.as_str() {
        "embed" | "emb" => {
            if rest.is_empty() { return err("需要 input 路径".into()); }
            let input = rest[0].to_string();
            let lrc = rest.contains(&"--lrc");
            let cover = rest.iter().position(|a| *a == "--cover").and_then(|i| rest.get(i+1).map(|s| s.to_string()));
            workflow::embed::run(&input, lrc, cover, cfg);
            input
        }
        "process" | "proc" => {
            if rest.is_empty() { return err("需要 input 路径".into()); }
            let input = rest[0].to_string();
            let abr = rest.iter().position(|a| *a == "--abr").and_then(|i| rest.get(i+1).and_then(|s| s.parse::<u32>().ok()));
            let speed = rest.iter().position(|a| *a == "--speed").and_then(|i| rest.get(i+1).and_then(|s| s.parse::<f32>().ok()));
            let normalize = rest.contains(&"--normalize");
            let cmd = rest.iter().position(|a| *a == "--cmd").and_then(|i| rest.get(i+1).map(|s| s.to_string()));
            workflow::process::run(&input, abr, speed, normalize, cmd, cfg);
            input
        }
        "fetch-catalog" | "cat" => {
            if rest.is_empty() { return err("需要 book_id".into()); }
            let api = api::Client::from_config(cfg, cfg.verbose);
            match api.fetch_catalog(rest[0]) {
                Ok(chs) => {
                    let json = serde_json::to_string_pretty(&chs).unwrap_or_default();
                    println!("{}", json);
                    chs.iter().map(|c| c.item_id.clone()).next().unwrap_or_default()
                }
                Err(e) => err(e)
            }
        }
        "fetch-content" | "content" => {
            if rest.is_empty() { return err("需要 item_id".into()); }
            let api = api::Client::from_config(cfg, cfg.verbose);
            match api.fetch_content(rest[0]) {
                Ok(text) => { println!("{}", text); text }
                Err(e) => err(e)
            }
        }
        "fetch-audio-url" | "audio-url" => {
            if rest.is_empty() { return err("需要 item_id".into()); }
            let tone: usize = rest.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
            let api = api::Client::from_config(cfg, cfg.verbose);
            match api.fetch_audio_url(rest[0], tone) {
                Ok(url) => { println!("{}", url); url }
                Err(e) => err(e)
            }
        }
        "search" => {
            if rest.is_empty() { return err("需要 keyword".into()); }
            let keyword = rest[0].to_string();
            let page: usize = rest.iter().position(|a| *a == "-p" || *a == "--page")
                .and_then(|i| rest.get(i+1).and_then(|s| s.parse().ok())).unwrap_or(1);
            let api = api::Client::from_config(cfg, cfg.verbose);
            match api.search(&keyword, page) {
                Ok(books) => {
                    let json = serde_json::to_string_pretty(&books).unwrap_or_default();
                    println!("{}", json);
                    books.iter().map(|b| b.book_id.clone()).next().unwrap_or_default()
                }
                Err(e) => err(e)
            }
        }
        "strip-html" | "html" => {
            if rest.is_empty() { return err("需要文件路径或 -".into()); }
            let text = if rest[0] == "-" {
                let mut buf = String::new();
                std::io::stdin().read_line(&mut buf).ok();
                buf
            } else {
                std::fs::read_to_string(rest[0]).unwrap_or_default()
            };
            let out = api::strip_html(&text);
            println!("{}", out);
            out
        }
        "compress" | "mp3" => {
            if rest.is_empty() { return err("需要文件路径".into()); }
            let input = rest[0].to_string();
            let abr: u32 = rest.iter().position(|a| *a == "--abr")
                .and_then(|i| rest.get(i+1).and_then(|s| s.parse().ok())).unwrap_or(32);
            let path = std::path::Path::new(&input);
            audio::post_process(path, abr, None, false, "", cfg.verbose);
            println!("  ok {}", input);
            input
        }
        "embed-lrc" | "lrc" => {
            if rest.is_empty() { return err("需要 MP3 路径".into()); }
            let input = rest[0].to_string();
            let path = std::path::Path::new(&input);
            let ch = types::Chapter {
                index: 0,
                title: path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
                item_id: String::new(),
            };
            audio::embed_lrc(path, &ch, cfg.verbose, None);
            println!("  ok {}", input);
            input
        }
        "embed-cover" | "cover" => {
            if rest.len() < 2 { return err("需要 MP3 路径和封面路径".into()); }
            let mp3 = rest[0];
            let cover = rest[1];
            audio::embed_cover(std::path::Path::new(mp3), std::path::Path::new(cover), cfg.verbose);
            println!("  ok {} ← {}", mp3, cover);
            mp3.to_string()
        }
        "read-info" | "info" => {
            if rest.is_empty() { return err("需要目录路径".into()); }
            match download::read_info_list(std::path::Path::new(rest[0])) {
                Ok((bid, title, fmt, chapters)) => {
                    let out = format!("book_id={}, title={}, format={}, chapters={}",
                        bid, title, fmt, chapters.len());
                    println!("  {}", out);
                    out
                }
                Err(e) => err(e)
            }
        }
        "read-audio-info" | "audio-info" => {
            if rest.is_empty() { return err("需要目录路径".into()); }
            let dir = std::path::Path::new(rest[0]);
            let audio_dir = if dir.join("Audio").exists() { dir.join("Audio") } else { dir.to_path_buf() };
            match download::read_audio_info_list(&audio_dir) {
                Ok((bid, title, chapters)) => {
                    let out = format!("book_id={}, title={}, chapters={}", bid, title, chapters.len());
                    println!("  {}", out);
                    out
                }
                Err(e) => err(e)
            }
        }
        "fetch-detail" | "detail" => {
            if rest.is_empty() { return err("需要 book_id".into()); }
            let api = api::Client::from_config(cfg, cfg.verbose);
            match api.fetch_detail(rest[0]) {
                Ok(info) => { println!("  {}", info); info }
                Err(e) => err(e)
            }
        }
        "fetch-content-batch" | "batch" => {
            if rest.len() < 2 { return err("需要 book_id 和 item_ids".into()); }
            let item_ids: Vec<&str> = rest[1..].to_vec();
            let api = api::Client::from_config(cfg, cfg.verbose);
            match api.fetch_content_batch(rest[0], &item_ids) {
                Ok(map) => {
                    for (id, content) in &map {
                        println!("  [{}]\n{}\n", id, content);
                    }
                    map.into_values().next().unwrap_or_default()
                }
                Err(e) => err(e)
            }
        }
        _ => err(format!("未知函数: {}", func))
    }
}

fn flush() {
    use std::io::Write;
    std::io::stdout().flush().unwrap();
}
