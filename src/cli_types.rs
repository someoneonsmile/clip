// 命令行定义。
//
// 本文件同时被两处使用：
//   1. `src/main.rs` 里 `mod cli_types;`
//   2. `build.rs` 里 `include!("src/cli_types.rs");`（为了用 clap 生成补全和 man）
//
// 因此这里只能依赖 clap 与标准库，不能出现 `use crate::...`。

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "clip",
    about = "System clipboard bridge — copy stdin to clipboard (like tee), paste clipboard to stdout.\n\nWhen no subcommand is given, mode is auto-detected:\n  piped input → copy + tee,  terminal → paste.",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Copy stdin content to clipboard
    Copy,
    /// Paste clipboard content to stdout
    Paste,
}
