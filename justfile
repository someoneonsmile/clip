# clip - 系统剪贴板桥接工具
# https://github.com/cargo/just

_default:
    @just --list

# 快速检查（类型检查，不生成二进制）
check:
    cargo check

# 编译 debug 版本
build:
    cargo build

# 编译 release 版本（优化）
release:
    cargo build --release

# 运行（debug）
run *args:
    cargo run -- {{args}}

# 运行（release）
run-release *args:
    cargo run --release -- {{args}}

# 运行测试
test:
    cargo test

# 格式化代码
fmt:
    cargo fmt

# 检查格式（不修改）
fmt-check:
    cargo fmt --check

# Clippy lint
lint:
    cargo clippy

# Clippy lint（严格模式，warning 视为 error）
lint-strict:
    cargo clippy -- -D warnings

# Clippy 自动修复
fix:
    cargo clippy --fix --allow-dirty --allow-staged

# Clippy 自动修复 + 严格 lint 验证
fix-strict: fix lint-strict

# 清理构建产物
clean:
    cargo clean

# 安装到系统（/usr/local/bin/clip）
DESTDIR := ""
PREFIX := "/usr/local"
BINDIR := PREFIX / "bin"

install:
    cargo build --release
    sudo install -Dm755 target/release/clip "{{ DESTDIR }}{{ BINDIR }}/clip"

# 从系统中卸载 clip
uninstall:
    sudo rm -f "{{ DESTDIR }}{{ BINDIR }}/clip"

# 监听文件变更，自动 check
watch:
    cargo watch -x check

# 监听文件变更，自动 fmt + lint + build
watch-all:
    cargo watch -x fmt -x clippy -x build

# 全量检查（fmt → clippy → test → release build）
ci: fmt-check lint-strict test release

# ─── AUR ──────────────────────────────────────────────

# 重新生成 .SRCINFO（手动修改 PKGBUILD 后执行）
aur-srcinfo:
    cd aur && makepkg --printsrcinfo > .SRCINFO

# 提交 aur/ 变更并推送到 AUR
# AUR 禁止 force push，使用 clone → 更新文件 → commit → push 的可靠方式
aur-push:
    #!/usr/bin/env bash
    set -euo pipefail
    git add aur/
    git commit -m "chore: update AUR package" || true
    TMP=$(mktemp -d)
    trap 'rm -rf "$TMP"' EXIT
    git clone ssh://aur@aur.archlinux.org/clip-cli-bin.git "$TMP"
    cp aur/PKGBUILD aur/.SRCINFO "$TMP/"
    git -C "$TMP" add -A
    git -C "$TMP" commit -m "chore: update AUR package" || true
    git -C "$TMP" push origin master

# 发布新版本到 AUR（自动更新版本号 + 推送）
aur-release VERSION:
    #!/usr/bin/env bash
    set -euo pipefail
    sed -i 's/^pkgver=.*/pkgver={{VERSION}}/' aur/PKGBUILD
    sed -i 's/^pkgrel=.*/pkgrel=1/' aur/PKGBUILD
    (cd aur && makepkg --printsrcinfo > .SRCINFO)
    git add aur/
    git commit -m "chore: update AUR to v{{VERSION}}" || true
    TMP=$(mktemp -d)
    trap 'rm -rf "$TMP"' EXIT
    git clone ssh://aur@aur.archlinux.org/clip-cli-bin.git "$TMP"
    cp aur/PKGBUILD aur/.SRCINFO "$TMP/"
    git -C "$TMP" add -A
    git -C "$TMP" commit -m "chore: update AUR to v{{VERSION}}" || true
    git -C "$TMP" push origin master

# ─── Nightly AUR ───────────────────────────────────────

# 重新生成 aur-nightly/.SRCINFO
aur-nightly-srcinfo:
    cd aur-nightly && makepkg --printsrcinfo > .SRCINFO

# 提交 aur-nightly/ 变更并推送到 Nightly AUR
# AUR 禁止 force push，使用 clone → 更新文件 → commit → push 的可靠方式
aur-nightly-push:
    #!/usr/bin/env bash
    set -euo pipefail
    git add aur-nightly/
    git commit -m "chore: update nightly AUR package" || true
    TMP=$(mktemp -d)
    trap 'rm -rf "$TMP"' EXIT
    git clone ssh://aur@aur.archlinux.org/clip-cli-nightly-bin.git "$TMP"
    cp aur-nightly/PKGBUILD aur-nightly/.SRCINFO "$TMP/"
    git -C "$TMP" add -A
    git -C "$TMP" commit -m "chore: update nightly AUR package" || true
    git -C "$TMP" push origin master

# 发布 Nightly AUR（自动使用当前日期作为版本号 + 推送）
aur-nightly-release:
    #!/usr/bin/env bash
    set -euo pipefail
    DATE=$(date +%Y%m%d)
    sed -i "s/^pkgver=.*/pkgver=$DATE/" aur-nightly/PKGBUILD
    sed -i 's/^pkgrel=.*/pkgrel=1/' aur-nightly/PKGBUILD
    (cd aur-nightly && makepkg --printsrcinfo > .SRCINFO)
    git add aur-nightly/
    git commit -m "chore: update nightly AUR to $DATE" || true
    TMP=$(mktemp -d)
    trap 'rm -rf "$TMP"' EXIT
    git clone ssh://aur@aur.archlinux.org/clip-cli-nightly-bin.git "$TMP"
    cp aur-nightly/PKGBUILD aur-nightly/.SRCINFO "$TMP/"
    git -C "$TMP" add -A
    git -C "$TMP" commit -m "chore: update nightly AUR to $DATE" || true
    git -C "$TMP" push origin master
