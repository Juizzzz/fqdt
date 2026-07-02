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
use std::path::Path;

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
    /// 查看目录或内容（简写 i）
    #[command(alias = "i")]
    Info {
        book_id: String,
        #[arg(short='r', long, help = "章节范围")]
        range: Option<String>,
        #[arg(short='s', long, help = "显示正文内容")]
        show: bool,
        #[arg(short='v', long, help = "详细输出")]
        verbose: bool,
    },
    /// 统一下载命令：正文/音频/增量/TTS
    ///
    /// 默认下载正文。加 --audio 同时下载音频，--audio-only 仅音频，--update 增量更新。
    /// 目录作为参数时自动进入增量模式。--tts 转换文本为语音。
    #[command(alias = "g")]
    Get {
        /// 书籍 ID 或本地目录
        target: Option<String>,
        #[arg(short='o', long, help = "输出目录")]
        output: Option<String>,
        #[arg(short='r', long, help = "章节范围 (1-50 / -5 / 10-)")]
        range: Option<String>,
        #[arg(short='t', long, help = "输出格式 (txt/epub)")]
        format: Option<String>,
        #[arg(short='j', long, help = "并发数")]
        jobs: Option<usize>,
        #[arg(short='f', long, help = "强制覆盖已存在文件")]
        force: bool,
        #[arg(short='v', long, help = "详细输出")]
        verbose: bool,

        // 音频选项
        #[arg(long, help = "同时下载音频")]
        audio: bool,
        #[arg(long, help = "仅下载音频，不下载正文")]
        audio_only: bool,
        #[arg(long, default_value = "1", help = "音色编号")]
        tone: usize,
        #[arg(long, help = "歌词模式: external/embed/both/off (默认 external)")]
        lrc: Option<String>,

        // 增量更新
        #[arg(long, help = "增量更新模式")]
        update: bool,

        // TTS
        #[arg(long, help = "TTS 文本文件或目录路径")]
        tts: Option<String>,
        #[arg(long, default_value = "zh-CN-XiaoxiaoNeural", help = "TTS 语音")]
        voice: String,

        // 压缩/后处理 (可同时用于音频和 TTS)
        #[arg(long, help = "MP3 压缩码率 (0=跳过)")]
        abr: Option<u32>,
        #[arg(long, help = "变速播放 (0.5=半速, 2.0=双倍)")]
        speed: Option<f32>,
        #[arg(long, help = "归一化音量")]
        normalize: bool,
        #[arg(short='e', long, help = "后处理命令模板 ({input} {output})")]
        exec: Option<String>,

        // 高级
        #[arg(short='i', long, default_value = "0", help = "请求间隔(ms)")]
        interval: u64,
    },
    /// 书架管理（简写 s）
    #[command(alias = "s")]
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
    /// 小工具集 + 函数管道（简写 fn）
    #[command(name = "function", alias = "fn")]
    Function {
        #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// [兼容] 下载章节正文（新用法: fqdt get <id>）
    #[command(hide = true)]
    Download {
        book_id: String,
        #[arg(short='o', long)]
        output: Option<String>,
        #[arg(short='j', long)]
        jobs: Option<usize>,
        #[arg(short='r', long)]
        range: Option<String>,
        #[arg(short='t', long)]
        format: Option<String>,
        #[arg(long)]
        audio: bool,
        #[arg(long)]
        tone: Option<usize>,
        #[arg(long)]
        abr: Option<u32>,
        #[arg(long)]
        lrc: Option<String>,
        #[arg(short='f', long)]
        force: bool,
        #[arg(short='v', long)]
        verbose: bool,
        #[arg(short='i', long)]
        interval: u64,
    },
    /// [兼容] 增量更新（新用法: fqdt get <id> --update）
    #[command(hide = true)]
    Update {
        book_id: Option<String>,
        #[arg(short='o', long)]
        output: Option<String>,
        #[arg(short='j', long)]
        jobs: Option<usize>,
        #[arg(short='r', long)]
        range: Option<String>,
        #[arg(short='f', long)]
        force: bool,
        #[arg(long)]
        audio: bool,
        #[arg(short='v', long)]
        verbose: bool,
        #[arg(short='i', long)]
        interval: u64,
    },
    /// [兼容] 下载语音或 TTS（新用法: fqdt get <id> --audio / --tts）
    #[command(hide = true)]
    Audio {
        book_id: Option<String>,
        #[arg(short='o', long)]
        output: Option<String>,
        #[arg(short='r', long)]
        range: Option<String>,
        #[arg(long)]
        tone: Option<usize>,
        #[arg(short='t', long)]
        tts: Option<String>,
        #[arg(long)]
        voice: Option<String>,
        #[arg(long)]
        rate: Option<String>,
        #[arg(long)]
        volume: Option<String>,
        #[arg(long)]
        pitch: Option<String>,
        #[arg(long)]
        abr: Option<u32>,
        #[arg(long)]
        speed: Option<f32>,
        #[arg(long)]
        normalize: bool,
        #[arg(short='e', long)]
        exec: Option<String>,
        #[arg(long)]
        lrc: Option<String>,
        #[arg(short='f', long)]
        force: bool,
        #[arg(short='j', long)]
        jobs: Option<usize>,
        #[arg(short='v', long)]
        verbose: bool,
        #[arg(short='i', long)]
        interval: u64,
    },
    /// [兼容] 测试 API 连接
    #[command(name = "test-api", hide = true)]
    TestApi,
    /// 自定义命令（来自 config.ini [workflow_cmd]）
    #[command(external_subcommand)]
    Custom(Vec<String>),
}

fn main() {
    let cli = Cli::parse();
    let mut cfg = Config::load();
    cfg.apply_cli_overrides(
        cli.search_url.as_deref(),
        cli.catalog_url.as_deref(),
        cli.content_url.as_deref(),
    );
    if let Some(to) = cli.timeout { cfg.timeout = to; }
    if let Some(h) = cli.http { cfg.http_method = h; }
    if let Some(c) = cli.curl_args { cfg.curl_args = c; }
    cfg.ensure_dirs();
    Config::save_default().ok();

    match cli.cmd {
        Cmd::Search { keyword, page, auto, dry_run, output, jobs, range, format, interval, verbose } =>
            workflow::search::run(&keyword, page, output.as_deref(), jobs, range.as_deref(),
                format.as_deref(), verbose, auto, dry_run, interval, &cfg),
        Cmd::Info { book_id, range, show, verbose } =>
            workflow::info::run(&book_id, range.as_deref(), show, verbose, &cfg),
        Cmd::Get { target, output, range, format, jobs, force, verbose, audio, audio_only, tone, lrc, update, tts, voice, abr, speed, normalize, exec, interval } =>
            dispatch_get(target, output, range, format, jobs, force, verbose, audio, audio_only, tone, lrc, update, tts, &voice, abr, speed, normalize, exec, interval, &cfg),
        Cmd::Shelf { add, delete, dl } => workflow::shelf::run(add, delete, dl, &cfg),
        Cmd::Init => { Config::save_default().ok(); println!("  ok ~/.config/fqdt/config.ini"); }
        Cmd::Function { args } => run_function(args, &cfg),
        Cmd::Download { book_id, output, jobs, range, format, audio, tone, abr, lrc, force, verbose, interval } => {
            eprintln!("  \x1b[2m提示: 改用 fqdt get {} [--audio] [选项...]\x1b[0m", book_id);
            dispatch_get(Some(book_id), output, range, format, jobs, force, verbose, audio, false, tone.unwrap_or(1), lrc, false, None, "zh-CN-XiaoxiaoNeural", abr, None, false, None, interval, &cfg);
        }
        Cmd::Update { book_id, output, jobs, range, force, audio, verbose, interval } => {
            if let Some(ref bid) = book_id { eprintln!("  \x1b[2m提示: 改用 fqdt get {} --update [选项...]\x1b[0m", bid); }
            dispatch_get(book_id, output, range, None, jobs, force, verbose, audio, false, 1, None, true, None, "zh-CN-XiaoxiaoNeural", None, None, false, None, interval, &cfg);
        }
        Cmd::Audio { book_id, output, range, tone, tts, voice, rate: _, volume: _, pitch: _, abr, speed, normalize, exec, lrc, force, jobs, verbose, interval } => {
            if let Some(ref bid) = book_id { eprintln!("  \x1b[2m提示: 改用 fqdt get {} --audio-only [选项...]\x1b[0m", bid); }
            if tts.is_some() { eprintln!("  \x1b[2m提示: 改用 fqdt get --tts <path> [选项...]\x1b[0m"); }
            let voice = voice.as_deref().unwrap_or("zh-CN-XiaoxiaoNeural");
            dispatch_get(book_id, output, range, None, jobs, force, verbose, false, true, tone.unwrap_or(1), lrc, false, tts, voice, abr, speed, normalize, exec, interval, &cfg);
        }
        Cmd::TestApi => test_api(&cfg),
        Cmd::Custom(args) => dispatch_custom(args, &cfg),
    }
}

fn dispatch_custom(args: Vec<String>, cfg: &Config) {
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("");
    // 从 config.ini [workflow_cmd] 查找自定义命令
    if let Some(template) = cfg.custom_commands.get(cmd) {
        let rest: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
        let cmd_str = if rest.is_empty() {
            template.clone()
        } else {
            let mut s = template.clone();
            for arg in &rest {
                s = s.replacen("{}", arg, 1);
            }
            s
        };
        if cfg.verbose { eprintln!("  [verbose] exec: {}", cmd_str); }
        match std::process::Command::new("sh").arg("-c").arg(&cmd_str).output() {
            Ok(out) => {
                print!("{}", String::from_utf8_lossy(&out.stdout));
                if !out.status.success() {
                    eprintln!("  err 退出码 {}", out.status);
                }
            }
            Err(e) => eprintln!("  err 执行失败: {}", e),
        }
        return;
    }
    eprintln!("  err 未知命令: {}", cmd);
    eprintln!("  可用命令: search, info, get, shelf, init, function");
}

#[allow(clippy::too_many_arguments)]
fn dispatch_get(target: Option<String>, output: Option<String>, range: Option<String>,
                format: Option<String>, jobs: Option<usize>, force: bool, verbose: bool,
                audio: bool, audio_only: bool, tone: usize, lrc: Option<String>,
                update: bool, tts: Option<String>, voice: &str,
                abr: Option<u32>, speed: Option<f32>, normalize: bool,
                exec: Option<String>, interval: u64, cfg: &Config) {
    let vb = verbose || cfg.verbose;
    let lrc_mode = lrc.as_deref().unwrap_or("external");
    let abr_val = abr.unwrap_or(cfg.abr);
    let post_cmd = exec.as_deref().unwrap_or(&cfg.post_process);

    // TTS 模式
    if let Some(tts_path) = tts {
        workflow::audio::run_tts(&tts_path, output.as_deref(), voice, cfg.tts_rate.as_str(),
            cfg.tts_volume.as_str(), cfg.tts_pitch.as_str(), abr_val, speed, normalize, post_cmd, lrc_mode, vb);
        return;
    }

    let target = match target {
        Some(t) => t,
        None => { eprintln!("  err 需要书籍 ID、目录、--tts 或 --audio-only"); return; }
    };
    let target_path = Path::new(&target);

    // 目录模式 = 增量更新
    if target_path.is_dir() {
        let p = types::DownloadParams {
            output: None, range, format, concurrent: jobs, audio: audio || audio_only,
            tone, abr: abr_val, lrc: lrc_mode.into(), force, verbose: vb, book_title: None,
        };
        workflow::download::run_dir(target_path, &p, vb, cfg);
        return;
    }

    // 纯音频下载
    if audio_only {
        let tone_val = tone;
        let fallbacks = if cfg.audio_tone_fallbacks.is_empty() {
            vec![2, 4, 5, 6, 74, 91]
        } else { cfg.audio_tone_fallbacks.clone() };
        let out = output
            .map(|s| std::path::PathBuf::from(s).join("Audio"))
            .unwrap_or_else(|| cfg.output_dir.join("Audio"));
        workflow::download::fetch_and_dl_audio(&target, &out, &range, tone_val, abr_val, lrc_mode, force, &fallbacks, vb, cfg);
        return;
    }

    // 增量更新
    if update {
        workflow::update::run(Some(&target), output.as_deref(), jobs, range.as_deref(), force, audio, vb, interval, cfg);
        return;
    }

    // 默认：下载正文（可选带音频）
    let p = types::DownloadParams {
        output, range, format, concurrent: jobs, audio, tone, abr: abr_val,
        lrc: lrc_mode.into(), force, verbose: vb, book_title: None,
    };
    workflow::download::run(&target, &p, cfg);
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
