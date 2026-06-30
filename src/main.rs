use arboard::Clipboard;
use base64::Engine;
use clap::{Parser, Subcommand};
use is_terminal::IsTerminal;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "clip",
    about = "System clipboard bridge — copy stdin to clipboard (like tee), paste clipboard to stdout.\n\nWhen no subcommand is given, mode is auto-detected:\n  piped input → copy + tee,  terminal → paste.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Copy stdin content to clipboard
    Copy,
    /// Paste clipboard content to stdout
    Paste,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Copy) => {
            let _ = store();
        }
        Some(Command::Paste) => read(),
        None => {
            // 判断模式：stdin 是否为终端（TTY）
            //   - 非 TTY（管道输入）→ 存储模式
            //   - TTY（交互终端）→ 读取模式
            if io::stdin().is_terminal() {
                read();
            } else {
                let content = store();
                if !content.is_empty() {
                    let mut stdout = io::stdout().lock();
                    let _ = stdout.write_all(&content);
                    let _ = stdout.write_all(b"\n");
                }
            }
        }
    }
}

/// 存储模式：读取 stdin 全部内容，写入系统剪贴板 + 文件缓存 + OSC52(SSH)
/// 返回实际存储的内容（已去除末尾换行符），调用方可将其 tee 到 stdout
fn store() -> Vec<u8> {
    let mut content = Vec::new();
    if let Err(e) = io::stdin().read_to_end(&mut content) {
        eprintln!("failed to read stdin: {}", e);
        process::exit(1);
    }

    // 去掉末尾换行符（echo 等命令默认会追加 \n）
    if content.last() == Some(&b'\n') {
        content.pop();
    }

    if content.is_empty() {
        eprintln!("warning: empty input, clipboard unchanged");
        return Vec::new();
    }

    let ssh = is_ssh();

    if ssh {
        // SSH 环境：跳过系统剪贴板（必然不可用），直接走文件缓存 + OSC52
        let path = cache_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(e) = fs::write(&path, &content) {
            eprintln!("failed to write cache: {}", e);
        }
        write_osc52(&content);
    } else {
        // 本地环境：尝试写入系统剪贴板
        match Clipboard::new() {
            Ok(mut cb) => {
                let text = String::from_utf8_lossy(&content).into_owned();
                if let Err(e) = cb.set_text(text) {
                    eprintln!("system clipboard unavailable: {}", e);
                }
            }
            Err(e) => {
                eprintln!("system clipboard unavailable: {}", e);
            }
        }

        // 文件缓存作为可靠回退
        let path = cache_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(e) = fs::write(&path, &content) {
            eprintln!("failed to write cache: {}", e);
        }
    }

    content
}

/// 读取模式：优先系统剪贴板（本地），SSH 下直接读文件缓存，输出到 stdout
fn read() {
    let ssh = is_ssh();

    if !ssh {
        // 本地环境：尝试从系统剪贴板读取
        match Clipboard::new() {
            Ok(mut cb) => match cb.get_text() {
                Ok(text) => {
                    print!("{}", text);
                    return;
                }
                Err(e) => {
                    eprintln!("system clipboard read failed: {}", e);
                }
            },
            Err(e) => {
                eprintln!("system clipboard unavailable: {}", e);
            }
        }
    }

    // 回退：从文件缓存读取（SSH 下直接走此路径）
    let path = cache_path();
    match fs::read_to_string(&path) {
        Ok(text) => {
            if text.is_empty() {
                eprintln!("clipboard is empty");
                process::exit(1);
            }
            print!("{}", text);
        }
        Err(e) => {
            eprintln!("no clipboard data available ({})", e);
            process::exit(1);
        }
    }
}

/// 剪贴板文件缓存路径： ~/.cache/clipboard/data
fn cache_path() -> PathBuf {
    let base = if let Ok(dir) = env::var("XDG_CACHE_HOME") {
        PathBuf::from(dir)
    } else {
        let home = env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        PathBuf::from(home).join(".cache")
    };
    base.join("clipboard").join("data")
}

/// 检测是否在 SSH 会话中
fn is_ssh() -> bool {
    env::var("SSH_TTY").is_ok()
        || env::var("SSH_CLIENT").is_ok()
        || env::var("SSH_CONNECTION").is_ok()
}

/// 将内容通过 OSC52 转义序列发送到终端，使终端将内容写入本地系统剪贴板。
/// 优先写入 /dev/tty，回退到 stderr。
fn write_osc52(content: &[u8]) {
    let encoded = base64::engine::general_purpose::STANDARD.encode(content);
    // OSC52 序列格式：ESC ] 5 2 ; c ; <base64> BEL
    // 使用 BEL (\x07) 结尾而非 ST (\x1b\\)，兼容性更好
    let osc52 = format!("\x1b]52;c;{}\x07", encoded);

    // 尝试写入 /dev/tty，这样即使 stdout 被重定向也能到达终端
    if let Ok(mut tty) = fs::OpenOptions::new().write(true).open("/dev/tty") {
        let _ = tty.write_all(osc52.as_bytes());
        let _ = tty.flush();
    } else {
        // 回退到 stderr（比 stdout 更可靠，因为 stdout 可能在管道中被消费）
        let _ = io::stderr().write_all(osc52.as_bytes());
        let _ = io::stderr().flush();
    }
}
