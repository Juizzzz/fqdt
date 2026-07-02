# fqdt — 番茄小说下载器

## 安装

```sh
# 从 GitHub Releases 下载
curl -L https://github.com/addallno/fqdt/releases/download/v0.5.0/fqdt-aarch64-linux -o ~/fqdt
chmod +x ~/fqdt

# 初始化配置
~/fqdt init
```

## 用法一览

| 命令 | 功能 |
|------|------|
| `search` | 搜索 + 交互下载 |
| `info` | 查看目录/章节 |
| `download` | 下载正文 |
| `update` | 增量更新正文 |
| `audio` | 下载音频/TTS |
| `shelf` | 书架管理 |
| `function` | 原子函数调用 |
| `init` | 生成默认配置 |

---

## search — 搜索并下载

```sh
# 搜索并交互选择
fqdt search 凡人修仙传

# 自动下载第 1 本
fqdt search 凡人 -D 1

# 第 2 页
fqdt search 凡人 -p 2

# 第 2 页第 3 本自动下载
fqdt search 凡人 -p 2 -D 3

# 指定输出目录 + 并发
fqdt search 凡人 -D 1 -o ./books -j 8

# 仅搜索不下载
fqdt search 凡人 --dry-run
```

## info — 查看目录

```sh
# 查看全部目录
fqdt info <book_id>

# 指定章节范围
fqdt info <book_id> -r 1-50
fqdt info <book_id> -r=-5      # 前 5 章
fqdt info <book_id> -r 10-     # 第 10 章到结尾
```

## download — 下载正文

```sh
# 基本下载
fqdt download <book_id> -o ./output

# 章节范围
fqdt download <book_id> -r 1-100
fqdt download <book_id> -r=-10
fqdt download <book_id> -r 50-

# 并发
fqdt download <book_id> -j 8

# 指定格式
fqdt download <book_id> -t epub
fqdt download <book_id> -t txt

# 强制覆盖已有文件
fqdt download <book_id> --force

# 下载正文 + 音频
fqdt download <book_id> --audio -r 1-10

# 从目录下载 (已有 info.list)
fqdt download ./output
fqdt download ./output -r 1-10
```

## update — 增量更新

```sh
# 更新正文 (只下载新章节)
fqdt update <book_id> -o ./output

# 从目录更新
fqdt update ./output

# 更新音频
fqdt update <book_id> -o ./output --audio
```

## audio — 音频下载 / TTS

```sh
# 下载第三方音频
fqdt audio <book_id> -o ./output/Audio -r 1-50 -c 6

# 指定音色 (1-91)
fqdt audio <book_id> --tone 5

# 音色回退列表
fqdt audio <book_id> --tone-fallbacks 2,4,5,6,74,91

# MP3 压缩
fqdt audio <book_id> --abr 32

# LRC 歌词
fqdt audio <book_id> --lrc zh

# 后处理命令模板
fqdt audio <book_id> --post-process "lame --abr 32 {input} {output}"

# 从目录增量下载音频
fqdt audio ./output/Audio

# TTS 转换
fqdt audio --tts ./novel.txt
fqdt audio --tts ./novel_dir/ -r 1-50
fqdt audio --tts ./novel.txt --voice zh-CN-XiaoxiaoNeural
fqdt audio --tts ./novel.txt --rate +20% --volume +50%
```

## shelf — 书架

```sh
# 列出书架
fqdt shelf

# 添加
fqdt shelf -a <book_id>:<书名>

# 删除 (按编号)
fqdt shelf -d 2

# 从书架下载
fqdt shelf -D 1
```

## function — 原子函数 (管道组合)

```sh
# 基本函数调用
fqdt function fetch-catalog <book_id>
fqdt function fetch-content <item_id>
fqdt function fetch-audio-url <item_id> 5
fqdt function search "凡人"
fqdt function strip-html page.html
fqdt function compress audio.mp3 --abr 32
fqdt function embed-lrc audio.mp3
fqdt function embed-cover audio.mp3 cover.jpg
fqdt function read-info ./output
fqdt function read-audio-info ./output/Audio
fqdt function fetch-detail <book_id>
```

### 管道组合

```sh
# embed → process (嵌入 LRC 后压缩)
fqdt function embed input.mp3 --lrc \; process {} --abr 32

# fetch-catalog → fetch-content (获取第一章内容)
fqdt function fetch-catalog <book_id> \; fetch-content {}

# fetch-catalog → fetch-content-batch (批量获取多章)
fqdt function fetch-catalog <book_id> \; fetch-content-batch <book_id> {}

# 三步: 获取目录 → 读取内容 → 剥离 HTML
fqdt function fetch-catalog <book_id> \; fetch-content {} \; strip-html -

# audio-info → embed-lrc (读信息后嵌入歌词)
fqdt function read-audio-info ./output/Audio \; embed-lrc {}.mp3

# search → fetch-detail (搜索后查详情)
fqdt function search "凡人" \; fetch-detail {}

# fetch-detail 管道配对
fqdt function fetch-detail <book_id> \; fetch-content {} \; strip-html -
```

管道规则:
- `\;` 分隔多步 (shell 转义分号)
- `{}` 被上一步输出替换
- 上一步失败则管道终止

## 组合命令

```sh
# search + download 一条命令
fqdt search 凡人 -D 1 -o ./books -j 8 -t epub

# search + download (带音频)
fqdt search 凡人 -D 1 --audio --abr 32

# download + audio 分两步但用同一目录
fqdt download <book_id> -o ./book
fqdt audio <book_id> -o ./book/Audio -r 1-100

# 先更新正文再更新音频
fqdt update <book_id> -o ./book
fqdt update <book_id> -o ./book --audio
```

## 配置

```sh
fqdt init
```

编辑 `~/.config/fqdt/config.ini`:

```ini
[download]
concurrent = 4
format = txt
output_dir = .
filename_template = {idx04}_{title}

[cache]
cache_enabled = true
cache_ttl = 86400

[api]
search_url = https://novel.snssdk.com/api/novel/channel/homepage/search/search/v1/?aid=1967&q={}&offset={},http://101.35.133.34:5000/api/search?key={}&offset={}
catalog_url = https://fanqienovel.com/api/reader/directory/detail?bookId={}
content_url = http://101.35.133.34:5000/api/content?tab=小说&item_id={},https://tt.sjmyzq.cn/api/raw_full?item_id={}
batch_url = http://101.35.133.34:5000/api/content?tab=批量&book_id={}&item_ids={}
detail_url = http://101.35.133.34:5000/api/detail?book_id={}
audio_content_url = http://101.35.133.34:5000/api/content?tab=听书&item_id={}&tone_id={}
audio_tone = 1
audio_tone_fallbacks = 2,4,5,6,74,91

[http]
http_method = auto
curl_args =

[tts]
tts_rate = +0%
tts_volume = +0%
tts_pitch = +0Hz

[audio]
abr = 0
post_process =
```

## 输出格式

### 文件名模板

| 模板 | 示例 |
|------|------|
| `{idx}` | `1` |
| `{idx04}` | `0001` |
| `{title}` | `第一章 穿越` |

### 章节范围

| 写法 | 含义 |
|------|------|
| `1-50` | 第 1 到 50 章 |
| `-5` | 前 5 章 |
| `10-` | 第 10 章到结尾 |
| `=-5` | 前 5 章 (带 `=` 避免 shell 解析) |

## 文件结构

```
output/
├── info.list              # 正文元数据
├── 0001_第一章_穿越.txt   # 正文文件
├── 0002_第二章_奇遇.txt
├── ...
├── 书名.epub              # (如果 -t epub)
└── Audio/
    ├── info.list           # 音频元数据
    ├── 0001_第一章_穿越.mp3
    ├── 0001_第一章_穿越.lrc
    └── ...
```

## 编译

```sh
cargo build --release
```
