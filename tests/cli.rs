use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "fqdt cli 中文 {} {}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_fqdt"))
            .args(args)
            .env("FQDT_CONFIG_DIR", self.0.join("配置"))
            .env("FQDT_CACHE_DIR", self.0.join("缓存"))
            .current_dir(&self.0)
            .output().unwrap()
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
}

#[test]
fn doctor_is_read_only() {
    let s = Sandbox::new();
    let result = s.run(&["doctor"]);
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("config.ini"));
    assert!(!s.0.join("配置").exists());
    assert!(!s.0.join("缓存").exists());
}

#[test]
fn init_preserves_custom_config() {
    let s = Sandbox::new();
    assert!(s.run(&["init"]).status.success());
    let config = s.0.join("配置/config.ini");
    assert!(config.exists());
    fs::write(&config, "[download]\nconcurrent = 3\n# 保留我的设置\n").unwrap();
    assert!(s.run(&["init"]).status.success());
    assert!(fs::read_to_string(config).unwrap().contains("保留我的设置"));
}

#[test]
fn init_failure_has_nonzero_exit() {
    let s = Sandbox::new();
    fs::write(s.0.join("配置"), "这里是文件，不是目录").unwrap();
    let result = s.run(&["init"]);
    assert!(!result.status.success());
    assert!(!String::from_utf8_lossy(&result.stdout).contains("  ok "));
}

#[test]
fn bookshelf_roundtrip_in_unicode_path() {
    let s = Sandbox::new();
    assert!(s.run(&["shelf", "--add", "123:中文测试书"]).status.success());
    let result = s.run(&["shelf"]);
    assert!(String::from_utf8_lossy(&result.stdout).contains("中文测试书"));
    assert!(s.run(&["shelf", "--delete", "1"]).status.success());
    assert_eq!(fs::read_to_string(s.0.join("配置/books.txt")).unwrap(), "");
}

#[test]
fn help_and_version_do_not_create_config() {
    let s = Sandbox::new();
    assert!(s.run(&["--help"]).status.success());
    assert!(s.run(&["--version"]).status.success());
    assert!(!s.0.join("配置").exists());
}
