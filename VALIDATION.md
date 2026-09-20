# macOS 适配验证记录

版本：本地 v0.6.0；基于上游提交 `10f71fd`（Cargo 版本 0.5.1）。
环境：macOS arm64，Rust 1.98.1；不新增生产依赖，Cargo.lock 仅更新本项目版本。

## 已完成的验证

| 验证 | 结果 |
|---|---|
| 原项目测试基线 | 4 项通过 |
| `cargo test --locked` | 6 项单元测试 + 5 项 CLI 集成测试通过 |
| `cargo clippy --locked --all-targets -- -D warnings` | 通过，无警告 |
| `python3 tests/test_installer.py` | 6 项通过：HTTP 失败、缺校验、校验错误、错误页面、架构不符、正常安装 |
| `python3 tests/test_workflows.py` | 3 项通过：本地接口 TXT/EPUB、长文本 TTS 调用、TTS 失败清理 |
| `cargo build --release --locked --target aarch64-apple-darwin` | 通过 |
| `cargo build --release --locked --target x86_64-apple-darwin` | 通过 |
| `lipo` 通用程序 | 包含 arm64、x86_64，两种 Mach-O 架构 |
| 通用程序 `--version`、`doctor` | 在本机成功运行，版本 0.6.0 |
| 使用通用程序重新执行本地工作流测试 | 3 项再次通过 |
| 源码安装器及预编译安装器 | 均安装到含空格的隔离测试目录，成功 |
| 所有 shell 脚本 `sh -n` | 通过 |

最终测试集合共 20 项，不将基线与重复执行计入总数。
验证用配置、缓存、工具链和安装目标均隔离，没有覆盖用户已有程序或配置。

## 修改的文件

- `src/platform.rs`：新增 macOS 目录、旧配置兼容、工具发现、临时文本文件和离线诊断。
- `src/config.rs`、`src/types.rs`：配置、书架和缓存统一使用平台路径。
- `src/main.rs`：新增 doctor；初始化配置失败时明确报错、返回非零退出码。
- `src/audio.rs`：Mac 工具路径识别；grun 仅保留在 Linux；长文本使用文件传给 edge-tts；支持负语速/音调参数。
- `dl.sh`：按平台下载，修复附件名称，校验内容，失败保留原文件。
- `scripts/install-macos.sh`：本地源码构建并安装。
- `scripts/install-binary-macos.sh`：免编译安装脚本，随包分发为 install.sh。
- `scripts/package-macos.sh`：生成 Mac 通用安装目录及校验文件。
- `.github/workflows/build.yml`：增加两种 Mac 架构，保留 Linux，按实际附件名发布。
- `tests/cli.rs`、`tests/test_installer.py`、`tests/test_workflows.py`：CLI、安装失败和离线工作流回归测试。
- `Cargo.toml`、`Cargo.lock`：本地适配版本更新到 0.6.0。
- `README.md`、`STATUS.md`、`AGENTS.md`、本文件：使用说明、适配状态及验证记录。

## 验证范围

- Apple 芯片版本和通用程序已在本机运行；Intel 版本已完成交叉编译，未在 Intel 实机运行。
- GitHub Actions 配置已改好，但未推送或在 GitHub 实际执行；Linux 交叉构建尚未在本地重跑。
- 小说接口使用本地模拟服务器测试，不代表上游第三方服务当前可用。
- edge-tts 使用模拟工具验证参数、长文本和清理流程，未调用在线语音服务。本机 doctor 检测到 curl、lame；未安装 edge-tts。
- 保留上游命令和功能；上游已有的音频 speed/normalize 未实现、部分失败退出码和下载元数据等问题已记录在 AGENTS.md，未借平台适配进行大范围重构。

## 下一步

解压通用安装包，运行 `sh install.sh`，然后运行 `~/.local/bin/fqdt doctor` 和 `~/.local/bin/fqdt init`。
使用自有测试书籍做一次单章在线验证，再进行整本下载。
