use crate::api::Client;
use crate::audio;
use crate::download;
use crate::types::{ChapterRange, Config};
use crate::util;
use std::io::Write;
use std::path::Path;

#[allow(clippy::too_many_arguments)]
pub fn run(book_id: &str, output: Option<&str>, concurrent: Option<usize>,
           range: Option<&str>, format: Option<&str>, audio: bool, tone: usize, abr: u32,
           lrc: &str, force: bool, verbose: bool, _interval: u64, cfg: &Config,
           book_title: Option<&str>) {
    let vb = verbose || cfg.verbose;
    let path = Path::new(book_id);

    if path.is_dir() {
        return run_dir(path, range, audio, tone, abr, lrc, force, vb, cfg, concurrent);
    }

    let api = Client::from_config(cfg, vb);
    print!("  获取目录... ");
    std::io::stdout().flush().unwrap();
    let all = match api.fetch_catalog(book_id) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("\n  err {}", e);
            return;
        }
    };
    let r = range.and_then(ChapterRange::parse);
    let chs = util::filter_by_range(&all, r.as_ref());
    if chs.is_empty() {
        println!("  err 空范围");
        return;
    }

    let fmt = format.unwrap_or(&cfg.format);
    let out = output
        .map(|s| s.into())
        .unwrap_or(cfg.output_dir.clone());
    let bt = book_title.unwrap_or_else(|| {
        if let Ok(detail) = api.fetch_detail(book_id) {
            let title = detail.split('|').next().unwrap_or("小说");
            let leaked: &'static str = Box::leak(title.to_string().into_boxed_str());
            leaked
        } else { "小说" }
    });
    let dler = download::Downloader::new(api, out.clone(), fmt, &cfg.filename_template, false, vb,
                                          book_id, bt);
    dler.run(&chs, concurrent.unwrap_or(cfg.concurrent));

    if audio {
        let fallbacks = if cfg.audio_tone_fallbacks.is_empty() {
            vec![2, 4, 5, 6, 74, 91]
        } else {
            cfg.audio_tone_fallbacks.clone()
        };
        fetch_and_dl_audio(book_id, &out.join("Audio"), range, tone, abr, lrc, force, &fallbacks, vb, cfg);
    }
}

#[allow(clippy::too_many_arguments)]
fn run_dir(path: &Path, range: Option<&str>, audio: bool, tone: usize, abr: u32,
           lrc: &str, force: bool, vb: bool, cfg: &Config, concurrent: Option<usize>) {
    let has_text = path.join("info.list").exists();
    let has_audio = path.join("Audio/info.list").exists();
    if !has_text && !has_audio {
        eprintln!("  err 目录中未找到 info.list");
        return;
    }

    if has_text {
        if let Ok((bid, btitle, fmt, existing)) = download::read_info_list(path) {
            let api = Client::from_config(cfg, vb);
            print!("  检查正文更新... ");
            std::io::stdout().flush().unwrap();
            if let Ok(all) = api.fetch_catalog(&bid) {
                let r = range.and_then(ChapterRange::parse);
                let new_chs: Vec<&crate::types::Chapter> = all
                    .iter()
                    .filter(|c| !existing.iter().any(|(idx, _, _)| *idx == c.index))
                    .filter(|c| r.as_ref().is_none_or(|x| x.contains(c.index)))
                    .collect();
                if new_chs.is_empty() {
                    println!("  正文已是最新 (共{}章)", all.len());
                } else {
                    println!("  发现 {} 章新正文 (共{}/{})", new_chs.len(), existing.len(), all.len());
                    let dler = download::Downloader::new(api, path.to_path_buf(), &fmt,
                                                         &cfg.filename_template, force, vb, &bid, &btitle);
                    dler.run(&new_chs, concurrent.unwrap_or(cfg.concurrent));
                }
            }
        }
    } else {
        println!("  正文已存在, 跳过");
    }

    if audio || has_audio {
        let audio_dir = path.join("Audio");
        let (bid, btitle, existing) = if has_audio {
            download::read_audio_info_list(&audio_dir).unwrap_or_default()
        } else {
            (String::new(), String::new(), vec![])
        };

        if bid.is_empty() {
            if let Ok((bid2, _, _, _)) = download::read_info_list(path) {
                let fallbacks = if cfg.audio_tone_fallbacks.is_empty() {
                    vec![2, 4, 5, 6, 74, 91]
                } else {
                    cfg.audio_tone_fallbacks.clone()
                };
                fetch_and_dl_audio(&bid2, &audio_dir, range, tone, abr, lrc, force, &fallbacks, vb, cfg);
            }
        } else {
            let api = Client::from_config(cfg, vb);
            print!("  检查音频更新... ");
            std::io::stdout().flush().unwrap();
            if let Ok(all) = api.fetch_catalog(&bid) {
                let r = range.and_then(ChapterRange::parse);
                let new_chs: Vec<&crate::types::Chapter> = all
                    .iter()
                    .filter(|c| !existing.iter().any(|(idx, _, _)| *idx == c.index))
                    .filter(|c| r.as_ref().is_none_or(|x| x.contains(c.index)))
                    .collect();
                if new_chs.is_empty() {
                    println!("  音频已是最新 (共{}章)", all.len());
                } else {
                    println!("  发现 {} 章新音频 (共{}/{})", new_chs.len(), existing.len(), all.len());
                    let fallbacks = if cfg.audio_tone_fallbacks.is_empty() {
                        vec![2, 4, 5, 6, 74, 91]
                    } else {
                        cfg.audio_tone_fallbacks.clone()
                    };
                    let dler = audio::AudioDownloader::new(api, audio_dir, audio::AudioParams {
                        tone, fallbacks,
                        ft: cfg.filename_template.clone(), force, verbose: vb,
                        abr, speed: None, normalize: false,
                        post_cmd: String::new(), lrc_mode: lrc.into(),
                    });
                    dler.run(&new_chs, Some(&btitle));
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn fetch_and_dl_audio(book_id: &str, audio_dir: &Path, range: Option<&str>, tone: usize,
                      abr: u32, lrc: &str, force: bool, fallbacks: &[usize],
                      verbose: bool, cfg: &Config) {
    let vb = verbose || cfg.verbose;
    let api = Client::from_config(cfg, vb);
    print!("  获取目录... ");
    std::io::stdout().flush().unwrap();
    match api.fetch_catalog(book_id) {
        Ok(all) => {
            if all.is_empty() {
                println!("  err 空目录");
                return;
            }
            let r = range.and_then(ChapterRange::parse);
            let chs = util::filter_by_range(&all, r.as_ref());
            if chs.is_empty() {
                println!("  err 空范围");
                return;
            }
            let fb = if fallbacks.is_empty() {
                vec![2, 4, 5, 6, 74, 91]
            } else {
                fallbacks.to_vec()
            };
            let dler = audio::AudioDownloader::new(api, audio_dir.to_path_buf(), audio::AudioParams {
                tone, fallbacks: fb,
                ft: cfg.filename_template.clone(), force, verbose: vb,
                abr, speed: None, normalize: false,
                post_cmd: String::new(), lrc_mode: lrc.into(),
            });
            dler.run(&chs, None);
        }
        Err(e) => eprintln!("  err {}", e),
    }
}
