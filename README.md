# clip <sup>v1.1.0</sup>

System clipboard bridge for the terminal. Pipe content **in**, paste content **out**. Works over SSH via OSC52.

## Install

### Pre-built binaries

Download from [GitHub Releases](https://github.com/someoneonsmile/clip/releases) (latest: `stable` rolling tag).

| Platform | Asset |
|----------|-------|
| Linux x86_64 (glibc) | `clip-x86_64-linux-gnu` |
| Linux x86_64 (musl, static) | `clip-x86_64-linux-musl` |
| Linux ARM64 | `clip-aarch64-linux-gnu` |
| macOS Intel | `clip-x86_64-macos` |
| macOS Apple Silicon | `clip-aarch64-macos` |
| Windows x86_64 | `clip-x86_64-windows.exe` |

```bash
# Linux / macOS example
curl -L -o /usr/local/bin/clip \
  https://github.com/someoneonsmile/clip/releases/latest/download/clip-x86_64-linux-gnu
chmod +x /usr/local/bin/clip
```

### From source

```bash
cargo install --path .
```

Requires Rust toolchain (1.85+ for edition 2024).

## Usage

```bash
# Auto-detect mode (no subcommand)
echo "hello" | clip      # copy stdin to clipboard
clip                     # paste clipboard to stdout

# Explicit subcommands
echo "hello" | clip copy # same as piped auto-detect
clip paste               # same as terminal auto-detect

# Pipe output
clip paste | wc -c       # count clipboard characters
clip paste | xargs ...    # use clipboard as argument
```

### CLI reference

```
$ clip --help
System clipboard bridge — copy stdin to clipboard, paste clipboard to stdout.

When no subcommand is given, mode is auto-detected:
  piped input → copy,  terminal → paste.

Usage: clip [COMMAND]

Commands:
  copy   Copy stdin content to clipboard
  paste  Paste clipboard content to stdout
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Each subcommand also accepts `--help` for detailed usage.

### SSH support

`clip` detects SSH sessions (`SSH_TTY`, `SSH_CLIENT`, `SSH_CONNECTION`) and routes clipboard operations through:

- **File cache** at `~/.cache/clipboard/data` (or `$XDG_CACHE_HOME/clipboard/data`)
- **OSC52** escape sequences — copies content to your local terminal's clipboard when the terminal emulator supports it (iTerm2, Kitty, WezTerm, Alacritty, Windows Terminal, and others)

No configuration needed. Just use `clip` the same way on local and remote machines.

### Exit codes

| Code | Meaning |
|------|---------|
| 0    | Success |
| 1    | Stdin read failed, or clipboard empty / cache not found |

## How it works

### Copy (`echo "foo" | clip`)

```
stdin → system clipboard (local)
     → file cache (~/.cache/clipboard/data)
     → OSC52 escape seq (SSH only, forwarded to terminal)
```

### Paste (`clip`)

```
system clipboard (local, preferred)
  ↓ fallback
file cache (~/.cache/clipboard/data)
  ↓
stdout
```

## Development

```bash
just check       # 类型检查（cargo check）
just build       # 编译 debug 版本
just release     # 编译 release 版本
just run         # 运行 debug 版本
just test        # 运行测试
just fmt         # 格式化代码
just lint        # Clippy lint
just lint-strict # Clippy 严格模式（warnings as errors）
just fix         # Clippy 自动修复
just install     # 安装到 ~/.cargo/bin/
just watch       # 监听文件变更自动 check
just ci          # 全量检查（fmt → clippy → test → release）
```

## License

MIT
