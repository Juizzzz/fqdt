# fqdt — 番茄小说下载器

## 安装

```sh
# 从 GitHub Releases 下载
curl -L https://github.com/addallno/fqdt/releases/download/v0.5.1/fqdt-aarch64-linux -o ~/fqdt
chmod +x ~/fqdt

# 初始化配置
~/fqdt init
```

## 用法一览

| 命令 | 简写 | 功能 |
|------|------|------|
| `search` | | 搜索 + 交互下载 |
| `info` | `i` | 查看目录/内容 |
| `get` | `g` | 统一下载：正文/音频/增量/TTS |
| `shelf` | `s` | 书架管理 |
| `function` | `fn` | 原子函数 + 管道 |
| `init` | | 生成默认配置 |
| `custom` | | 可在 `config.ini` 注册自定义命令 |

---

## get — 统一下载命令（替代 download/update/audio）

```sh
# 下载正文（默认）
fqdt get <book_id>

# 简写 g
fqdt g <book_id>

# 指定输出目录 + 格式 + 并发
fqdt get <book_id> -o ./books -t epub -j 8

# 章节范围
fqdt get <book_id> -r 1-100
fqdt get <book_id> -r=-10

# 下载正文 + 音频
fqdt get <book_id> --audio --tone 5

# 仅下载音频
fqdt get <book_id> --audio-only --abr 32

# MP3 压缩 + 歌词
fqdt get <book_id> --audio-only --abr 32 --lrc external

# 增量更新
fqdt get <book_id> --update
fqdt get ./output           # 目录自动增量模式
fqdt get ./output --audio   # 目录+音频增量

# TTS 转换文本为语音
fqdt get --tts novel.txt
fqdt get --tts novel_dir/ --voice zh-CN-XiaoxiaoNeural
fqdt get --tts novel.txt --rate +20% --normalize

# 强制覆盖
fqdt get <book_id> -f
```

## search — 搜索并下载

```sh
# 搜索并交互选择
fqdt search 凡人修仙传

# 自动下载第 1 本
fqdt search 凡人 -D 1

# 第 2 页第 3 本
fqdt search 凡人 -p 2 -D 3

# 仅搜索显示翻页提示
fqdt search 凡人 --dry-run
```

## info — 查看目录（简写 i）

```sh
# 查看全部目录（自动显示书名/作者/简介）
fqdt info <book_id>
fqdt i <book_id>

# 指定章节范围 + 显示正文
fqdt info <book_id> -r 1-5 -s
```

## shelf — 书架管理（简写 s）

```sh
fqdt s               # 列出
fqdt shelf -a <ID>:<标题>  # 添加
fqdt shelf -d <编号>       # 删除
fqdt shelf -D <编号>       # 下载
```

## function — 原子函数 + 管道（简写 fn）

```sh
# 基本函数
fqdt fn fetch-catalog <book_id>
fqdt fn fetch-content <item_id>
fqdt fn fetch-detail <book_id>
fqdt fn search "凡人"
fqdt fn strip-html page.html
fqdt fn compress audio.mp3 --abr 32
fqdt fn embed-lrc audio.mp3
fqdt fn embed-cover audio.mp3 cover.jpg

# 管道组合 (\; = 分步骤)
fqdt fn embed input.mp3 --lrc \; process {} --abr 32
fqdt fn fetch-catalog <book_id> \; fetch-content {}
fqdt fn fetch-catalog <book_id> \; fetch-content-batch <book_id> {}
fqdt fn search "凡人" \; fetch-detail {}
```

## 自定义命令

在 `~/.config/fqdt/config.ini` 添加：

```ini
[workflow_cmd]
# 格式: 命令名 = shell 命令模板, {} 会被参数替换
echo = echo 你输入了: {}
decode = python3 ~/decode_woff.py {}
```

使用：

```sh
fqdt echo 你好世界   # → "你输入了: 你好世界"
fqdt decode font.woff
```

## 配置文件

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
# ... URL 配置 ...

[workflow_cmd]
# 自定义命令
```

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

## 编译

```sh
cargo build --release
```
