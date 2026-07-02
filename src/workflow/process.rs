use crate::audio;
use crate::types::Config;

pub fn run(input: &str, abr: Option<u32>, speed: Option<f32>, normalize: bool,
           cmd: Option<String>, cfg: &Config) {
    let vb = cfg.verbose;
    let p = std::path::Path::new(input);
    if !p.exists() {
        eprintln!("  err {} 不存在", input);
        return;
    }
    let abr_val = abr.unwrap_or(0);
    let cmd_ref = cmd.as_deref().unwrap_or("");

    if p.is_dir() {
        for e in std::fs::read_dir(p).unwrap().flatten() {
            let path = e.path();
            if path.extension().map(|x| x == "mp3").unwrap_or(false) {
                audio::post_process(&path, abr_val, speed, normalize, cmd_ref, vb);
                println!("  ok {}", path.display());
            }
        }
    } else {
        audio::post_process(p, abr_val, speed, normalize, cmd_ref, vb);
        println!("  ok {}", p.display());
    }
}
