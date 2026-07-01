# AGENTS.md — clip

System clipboard bridge for the terminal. Single-binary Rust CLI (~186 LOC in `src/main.rs`).

## Commands (use `just`)

```bash
just check        # cargo check (fast, no binary)
just build        # cargo build (debug)
just release      # cargo build --release
just test         # cargo test (currently 0 tests)
just fmt          # cargo fmt
just fmt-check    # cargo fmt --check
just lint         # cargo clippy
just lint-strict  # cargo clippy -- -D warnings
just fix          # cargo clippy --fix --allow-dirty
just ci           # full gate: fmt-check → lint-strict → test → release build
just install      # cargo install --path . → ~/.cargo/bin/clip
```

Run with args: `just run -- --help` or `just run-release -- --help`.

## Toolchain

- **Rust edition 2024** → requires Rust **≥1.85**
- CI uses `dtolnay/rust-toolchain@stable` with `rustfmt` + `clippy` components
- No custom `rustfmt.toml` or `clippy.toml` — all tool defaults

## CI / Release

| Workflow | Trigger | What it does |
|---|---|---|
| `ci.yml` | push/PR to `main` | `fmt-check`, `lint-strict`, `cargo test` |
| `release.yml` | tag `v*` | calls `build.yml` (all targets), publishes GitHub Release with changelog, auto-updates AUR |
| `nightly.yml` | daily cron | same as release but tagged `nightly` (prerelease, rolling), auto-updates nightly AUR |

### Build matrix (release)

| Target | OS | Note |
|---|---|---|
| `x86_64-unknown-linux-gnu` | ubuntu-latest | native |
| `x86_64-unknown-linux-musl` | ubuntu-latest | uses `cross` tool, not cargo |
| `aarch64-unknown-linux-gnu` | ubuntu-24.04-arm | native |
| `x86_64-apple-darwin` | macos-latest | native |
| `aarch64-apple-darwin` | macos-latest | native |
| `x86_64-pc-windows-msvc` | windows-latest | native |

Artifact naming: `clip-{target}.exe` on Windows, `clip-{target}` elsewhere.

## Architecture

Single file `src/main.rs`:

- **`main()`**: auto-detect mode — if stdin is a TTY → read clipboard; otherwise → store + tee to stdout
- **`store()`**: read stdin → strip trailing `\n` → write system clipboard (local) → always write file cache → OSC52 if SSH
- **`read()`**: try system clipboard (local) → fallback to file cache → stdout
- **`cache_path()`**: `$XDG_CACHE_HOME/clipboard/data` or `~/.cache/clipboard/data`
- **`is_ssh()`**: checks `SSH_TTY`, `SSH_CLIENT`, `SSH_CONNECTION`
- **`write_osc52()`**: base64-encode content → `ESC]52;c;<base64>BEL` → write to `/dev/tty` (fallback: stderr)

Dependencies: `arboard` (system clipboard), `base64`, `clap` (derive), `is-terminal`.

## AUR

项目维护两个 AUR 包：

| 包名 | 子目录 | 远程 remote | 发布触发 |
|---|---|---|---|
| `clip-cli-bin` (稳定版) | `aur/` | `aur` → `clip-cli-bin.git` | `release.yml` (tag `v*`) |
| `clip-cli-nightly-bin` (每夜构建) | `aur-nightly/` | `aur-nightly` → `clip-cli-nightly-bin.git` | `nightly.yml` (每日 cron) |

两者通过 `git subtree` 维护，共享同一个 AUR 账户的 SSH key。

```bash
# 稳定版
just aur-srcinfo                # 重新生成 aur/.SRCINFO
just aur-release VERSION        # 更新版本号 + 推送

# 每夜构建
just aur-nightly-srcinfo        # 重新生成 aur-nightly/.SRCINFO
just aur-nightly-release DATE   # 更新日期版本号（YYYYMMDD）+ 推送
```

### Automated publishing

`release.yml` 和 `nightly.yml` 各自的 `aur` job 会在发布后自动更新对应 AUR 包：

1. 更新 `PKGBUILD` 中的 `pkgver`（稳定版用 tag 版本号，nightly 用当前日期 `YYYYMMDD`）、重置 `pkgrel=1`
2. 用 Arch Linux Docker 容器运行 `makepkg --printsrcinfo` 生成 `.SRCINFO`
3. `git subtree push` 推送到对应 AUR 仓库

**前置条件**：GitHub 仓库需要配置 secret `AUR_SSH_PRIVATE_KEY`（对应已上传到 AUR 账户的公钥）。

## Conventions

- Commit messages: `type: 中文描述` (e.g., `feat: 添加 OSC52 支持`, `fix: 修复 SSH 下粘贴失败`)
- PR branch → `main`; only release tags (`v*`) trigger publishing
- The `justfile` is the authoritative source for dev commands — prefer `just <cmd>` over raw cargo
