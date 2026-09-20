use std::path::{Path, PathBuf};

use crate::types::get_home;

/// 配置与缓存目录；由配置、书架和 doctor 工作流共同使用。
pub struct AppPaths {
    pub config: PathBuf,
    pub cache: PathBuf,
}

impl AppPaths {
    /// macOS 使用 Library；已有旧配置或书架时继续使用旧目录，不移动用户文件。
    pub fn discover() -> Self {
        let home = get_home();
        let mut paths = Self::for_home(&home, cfg!(target_os = "macos"));
        if let Some(dir) = std::env::var_os("FQDT_CONFIG_DIR").filter(|v| !v.is_empty()) {
            paths.config = PathBuf::from(dir);
        }
        if let Some(dir) = std::env::var_os("FQDT_CACHE_DIR").filter(|v| !v.is_empty()) {
            paths.cache = PathBuf::from(dir);
        }
        paths
    }

    fn for_home(home: &Path, macos: bool) -> Self {
        let legacy = home.join(".config/fqdt");
        let config = if macos
            && !legacy.join("config.ini").exists()
            && !legacy.join("books.txt").exists()
        {
            home.join("Library/Application Support/fqdt")
        } else {
            legacy
        };
        let cache = if macos {
            home.join("Library/Caches/fqdt")
        } else {
            config.join("cache")
        };
        Self { config, cache }
    }
}

/// 在 PATH 和 macOS 常见安装目录查找外部工具；用于音频及 doctor。
pub fn find_tool(name: &str) -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    if cfg!(target_os = "macos") {
        dirs.extend([
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("/usr/local/bin"),
            get_home().join(".local/bin"),
        ]);
    }
    dirs.into_iter().map(|p| p.join(name)).find(|p| {
        p.metadata().is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    })
}

/// 保存长文本供 edge-tts --file 使用，避免 macOS 的命令行参数长度限制。
pub struct TemporaryText(pub PathBuf);

impl TemporaryText {
    pub fn create(text: &str) -> Result<Self, String> {
        use std::io::Write;
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        loop {
            let path = std::env::temp_dir().join(format!(
                "fqdt-tts-{}-{}.txt", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match std::fs::OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    let guard = Self(path);
                    file.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
                    return Ok(guard);
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(format!("无法创建 TTS 临时文件: {}", e)),
            }
        }
    }
}

impl Drop for TemporaryText {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// 显示本地运行环境及可选工具，不请求网络、不修改配置；CLI: fqdt doctor。
pub fn doctor() {
    let paths = AppPaths::discover();
    println!("  ok 平台: {} / {}", std::env::consts::OS, std::env::consts::ARCH);
    println!("  配置: {}", paths.config.join("config.ini").display());
    println!("  缓存: {}", paths.cache.display());
    println!("  书架: {}", paths.config.join("books.txt").display());
    for (name, purpose, install) in [
        ("curl", "音频下载", "macOS 自带 curl"),
        ("lame", "可选 MP3 压缩", "brew install lame"),
        ("edge-tts", "可选语音合成", "brew install pipx && pipx install edge-tts"),
    ] {
        match find_tool(name) {
            Some(path) => println!("  ok {}: {} ({})", name, path.display(), purpose),
            None => println!("  未找到 {} ({})；安装方法: {}", name, purpose, install),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_and_linux_paths() {
        let home = std::env::temp_dir().join(format!("fqdt-paths-{}", std::process::id()));
        let native = AppPaths::for_home(&home, true);
        assert_eq!(native.config, home.join("Library/Application Support/fqdt"));
        assert_eq!(native.cache, home.join("Library/Caches/fqdt"));
        let linux = AppPaths::for_home(&home, false);
        assert_eq!(linux.config, home.join(".config/fqdt"));
        assert_eq!(linux.cache, home.join(".config/fqdt/cache"));
    }

    #[test]
    fn legacy_bookmarks_are_preserved_without_config() {
        let home = std::env::temp_dir().join(format!("fqdt-legacy-{}", std::process::id()));
        let old = home.join(".config/fqdt");
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(old.join("books.txt"), "123:测试\n").unwrap();
        assert_eq!(AppPaths::for_home(&home, true).config, old);
        std::fs::remove_file(old.join("books.txt")).unwrap();
        std::fs::write(old.join("config.ini"), "[download]\n").unwrap();
        assert_eq!(AppPaths::for_home(&home, true).config, old);
        std::fs::remove_dir_all(&home).unwrap();
    }
}
