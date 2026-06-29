# clip

System clipboard bridge for the terminal. Pipe content **in**, paste content **out**. Works over SSH via OSC52.

## Install

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

## License

MIT
