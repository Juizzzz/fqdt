# fqdt — 番茄小说下载器

## macOS 版 v0.6.0

本分支保留原有命令行功能，新增 Apple Silicon（M 系列）和 Intel Mac 构建。
这是原生终端程序，不是图形界面 App。建议使用 macOS 11 或更新版本。
第三方小说接口和在线语音服务能否访问，仍取决于服务本身；安装成功不代表接口一定可用。

## 安装

### 使用交付的 Mac 通用安装包（无需 Rust）

解压 `fqdt-macos-universal.tar.gz`，进入解压目录，运行：

```sh
sh install.sh
"$HOME/.local/bin/fqdt" doctor
"$HOME/.local/bin/fqdt" init
```

通用程序包含 arm64 和 x86_64 两种架构。安装器只复制程序，不更改 shell 配置、不需要 sudo。
若要直接输入 `fqdt`，可自行在 `~/.zshrc` 中加入 `export PATH="$HOME/.local/bin:$PATH"`，然后重新打开终端。

### 从本地源码安装

先安装 Apple 命令行工具和 Rust（已安装时跳过）。运行第一条后等待系统安装完成：

```sh
xcode-select --install
```

Rust 官方安装方式：

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

在**这份已修改源码**目录运行（重新克隆上游不会得到本次修改）：

```sh
sh scripts/install-macos.sh
"$HOME/.local/bin/fqdt" doctor
"$HOME/.local/bin/fqdt" init
```

`FQDT_INSTALL_DIR` 可以覆盖安装目录。安装器根据当前终端架构构建；Apple 芯片请优先使用原生 arm64 终端。
`CARGO` 可指定 cargo 可执行文件路径。

### 从 GitHub Release 下载（维护者发布本分支之后）

本次交付没有向上游推送代码或创建 GitHub Release，不能假定上游存在 macOS 下载文件。
维护者发布本分支 tag 后，可指定对应仓库和版本下载：

```sh
FQDT_REPO=你的账号/fqdt FQDT_VERSION=v0.6.0 sh dl.sh ./fqdt
./fqdt doctor
./fqdt init
```

下载器自动匹配系统/架构，并验证 SHA-256 和程序文件格式；HTTP 错误或校验失败会保留已有程序。
发布附件为 `fqdt-aarch64-apple-darwin`、`fqdt-x86_64-apple-darwin`、`fqdt-aarch64-linux` 及各自的 `.sha256`。
Linux 原有构建仍保留；不要在 Mac 上运行 Linux 附件。

## 配置和数据目录

- 新 macOS 安装：`~/Library/Application Support/fqdt/config.ini`，书架为同目录的 `books.txt`。
- macOS 缓存：`~/Library/Caches/fqdt/`。
- 若 `~/.config/fqdt/` 已有 `config.ini` 或 `books.txt`，继续使用该旧目录，不移动或覆盖原数据。
- Linux 继续使用 `~/.config/fqdt/`。
- `FQDT_CONFIG_DIR`、`FQDT_CACHE_DIR` 可分别指定目录，测试使用独立目录以免影响用户数据。
- `fqdt doctor` 显示实际路径和可选工具，不联网，也不创建配置。
- 默认下载目录仍是当前工作目录，可用 `-o "$HOME/Downloads/小说"` 指定；含空格的路径必须加引号。

## 可选音频依赖

正文下载和 EPUB 导出不需要额外音频工具。音频下载使用 macOS 自带的 `curl`。
如需 MP3 压缩或在线语音合成，可在已安装 Homebrew 的情况下执行：

```sh
brew install lame pipx
pipx install edge-tts
```

程序会在 PATH、`/opt/homebrew/bin`、`/usr/local/bin`、`~/.local/bin` 查找工具。
长文本通过临时文件交给 edge-tts，避免超出命令行参数长度限制；正常结束或调用失败时清理临时文件。
`fqdt doctor` 可检查工具安装情况。在线服务的成功率不包含在离线测试保证内。

## 用法一览

| 命令 | 别名 | 功能 |
|------|------|------|
| `search` | | 搜索 + 交互下载 |
| `download` | `d` | 下载正文 |
| `audio` | `a` | 下载语音 |
| `update` | `u` | 增量更新 |
| `tts` | `t` | 文本转语音 |
| `info` | `i` | 查看目录/内容 |
| `shelf` | `s` | 书架管理 |
| `function` | `fn` | 原子函数 + 管道 |
| `init` | | 生成默认配置 |
| `doctor` | | 检查平台、路径与可选依赖 |

---

## download — 下载正文（简写 d）

```sh
fqdt download <book_id>
fqdt d <book_id>                    # 简写
fqdt d <book_id> -o ./books -t epub -j 8   # 输出+格式+并发
fqdt d <book_id> -r 1-100                 # 章节范围
fqdt d <book_id> -r=-10                   # 前10章
fqdt d <book_id> -f                       # 强制覆盖
```

## audio — 下载语音（简写 a）

```sh
fqdt audio <book_id>
fqdt a <book_id>                    # 简写
fqdt a <book_id> -r 1-50 -j 6       # 范围+并发
fqdt a <book_id> --tone 5           # 指定音色
fqdt a <book_id> --lrc embed        # 歌词嵌入MP3
```

## update — 增量更新（简写 u）

```sh
fqdt update <book_id>               # 按ID更新
fqdt update ./output                # 按目录自动检测
fqdt u ./output -j 4                # 指定并发
```

## tts — 文本转语音（简写 t）

```sh
fqdt tts novel.txt                  # 单文件
fqdt tts novel_dir/                 # 整个目录
fqdt tts file.txt --voice zh-CN-XiaoxiaoNeural
```

## search — 搜索并下载

```sh
fqdt search 凡人修仙传               # 搜索并交互选择
fqdt search 凡人 -D 1               # 自动下载第1本
fqdt search 凡人 -p 2 -D 3          # 第2页第3本
fqdt search 凡人 --dry-run          # 仅搜索
```

## info — 查看目录（简写 i）

```sh
fqdt info <book_id>
fqdt i <book_id>                    # 简写
fqdt i <book_id> -r 1-5 -s          # 章节范围+显示正文
```

## shelf — 书架管理（简写 s）

```sh
fqdt s                              # 列出
fqdt shelf -a <ID>:<标题>          # 添加
fqdt shelf -d <编号>               # 删除
fqdt shelf -D <编号>               # 下载
fqdt shelf -U                       # 一键更新所有
```

## function — 原子函数 + 管道（简写 fn）

```sh
fqdt fn fetch-catalog <book_id>
fqdt fn fetch-content <item_id>
fqdt fn search "凡人"
fqdt fn compress audio.mp3 --abr 32
fqdt fn embed-lrc audio.mp3
fqdt fn save output.txt             # 管道保存到文件

# 管道组合
fqdt fn fetch-catalog <id> \; fetch-content {}
fqdt fn search "凡人" \; fetch-detail {}
fqdt fn ... \; save result.txt      # 保存管道输出
```

## 自定义命令

在 `fqdt doctor` 显示的配置文件中添加：

```ini
[workflow_cmd]
echo = echo 你输入了: {}
decode = python3 ~/decode_woff.py {}
```

使用：

```sh
fqdt echo 你好世界
fqdt decode font.woff
```

## 智能功能

- **智能源切换**: 自动按成功率+延迟排序 API 源，失败 URL 自动降级
- **自动并发**: 根据 CPU 核心自动选择并发数（核心×2，最高 32）
- **哈希增量**: `--force` 时对比已有内容，相同则跳过写入
- **书架一键更新**: `fqdt shelf -U` 更新所有收藏书籍

## 输出结构

```
output/
├── info.list
├── 0001_第一章_穿越.txt
├── 0002_第二章_奇遇.txt
├── 书名.epub
└── Audio/
    ├── info.list
    ├── 0001_第一章_穿越.mp3
    ├── 0001_第一章_穿越.lrc
    └── ...
```

## 编译与验证

```sh
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
python3 tests/test_workflows.py
# 以下下载器测试用于 macOS：
python3 tests/test_installer.py
cargo build --release --locked
```

跨架构构建（需要在 Mac 上执行）：

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --release --locked --target aarch64-apple-darwin
cargo build --release --locked --target x86_64-apple-darwin
sh scripts/package-macos.sh
```

CI 为两种 Mac 架构构建原生程序，并保留 Linux ARM64 构建。Intel 构建成功不等于已在 Intel 实机完成运行验证。

## 已知上游限制

保留了已有功能与配置接口，没有在平台适配中重写下载器：
- `function process` 的 `speed` / `normalize` 参数在上游尚未真正执行音频处理。
- 部分下载、语音工作流失败时打印错误，但进程退出码仍可能是 0；请同时检查输出文件和错误消息。
- 本次修复了 `init` 的失败退出码；第三方 API 可用性不作承诺。
