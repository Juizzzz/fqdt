mod api;
mod audio;
mod config;
mod platform;
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
    /// 下载正文（简写 d）
    #[command(alias = "d")]
    Download {
        book_id: String,
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
        #[arg(short='i', long, default_value = "0", help = "请求间隔(ms)")]
        interval: u64,
    },
    /// 下载语音（简写 a）
    #[command(alias = "a")]
    Audio {
        book_id: String,
        #[arg(short='o', long, help = "输出目录")]
        output: Option<String>,
        #[arg(short='r', long, help = "章节范围 (1-50 / -5 / 10-)")]
        range: Option<String>,
        #[arg(long, default_value = "1", help = "音色编号")]
        tone: usize,
        #[arg(long, help = "歌词模式: external/embed/both/off (默认 external)")]
        lrc: Option<String>,
        #[arg(short='j', long, help = "并发数")]
        jobs: Option<usize>,
        #[arg(short='f', long, help = "强制覆盖已存在文件")]
        force: bool,
        #[arg(short='v', long, help = "详细输出")]
        verbose: bool,
        #[arg(short='i', long, default_value = "0", help = "请求间隔(ms)")]
        interval: u64,
    },
    /// 增量更新已下载的书籍（简写 u）
    #[command(alias = "u")]
    Update {
        target: String,
        #[arg(short='o', long, help = "输出目录")]
        output: Option<String>,
        #[arg(short='j', long, help = "并发数")]
        jobs: Option<usize>,
        #[arg(short='f', long, help = "强制覆盖已存在文件")]
        force: bool,
        #[arg(short='v', long, help = "详细输出")]
        verbose: bool,
        #[arg(short='i', long, default_value = "0", help = "请求间隔(ms)")]
        interval: u64,
    },
    /// 文本转语音（简写 t）
    #[command(alias = "t")]
    Tts {
        path: String,
        #[arg(long, default_value = "zh-CN-XiaoxiaoNeural", help = "TTS 语音")]
        voice: String,
        #[arg(short='j', long, help = "并发数")]
        jobs: Option<usize>,
        #[arg(short='v', long, help = "详细输出")]
        verbose: bool,
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
        #[arg(short='U', long)]
        update: bool,
    },
    /// 生成默认配置
    Init,
    /// 检查平台、配置路径及可选依赖（离线）
    Doctor,
    /// 小工具集 + 函数管道（简写 fn）
    #[command(name = "function", alias = "fn")]
    Function {
        #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// [兼容] 统一下载命令
    #[command(hide = true)]
    Get {
        target: Option<String>,
        #[arg(short='o', long)] output: Option<String>,
        #[arg(short='r', long)] range: Option<String>,
        #[arg(short='t', long)] format: Option<String>,
        #[arg(short='j', long)] jobs: Option<usize>,
        #[arg(short='f', long)] force: bool,
        #[arg(short='v', long)] verbose: bool,
        #[arg(long)] audio: bool,
        #[arg(long)] audio_only: bool,
        #[arg(long, default_value = "1")] tone: usize,
        #[arg(long)] lrc: Option<String>,
        #[arg(long)] update: bool,
        #[arg(short='i', long, default_value = "0")] interval: u64,
    },
    /// 自定义命令（来自 config.ini [workflow_cmd]）
    #[command(external_subcommand)]
    Custom(Vec<String>),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if matches!(cli.cmd, Cmd::Doctor) {
        platform::doctor();
        return;
    }
    let mut cfg = Config::load();
    cfg.apply_cli_overrides(
        cli.search_url.as_deref(),
        cli.catalog_url.as_deref(),
        cli.content_url.as_deref(),
    );
    if let Some(to) = cli.timeout { cfg.timeout = to; }
    cfg.ensure_dirs();
    cfg.validate();
    if let Err(e) = Config::save_default() {
        eprintln!("  err 无法创建配置: {}", e);
        std::process::exit(1);
    }

    match cli.cmd {
        Cmd::Search { keyword, page, auto, dry_run, output, jobs, range, format, interval, verbose } =>
            workflow::search::run(&keyword, page, output.as_deref(), jobs, range.as_deref(),
                format.as_deref(), verbose, auto, dry_run, interval, &cfg).await,
        Cmd::Info { book_id, range, show, verbose } =>
            workflow::info::run(&book_id, range.as_deref(), show, verbose, &cfg).await,
        Cmd::Download { book_id, output, range, format, jobs, force, verbose, interval } =>
            dispatch_download(book_id, output, range, format, jobs, force, verbose, interval, &cfg).await,
        Cmd::Audio { book_id, output, range, tone, lrc, jobs, force, verbose, interval } =>
            dispatch_audio(book_id, output, range, tone, lrc, jobs, force, verbose, interval, &cfg).await,
        Cmd::Update { target, output, jobs, force, verbose, interval } =>
            dispatch_update(target, output, jobs, force, verbose, interval, &cfg).await,
        Cmd::Tts { path, voice, jobs, verbose } =>
            dispatch_tts(path, &voice, jobs, verbose, &cfg).await,
        Cmd::Get { target, output, range, format, jobs, force, verbose, audio, audio_only, tone, lrc, update, interval } =>
            dispatch_get(target, output, range, format, jobs, force, verbose, audio, audio_only, tone, lrc, update, interval, &cfg).await,
        Cmd::Shelf { add, delete, dl, update } => workflow::shelf::run(add, delete, dl, update, &cfg).await,
        Cmd::Init => println!("  ok {}", platform::AppPaths::discover().config.join("config.ini").display()),
        Cmd::Doctor => unreachable!(),
        Cmd::Function { args } => run_function(args, &cfg).await,
        Cmd::Custom(args) => dispatch_custom(args, &cfg),
    }
}

fn dispatch_custom(args: Vec<String>, cfg: &Config) {
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("");
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
    eprintln!("  可用命令: search, info, download, audio, update, tts, shelf, init, function");
}

#[allow(clippy::too_many_arguments)]
async fn dispatch_download(book_id: String, output: Option<String>, range: Option<String>,
    format: Option<String>, jobs: Option<usize>, force: bool, verbose: bool,
    _interval: u64, cfg: &Config) {
    let vb = verbose || cfg.verbose;
    let p = types::DownloadParams {
        output, range, format, concurrent: jobs,
        audio: false, tone: 1, abr: cfg.abr,
        lrc: "external".into(), force, verbose: vb, book_title: None,
    };
    workflow::download::run(&book_id, &p, cfg).await;
}

#[allow(clippy::too_many_arguments)]
async fn dispatch_audio(book_id: String, output: Option<String>, range: Option<String>,
    tone: usize, lrc: Option<String>, _jobs: Option<usize>, force: bool, verbose: bool,
    _interval: u64, cfg: &Config) {
    let vb = verbose || cfg.verbose;
    let lrc_mode = lrc.as_deref().unwrap_or("external");
    let path = Path::new(&book_id);

    if path.is_dir() {
        let p = types::DownloadParams {
            output: None, range, format: None, concurrent: None,
            audio: true, tone, abr: cfg.abr,
            lrc: lrc_mode.into(), force, verbose: vb, book_title: None,
        };
        workflow::download::run_dir(path, &p, vb, cfg).await;
        return;
    }

    let fallbacks = if cfg.audio_tone_fallbacks.is_empty() {
        vec![2, 4, 5, 6, 74, 91]
    } else { cfg.audio_tone_fallbacks.clone() };
    let out = output
        .map(|s| std::path::PathBuf::from(s).join("Audio"))
        .unwrap_or_else(|| cfg.output_dir.join("Audio"));
    workflow::download::fetch_and_dl_audio(&book_id, &out, &range, tone, cfg.abr, lrc_mode, force, &fallbacks, vb, cfg).await;
}

#[allow(clippy::too_many_arguments)]
async fn dispatch_update(target: String, output: Option<String>, jobs: Option<usize>,
    force: bool, verbose: bool, _interval: u64, cfg: &Config) {
    let vb = verbose || cfg.verbose;
    workflow::update::run(Some(&target), output.as_deref(), jobs, None, force, false, vb, _interval, cfg).await;
}

#[allow(clippy::too_many_arguments)]
async fn dispatch_tts(path: String, voice: &str, _jobs: Option<usize>, verbose: bool, cfg: &Config) {
    let vb = verbose || cfg.verbose;
    workflow::audio::run_tts(&path, None, voice, &cfg.tts_rate, &cfg.tts_volume, &cfg.tts_pitch,
        cfg.abr, None, false, "", "external", vb).await;
}

#[allow(clippy::too_many_arguments)]
async fn dispatch_get(target: Option<String>, output: Option<String>, range: Option<String>,
    format: Option<String>, jobs: Option<usize>, force: bool, verbose: bool,
    audio: bool, audio_only: bool, tone: usize, lrc: Option<String>,
    update: bool, interval: u64, cfg: &Config) {
    let vb = verbose || cfg.verbose;
    let target = match target {
        Some(t) => t,
        None => { eprintln!("  err 需要书籍 ID、目录或 --audio-only"); return; }
    };

    if update {
        workflow::update::run(Some(&target), output.as_deref(), jobs, range.as_deref(), force, audio, vb, interval, cfg).await;
        return;
    }

    if audio_only {
        let lrc_mode = lrc.as_deref().unwrap_or("external");
        let fallbacks = if cfg.audio_tone_fallbacks.is_empty() {
            vec![2, 4, 5, 6, 74, 91]
        } else { cfg.audio_tone_fallbacks.clone() };
        let out = output
            .map(|s| std::path::PathBuf::from(s).join("Audio"))
            .unwrap_or_else(|| cfg.output_dir.join("Audio"));
        workflow::download::fetch_and_dl_audio(&target, &out, &range, tone, cfg.abr, lrc_mode, force, &fallbacks, vb, cfg).await;
        return;
    }

    let lrc_mode = lrc.as_deref().unwrap_or("external");
    let p = types::DownloadParams {
        output, range, format, concurrent: jobs,
        audio, tone, abr: cfg.abr,
        lrc: lrc_mode.into(), force, verbose: vb, book_title: None,
    };
    workflow::download::run(&target, &p, cfg).await;
}

async fn run_function(args: Vec<String>, cfg: &Config) {
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
            fqdt function save <file>\n  \
            fqdt function save <file> (管道末端保存输出)\n  \
            fqdt function ... \\; ... \\; ...  (chaining with ;)\n  \
            \n  主命令:\n  \
            fqdt download <book_id> [选项]\n  \
            fqdt audio <book_id> [选项]\n  \
            fqdt update <book_id|目录> [选项]\n  \
            fqdt tts <路径> [选项]\n  \
            fqdt search <keyword> [选项]\n  \
            fqdt info <book_id> [选项]\n  \
            fqdt shelf [选项]\n  \
            fqdt init\n  \
            fqdt function <fn> [参数]");
        return;
    }

    let sep_positions: Vec<usize> = args.iter().enumerate()
        .filter(|(_, s)| *s == ";").map(|(i, _)| i).collect();

    if sep_positions.is_empty() {
        let out = dispatch_one(&args, cfg, "").await;
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
            piped = dispatch_one(&resolved, cfg, &piped).await;
            if !piped.is_empty() {
                println!("{}", piped);
            }
        }
    }
}

async fn dispatch_one(args: &[String], cfg: &Config, piped: &str) -> String {
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
            match api.fetch_catalog(rest[0]).await {
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
            match api.fetch_content(rest[0]).await {
                Ok(text) => { println!("{}", text); text }
                Err(e) => err(e)
            }
        }
        "fetch-audio-url" | "audio-url" => {
            if rest.is_empty() { return err("需要 item_id".into()); }
            let tone: usize = rest.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
            let api = api::Client::from_config(cfg, cfg.verbose);
            match api.fetch_audio_url(rest[0], tone).await {
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
            match api.search(&keyword, page).await {
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
            match api.fetch_detail(rest[0]).await {
                Ok(info) => { println!("  {}", info); info }
                Err(e) => err(e)
            }
        }
        "fetch-content-batch" | "batch" => {
            if rest.len() < 2 { return err("需要 book_id 和 item_ids".into()); }
            let item_ids: Vec<&str> = rest[1..].to_vec();
            let api = api::Client::from_config(cfg, cfg.verbose);
            match api.fetch_content_batch(rest[0], &item_ids).await {
                Ok(map) => {
                    for (id, content) in &map {
                        println!("  [{}]\n{}\n", id, content);
                    }
                    map.into_values().next().unwrap_or_default()
                }
                Err(e) => err(e)
            }
        }
        "save" => {
            if rest.is_empty() { return err("需要输出路径".into()); }
            let path = rest[0].to_string();
            if let Err(e) = std::fs::write(&path, piped) {
                return err(format!("写入失败: {}", e));
            }
            println!("  ok 保存 {} ({} 字节)", path, piped.len());
            piped.to_string()
        }
        _ => err(format!("未知函数: {}", func))
    }
}

