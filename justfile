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

# 安装到系统（~/.cargo/bin/clip）
install:
    cargo install --path .

# 监听文件变更，自动 check
watch:
    cargo watch -x check

# 监听文件变更，自动 fmt + lint + build
watch-all:
    cargo watch -x fmt -x clippy -x build

# 全量检查（fmt → clippy → test → release build）
ci: fmt-check lint-strict test release
