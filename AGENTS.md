# fqdt — 番茄小说下载器

- **版本**: v0.6.0（`Cargo.toml` 中定义，每次变更更新）
- **目标**: 番茄小说离线下载 + 音频获取 + 语音合成
- **运行**: macOS（Apple Silicon / Intel），兼容原 Linux / Termux
- **编译**: macOS 原生 Cargo 双架构；Linux 继续使用 cross
- **核心原则**: 基本函数原子化 → 高阶函数组合 → 预编译工作流 → 配置文件可扩展

---

## 1. 架构愿景

### 1.1 三层架构

```
[基本函数]  ← 原子操作，每个做且只做一件事
     ↓ 组合
[高阶函数]  ← 接收函数参数，包装行为（重试/缓存/日志/进度/过滤）
     ↓ 编排
[工作流]    ← 预定义的函数调用序列，一条命令完成复杂任务
     ↓ 扩展
[配置驱动]  ← config.ini 可注册自定义函数和工作流
```

### 1.2 基本函数（Atoms）

每个基本函数满足：
- **一个函数一个职责**
- **纯输入→输出**，不处理 UI/日志/并发（这些由高阶函数包装）
- **返回 `Result<T, String>`**，调用者决定错误处理
- **上方有文档注释**：功能说明 + 被哪些工作流调用 + 可被 `function` 子命令直接调用

| 分类 | 函数 | 输入 | 输出 | 被工作流调用 |
|------|------|------|------|-------------|
| **API** | `search_books()` | keyword, page | `Vec<Book>` | search |
|  | `fetch_catalog()` | book_id | `Vec<Chapter>` | download, update, info |
|  | `fetch_content()` | item_id | `String` | download_text, info --show |
|  | `fetch_audio_url()` | item_id, tone | `String` (URL) | download_audio |
|  | `http_get()` | url | `String` | 以上所有 |
| **正文** | `download_text_chapter()` | Chapter, dir, fmt | `PathBuf` | download, update |
|  | `generate_epub()` | title, chapters, path | `()` | download (epub模式) |
|  | `append_epub_chapter()` | epub_path, Chapter, text | `()` | download (epub模式) |
| **音频** | `download_audio_chapter()` | Chapter, dir, tone | `PathBuf` | download_audio, update_audio |
|  | `compress_mp3()` | input, output, abr | `()` | download_audio, process |
|  | `post_process_file()` | input, output, cmd | `()` | download_audio, process |
|  | `normalize_audio()` | input, output | `()` | process |
|  | `speed_audio()` | input, output, rate | `()` | process |
| **LRC** | `gen_lrc_text()` | Chapter | `String` | download_audio, embed |
|  | `write_lrc_file()` | path, text | `()` | download_audio, embed |
|  | `embed_lrc_uslt()` | mp3_path, text | `()` | download_audio, embed |
| **封面** | `embed_cover_apic()` | mp3_path, image_path | `()` | embed |
| **TTS** | `edge_tts_speak()` | text, voice, params | `PathBuf` | tts_convert |
|  | `convert_tts_file()` | txt_path, params | `PathBuf` | tts_convert |
|  | `convert_tts_dir()` | dir_path, params | `Vec<PathBuf>` | tts_convert |
| **元数据** | `read_info_list()` | dir | `(book_id, title, fmt, chapters)` | update |
|  | `write_info_list()` | dir, metadata | `()` | download, update |
|  | `read_audio_info_list()` | dir | `(book_id, title, chapters)` | update_audio |
|  | `write_audio_info_list()` | dir, metadata | `()` | download_audio |
| **书架** | `load_bookmarks()` | — | `Vec<(id, title)>` | shelf |
|  | `save_bookmark()` | id, title | `()` | shelf --add |
|  | `remove_bookmark()` | idx | `()` | shelf --del |
| **范围** | `parse_range()` | str | `Option<ChapterRange>` | download, update, info |
|  | `filter_by_range()` | chapters, range | `Vec<&Chapter>` | download, update, info |
| **工具** | `sanitize_filename()` | str | `String` | 全局 |
|  | `strip_html()` | html_str | `String` | fetch_content 后处理 |
|  | `chapter_heading()` | Chapter | `String` | dl_file, gen_lrc |
|  | `bar_style()` | — | `ProgressStyle` | 全局 |

### 1.3 高阶函数（Wrappers）

高阶函数接收一个基本函数作为参数，为它添加横切关注点：

```
fn with_verbose<F, T>(f: F, args, label) → T     // 调用 fn 并记录详细日志
fn with_cache<F>(f: F, args, cache_key, ttl) → T  // 缓存检查 → 调用 → 写入缓存
fn with_retry<F>(f: F, args, max_retries) → T      // 失败重试（指数退避）
fn with_progress<F>(f: F, items, concurrent) → T   // 并发执行 + 进度条
fn with_output<F>(f: F, args, format) → T          // 格式化输出
fn with_filter<F>(f: F, items, predicate) → T      // 先过滤再执行
fn with_elapsed<F>(f: F, args) → T                 // 计时并输出耗时
fn with_dry_run<F>(f: F, args) → T                 // 模拟执行，只打印不实际调用
```

**设计模式**：高阶函数不修改基本函数的签名，通过闭包包装：

```rust
// 高阶函数示例: with_retry
fn with_retry<T>(f: impl Fn() -> Result<T, String>, max: u32, label: &str) -> Result<T, String> {
    let mut last_err = String::new();
    for i in 0..max {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => { last_err = e; sleep(Duration::from_secs(1 << i)); }
        }
    }
    Err(format!("重试{}次失败: {}", max, last_err))
}

// 使用: 带重试的基本函数调用
let chapters = with_retry(|| api.fetch_catalog(bid), 3, "fetch_catalog")?;
```

```rust
// 更高阶: with_verbose + with_retry 可以链式组合
fn verbose_retry<F, T>(f: F, max: u32, label: &str, vb: bool) -> Result<T, String>
where F: Fn() -> Result<T, String> {
    let f1 = || { if vb { eprintln!("  [verbose] {} 开始", label); } f() };
    let result = with_retry(f1, max, label);
    if vb { eprintln!("  [verbose] {} 完成: {:?}", label, result.is_ok()); }
    result
}
```

### 1.4 工作流（Workflows）

工作流是预定义的函数调用序列。每个工作流：
- 是 `pub fn`，位于独立模块 `src/workflow/`
- 上方有**详细注释**：工作流流程、调用的基本函数、适用场景
- 不直接处理 `struct Client` 创建——由调用者传入

```rust
/// # 下载书籍工作流
///
/// ## 流程
/// 1. `api.fetch_catalog(book_id)` — 获取全本目录
/// 2. `filter_by_range(chapters, range)` — 按范围筛选
/// 3. `download::Downloader::run(chapters, concurrent)` — 并发下载正文
/// 4. `write_info_list(dir, meta)` — 写入元数据
/// 5. [可选] 如果 audio=true:
///    a. `api.fetch_catalog(book_id)` — 重新获取目录（已缓存）
///    b. `AudioDownloader::run(chapters, tone, abr, lrc)` — 并发下载音频
///    c. `write_audio_info_list(dir, meta)` — 写入音频元数据
///
/// ## 调用
/// - CLI: `fqdt download <book_id> [--audio] [--abr N]`
/// - Config: `[workflow.download]` 可覆盖步骤
pub fn workflow_download(api: &Client, book_id: &str, out: PathBuf, range: Option<&str>,
                         fmt: &str, ft: &str, concurrent: usize, force: bool, vb: bool,
                         audio: bool, tone: usize, abr: u32, lrc: &str, fallbacks: &[usize]) { ... }
```

预编译工作流清单：

| 工作流 | 命令 | 功能 |
|--------|------|------|
| `workflow_download` | `download` | 下载正文 + 可选音频 |
| `workflow_update` | `update` | 增量更新（正文/音频） |
| `workflow_audio` | `audio` | 纯音频下载 |
| `workflow_search_dl` | `search` | 搜索 → 选择 → 下载 |
| `workflow_tts_convert` | `audio --tts` | TTS 文件/目录转换 |
| `workflow_embed` | `function embed` | 嵌入 LRC/封面 |
| `workflow_process` | `function process` | 后处理（压缩/变速/归一化） |
| `workflow_shelf` | `shelf` | 书架管理 |

### 1.5 `function` 子命令

`function` 子命令**直接暴露基本函数**，供 CLI 调用：

```sh
fqdt function fetch-catalog <book_id>          # 获取目录并打印
fqdt function fetch-content <item_id>           # 获取正文并打印
fqdt function fetch-audio-url <item_id> <tone>  # 获取音频 URL
fqdt function compress <input.mp3> --abr 32     # 压缩 MP3
fqdt function gen-lrc <chapter.json>            # 生成 LRC 文本
fqdt function embed-lrc <file.mp3>              # 嵌入 LRC 到 ID3
fqdt function embed-cover <file.mp3> --cover img.jpg
fqdt function strip-html <file.html>            # 剥离 HTML 标签
fqdt function search <keyword>                  # 搜索并打印结果（JSON）
fqdt function read-info <dir>                   # 读取 info.list 并打印
```

**实现方式**：`function` 子命令是 clap 的 `#[command(subcommand)]`，每个基本函数映射到一个 `FunctionCmd` 变体。函数实现本身在各自的模块中（`api.rs`/`audio.rs`/`download.rs`），`main.rs` 只做分发。

### 1.6 配置驱动的扩展

`config.ini` 允许**不修改代码**添加新的 function 和工作流：

```ini
[function]
# 格式: name = command_template
# {} 会被 CLI 参数替换
decode_woff = "python3 ~/decode_woff.py {}"
fetch_raw = "curl -s 'https://api.example.com/raw?item_id={}'"

[function_arg]
# 可选: 定义参数名和默认值
decode_woff = "font_path,output"

[workflow]
# 格式: name = step1 | step2 | step3 ...
# 每一步是 function name 或 shell 命令
download_custom = "search $1 -D 1 | fetch-catalog {} | fetch-content {}"

[workflow_cmd]
# 注册为主命令（等价于 clap 子命令）
# 将出现在 `fqdt --help` 中
echo = "echo hello"
```

**实现方案**：CLI 解析时，先读 clap 静态命令，再读 `config.ini` 的 `[workflow_cmd]` 节，动态注入到匹配中。未识别的子命令回退到 config 查找。

---

## 2. 目录结构（目标状态）

```
src/
├── main.rs          CLI 入口 + 静态命令解析 + 配置命令注入
├── api.rs           Client 结构体 + 所有 HTTP 基本函数
├── audio.rs         AudioDownloader + 音频/封面/LRC 基本函数
├── download.rs      Downloader + 正文基本函数
├── config.rs        Config 解析 + 书架 + 配置驱动扩展
├── types.rs         数据结构定义
├── epub.rs          EPUB 生成函数
├── workflow.rs 或 workflow/
│   ├── mod.rs       工作流调度
│   ├── download.rs  workflow_download
│   ├── update.rs    workflow_update
│   ├── audio.rs     workflow_audio
│   ├── embed.rs     workflow_embed
│   └── process.rs   workflow_process
└── util.rs          工具函数 + 高阶函数
```

---

## 3. 注释规范

### 3.1 基本函数注释

每个 `pub fn` 基本函数必须带这样的注释：

```rust
/// 搜索小说
///
/// 调用 snssdk 官方搜索 API，返回匹配的书籍列表。
/// 结果按相关度排序，每页 10 本。
///
/// # 参数
/// - `keyword`: 搜索关键词（URL 编码自动处理）
/// - `page`: 页码（1-indexed）
///
/// # 被工作流调用
/// - `workflow_search_dl` — 搜索→选择→下载
///
/// # CLI 调用
/// - `fqdt function search <keyword> -p <page>`
///
/// # 可选高阶函数包装
/// - `with_verbose` — 打印 API URL 和响应摘要
/// - `with_cache` — 搜索结果可缓存（TTL=300s）
/// - `with_retry(3)` — 搜索 API 偶有超时，重试 3 次
pub fn search_books(api: &Client, keyword: &str, page: usize) -> Result<Vec<Book>, String> { ... }
```

### 3.2 工作流注释

```rust
/// # workflow_download — 下载书籍（正文 + 可选音频）
///
/// ## 前置条件
/// - `book_id` 有效
/// - 如果 `audio=true`, 需要 `lame` 命令可用（或 `abr=0` 跳过压缩）
///
/// ## 工作流图
/// ```
/// fetch_catalog(book_id)
///     │
///     ▼
/// filter_by_range(chapters, range)
///     │
///     ├──→ [正文] Downloader::run(chapters)
///     │       │
///     │       ▼
///     │   write_info_list(dir)
///     │
///     └──→ [音频] fetch_catalog(book_id)  ← 缓存命中则跳过
///             │
///             ▼
///         AudioDownloader::run(chapters)
///                 │
///                 ├──→ compress_mp3 (if abr>0)
///                 ├──→ post_process_file (if cmd not empty)
///                 └──→ handle_lrc (gen + embed)
/// ```
pub fn workflow_download(...) { ... }
```

---

## 4. 编码约定

### 4.1 命名

- 基本函数：`动词_名词()`，如 `fetch_catalog()`、`compress_mp3()`、`write_lrc_file()`
- 高阶函数：`with_动词()`，如 `with_retry()`、`with_progress()`
- 工作流：`workflow_名词()`，如 `workflow_download()`
- 结构体：`Downloader`、`AudioDownloader`、`Client`、`Config`、`ChapterRange`
- 变量：`snake_case`，避免缩写（除 `api`、`cfg`、`ch`、`idx` 等非常通用的）

### 4.2 函数长度

- 基本函数：**≤30 行**（超出则再拆分）
- 高阶函数：**≤20 行**
- 工作流：**≤60 行**（超出则拆为子步骤）

### 4.3 Option 参数

所有可选行为通过 `Option<T>` 参数控制，不通过全局变量或配置隐式改变：

```rust
// ✅ 正确：显式 Option 参数
pub fn compress_mp3(input: &Path, output: &Path, abr: Option<u32>) -> Result<(), String> {
    let rate = abr.unwrap_or(0);
    if rate == 0 { return Ok(()); }  // abr=None 或 0 表示不压缩
    // ... 实际压缩逻辑
}

// ❌ 错误：隐式依赖 cfg
pub fn compress_mp3(input: &Path, output: &Path) -> Result<(), String> {
    let cfg = Config::load();  // 不要这样！函数应该是纯的
    // ...
}
```

### 4.4 高阶函数模式

高阶函数的基本模式是接收闭包，包装行为：

```rust
/// 带重试的高阶函数
///
/// 接受一个返回 `Result<T, String>` 的闭包，在失败时自动重试。
/// 可用于任何基本函数（API 调用、文件写入等）。
pub fn with_retry<T>(f: impl Fn() -> Result<T, String>, max: u32) -> Result<T, String> {
    let mut last_err = String::new();
    for i in 0..max {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => { last_err = e; if i + 1 < max { sleep(Duration::from_secs(1 << i)); } }
        }
    }
    Err(format!("重试{}次后失败: {}", max, last_err))
}

/// 使用示例：将高阶函数应用到基本函数
fn do_search(api: &Client, kw: &str, page: usize) -> Result<Vec<Book>, String> {
    with_retry(|| api.search_books(kw, page), 3)
}
```

```rust
/// 带进度条的高阶函数
///
/// 将闭包应用到 items 的每个元素上，显示进度条。
/// closure 签名: `Fn(&T) -> Result<U, String>`
pub fn with_progress<T, U>(items: &[T], f: impl Fn(&T) -> Result<U, String>,
                           label: &str) -> Vec<Result<U, String>> { ... }
```

### 4.5 导入顺序

```rust
// 标准库
use std::fs;
use std::path::{Path, PathBuf};

// 外部 crate
use indicatif::{ProgressBar, ProgressStyle};

// 内部模块
use crate::api::Client;
use crate::types::Chapter;
```

---

## 5. 接口设计

### 5.1 基本函数 → `function` 子命令映射

每个基本函数在 `FunctionCmd` 枚举中有一个对应变体：

```rust
enum FunctionCmd {
    Search { keyword: String, page: Option<usize> },
    FetchCatalog { book_id: String },
    FetchContent { item_id: String },
    FetchAudioUrl { item_id: String, tone: usize },
    Compress { input: String, abr: u32 },
    EmbedLrc { input: String },
    EmbedCover { input: String, cover: String },
    StripHtml { input: String },
    ReadInfo { dir: String },
    GenLrc { chapter_json: String },
}
```

### 5.2 工作流参数设计

每个工作流函数接收 `WorkflowParams` 结构体而不是散落的参数：

```rust
struct DownloadParams {
    book_id: String, output: PathBuf, range: Option<String>,
    format: String, concurrent: usize, force: bool,
    audio: bool, tone: usize, abr: u32, lrc: String,
}

fn workflow_download(api: &Client, p: DownloadParams, cfg: &Config) { ... }
```

---

## 6. API 探索指南

> ⚠️ **重要**: 以下 API 信息是截至 v0.4.0 已知的。**你应该自行探索是否还有新的接口可用**。番茄小说（字节跳动）的 API 经常变化，第三方 API 服务器也可能增减。

### 6.1 已知 API

| 用途 | URL | 来源 | 状态 |
|------|-----|------|------|
| 搜索 | `novel.snssdk.com/api/novel/channel/homepage/search/search/v1/?aid=1967&q={}&offset={}` | 官方 | ✅ 稳定 |
| 搜索备用 | `101.35.133.34:5000/api/search?key={}&offset={}` | 第三方 | ✅ 可靠 |
| 目录 | `fanqienovel.com/api/reader/directory/detail?bookId={}` | 官方 | ✅ 稳定 |
| 正文 | `101.35.133.34:5000/api/content?tab=小说&item_id={}` | 第三方 | ✅ 首选 |
| 正文备用 | `tt.sjmyzq.cn/api/raw_full?item_id={}` | 第三方 | ⚠️ 偶有5xx |
| 正文备用2 | `101.35.133.34:5000/api/raw_full?item_id={}` | 第三方 | ✅ 可靠 |
| 音频 URL | `101.35.133.34:5000/api/content?tab=听书&item_id={}&tone_id={}` | 第三方 | ✅ 稳定 |
| 音频文档 | `101.35.133.34:5000/docs` | 第三方 | ✅ 有完整文档 |

### 6.2 探索方向

探索以下可能性：
1. **官方 WEB 端 API** — 检查 `fanqienovel.com` 的 Network 请求，寻找未文档化的 JSON 接口
2. **第三方服务器 `/docs`** — `101.35.133.34:5000/docs` 可能有新的端点
3. **`tt.sjmyzq.cn`** — 尝试其 `/api/` 下其他路径
4. **章节推荐 API** — 有些服务器提供相关书籍推荐
5. **书籍详情 API** — 获取封面图 URL、分类、标签等元数据
6. **用户书架 API** — 官方可能提供用户收藏列表接口
7. **批量内容 API** — 一次请求返回多章内容，减少 HTTP 开销

探索时注意：
- 使用 `curl -v` 查看完整响应头
- 检查 `Set-Cookie` 看是否需要鉴权
- 响应 JSON 的结构可能有新字段
- 第三方 API 可能有使用限制/频率控制

### 6.3 音频音色

已知 Book 7185478854738185271 可用音色：
```
1, 2, 4, 5, 6, 74, 91
```

**探索**: 这是否是所有书籍通用的？尝试用 `tone_id=100`+ 看是否返回更多。检查服务器是否有 `/api/voices` 或 `/api/tones` 端点。

---

## 7. 当前状态

### 7.1 已完成

| 模块 | 功能 | 状态 |
|------|------|------|
| **API 层** | 搜索/目录/正文/音频 API 调用 + URL 轮换 | ✅ |
|  | HTTP 三模式（auto/minreq/curl） | ✅ |
|  | 文件缓存（TTL 控制） | ✅ |
|  | HTML 标签剥离 | ✅ |
| **正文下载** | 并发下载（线程池 + round-robin） | ✅ |
|  | EPUB 生成（zip 库，零系统依赖） | ✅ |
|  | info.list 读写 | ✅ |
|  | 目录模式增量更新 | ✅ |
|  | `--force` 覆盖控制 | ✅ |
| **音频下载** | 并发音频下载 + 音色回退 | ✅ |
|  | ABR 流式压缩（lame pipe） | ✅ |
|  | 后处理命令模板 | ✅ |
|  | LRC 生成 + ID3v2 USLT 嵌入 | ✅ |
|  | 封面嵌入（APIC, jpg/png） | ✅ |
|  | Audio/info.list 读写 | ✅ |
| **TTS** | edge-tts 文件/目录转换 | ✅ |
|  | 参数化（rate/volume/pitch） | ✅ |
| **CLI** | 6 个子命令 + function 子命令 | ✅ |
|  | 简体中文帮助 | ✅ |
|  | 彩色终端 + 进度条 | ✅ |
| **配置** | INI 解析 + 书架 | ✅ |
| **CI** | GitHub Actions cross 编译 | ✅ |

### 7.2 待办（按优先级）

1. **⚡ 代码拆分** — 将当前 `download()`、`update()`、`audio_dl()` 等大函数拆成基本函数
2. **⚡ 高阶函数提取** — 实现 `with_retry`、`with_cache`、`with_progress`、`with_verbose`、`with_filter`、`with_output`
3. **⚡ workflow 模块** — 创建 `src/workflow/`，将预编译工作流从 main.rs 移出
4. **📦 function 子命令扩展** — 为每个基本函数添加 `FunctionCmd` 变体
5. **📦 配置驱动扩展** — 实现 `[function]`、`[workflow]`、`[workflow_cmd]` 节
6. **📦 高阶 verbose** — 用 `with_verbose` 替换所有 `if verbose { ... }` 散落代码
7. **🔍 API 探索** — 探索是否有新的官方/第三方 API 端点
8. **💡 智能 `--force`** — 内容哈希比对，相同则跳过
9. **💡 Rust 原生 lame** — 用 `lame-rs` 消除外部依赖
10. **🧹 clippy 清理** — 解决 31 个 warnings

11. [x] macOS 适配：原生配置/缓存目录、兼容已有配置、Apple Silicon/Intel 构建与安装。
12. [x] 修复发布附件与下载脚本名称不一致；下载失败不得覆盖已有程序。
13. [x] 修复 init 忽略错误仍报告成功；添加离线环境诊断与回归验证。
14. [x] 音频 macOS 适配：限制 grun 为 Linux 回退，修复长文本 TTS 的命令行长度限制。
15. [ ] 已有音频后处理 speed/normalize 参数尚未实现；本次保留接口，后续单独修复。
16. [ ] 已有部分工作流遇到失败仍以退出码 0 结束；后续统一错误传播。
17. [ ] 已有正文下载检查文件名时遗漏 .txt 后缀，且部分并发 EPUB/元数据错误路径需另行修复。
18. [ ] 已有 API 错误文本按字节截断可能在 Unicode 边界 panic；正文空响应也可能误判为成功，需单独修复。

### 7.3 已知问题

- 第三方 API 不稳定（偶发 5xx）
- edge-tts 需要 Python 环境
- 编译只能用 rustls（Termux 无 libssl-dev）
- 本地 `cargo build` 为 debug 模式；macOS 使用 `cargo build --release`，Linux 交叉编译使用 cross。

---

## 8. 工作规范

### 8.1 To-Do 管理

- **永远保留原有待办项**，新需求追加到末尾
- 用户提出需求 → **先更新 AGENTS.md 待办** → 再实现代码
- 待办状态标记：`[ ]` 未开始 / `[x]` 已完成 / `[-]` 进行中
- 每完成一项，在 `### 7.2 待办` 对应项后加 `✅`

### 8.2 版本管理

- 每次发布前更新 `Cargo.toml` 的 `version`
- 每次 AGENTS.md 变更时更新开头的版本号
- 版本格式：`vMAJOR.MINOR.PATCH`
- PATCH：bug 修复 / 重构（不新加功能）
- MINOR：新功能（向后兼容）
- MAJOR：破坏性变更

### 8.3 交互规范

- **全部使用简体中文**（注释、帮助、错误消息、此文件）
- 错误前缀 `  err `，成功前缀 `  ok `（2空格缩进）
- 不使用 emoji
- 保持输出简洁对齐
- 对用户直接给结论，不解释推理过程

### 8.4 代码原则

- 每个基本函数 ≤30 行
- 每个工作流 ≤60 行
- 高阶函数 ≤20 行
- 不重复造轮子（优先用已有的库）
- 不引入无依赖的代码（每次加 crate 要评估必要性）
- CLI 参数短名遵循惯例：`-o` `-r` `-v` `-j` `-h` `-f` `-e`

### 8.5 持续审视（Crucial）

每次改动前和改动后，必须主动检查以下方面，并将发现的问题写入待办：

**🐛 Bug 检查清单：**
- 边界条件：空输入、超大输入、负值、特殊字符（Unicode/emoji/shell 元字符）
- 错误路径：网络超时、API 返回异常、文件权限、磁盘满、缓存损坏
- 并发安全：`AtomicUsize` 是否正确使用、`thread::spawn` 闭包是否捕获了 `&self`
- 资源泄漏：`File` 是否及时关闭、临时文件是否清理、`thread::join` 是否处理 panic
- 状态不一致：info.list 和实际文件列表不匹配、音频和 LRC 文件不对应

**⚡ 优化检查清单：**
- 重复请求：连续多次调用同一 API？（加缓存或用局部变量复用结果）
- 冗余 I/O：每次循环都 `fs::metadata`？可以批量读取
- 内存：大文件是否全部读入内存？（用 stream）
- 锁/竞争：`Arc<AtomicUsize>` 是否可以用更轻量的方式（如 `crossbeam`）
- 冷热分离：频繁访问的数据是否在 hot path 上？（如 `Config::load()` 每次调用都读文件？）

**💡 现代化设计检查清单：**
- 2018/2021 edition 特性是否充分利用？（`impl Trait`、`async/await`、`let-else`、`matches!` 等）
- 能否用 `trait` 抽象出通用行为？（如 `Downloader` + `AudioDownloader` 共享的并发逻辑）
- `enum` + `match` 是否优于多重 `if-else`？
- 能否用 `Result` + `?` 替代散落的 `unwrap()`/`expect()`？
- 函数签名是否过于复杂？（考虑用 `struct` 折叠参数）
- 有无死代码？（`pub` 但现在无人调用的函数）
- 依赖是否过时？（检查 `cargo outdated`）
- 能否用 `clap` 的 `value_parser` 做更严格的参数校验？

**发现任何上述问题 → 立即更新 7.2 待办 → 再实施修复。**

---

## 9. 外部依赖总览

| 依赖 | 用途 | 必须？ | 替代方案 |
|------|------|--------|---------|
| `curl` (外部命令) | HTTP 下载 fallback | 否 | minreq |
| `minreq` (crate) | HTTP 库（rustls-tls） | 是 | reqwest |
| `lame` (外部命令) | MP3 压缩 | 否 | lame-rs（未来） |
| `edge-tts` (Python) | TTS 语音合成 | 否 | 官方语音 API |
| `indicatif` | 进度条 | 是 | — |
| `id3` | ID3v2 tag 读写 | 是 | mp3-metadata |
| `zip` | EPUB 生成 | 是 | — |
| `clap` | CLI 参数解析 | 是 | — |
| `serde_json` | JSON 解析 | 是 | — |
| `cross` (CI tool) | 交叉编译 | CI only | — |

---

## 10. 文件清单

```
/root/build/fqdt/
├── AGENTS.md              ← 本文件（AI agent 主文档）
├── Cargo.toml             ← 版本号 + 依赖
├── Cargo.lock
├── .github/workflows/
│   └── build.yml          ← CI: cross aarch64-unknown-linux-gnu
└── src/
    ├── main.rs            ← CLI + 分发 + 配置注入（待拆分）
    ├── api.rs             ← HTTP 客户端 + API 调用
    ├── audio.rs           ← 音频/TTS/LRC/封面/后处理
    ├── download.rs        ← 正文下载 + info.list
    ├── config.rs          ← 配置解析 + 书架
    ├── types.rs           ← 数据结构
    └── epub.rs            ← EPUB 生成
```
