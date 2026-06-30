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
| `nightly.yml` | daily cron | same as release but tagged `nightly` (prerelease, rolling) |

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

```bash
just aur-srcinfo          # regenerate .SRCINFO after editing PKGBUILD
just aur-release VERSION  # update version, commit, push to AUR subtree
```

AUR is maintained as a `git subtree` at remote `aur`.

### Automated publishing

`release.yml` 会在发布版本后自动更新 AUR 包（`aur` job）。流程：

1. 更新 `aur/PKGBUILD` 中的 `pkgver`、重置 `pkgrel=1`
2. 用 Arch Linux Docker 容器运行 `makepkg --printsrcinfo` 生成 `.SRCINFO`
3. `git subtree push` 推送到 `ssh://aur@aur.archlinux.org/clip-cli-bin.git`

**前置条件**：GitHub 仓库需要配置 secret `AUR_SSH_PRIVATE_KEY`（对应已上传到 AUR 账户的公钥）。

## Conventions

- Commit messages: `type: 中文描述` (e.g., `feat: 添加 OSC52 支持`, `fix: 修复 SSH 下粘贴失败`)
- PR branch → `main`; only release tags (`v*`) trigger publishing
- The `justfile` is the authoritative source for dev commands — prefer `just <cmd>` over raw cargo
