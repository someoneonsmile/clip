use arboard::Clipboard;
#[cfg(target_os = "linux")]
use arboard::SetExtLinux;
use base64::Engine;
use clap::{Parser, Subcommand};
use is_terminal::IsTerminal;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process;

/// daemonize 模式标记参数。正常用户不会传入，仅用于重新 exec 自身时识别后台持有进程。
const DAEMONIZE_ARG: &str = "__clip_internal_daemonize";

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
    // 后台持有进程：在 clap 解析参数前拦截，因为该标记并非合法子命令
    #[cfg(target_os = "linux")]
    if env::args().nth(1).as_deref() == Some(DAEMONIZE_ARG) {
        daemonize();
        return;
    }

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
        // X11 下通过后台 daemon 持有数据，避免进程退出后 selection 丢失（见 try_set_clipboard）
        let clipboard_ok = try_set_clipboard(&content);

        // 系统剪贴板不可用时，回退到 OSC52（兼容纯 Wayland 合成器等场景）
        if !clipboard_ok {
            write_osc52(&content);
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

/// 尝试写入系统剪贴板。返回 true 表示成功（或已交由后台 daemon 处理）。
fn try_set_clipboard(content: &[u8]) -> bool {
    #[cfg(target_os = "linux")]
    {
        if is_wayland() {
            // Wayland：数据由 compositor 持有，进程退出不丢，直接写入即可
            set_clipboard_direct(content)
        } else {
            // X11：spawn 后台 daemon 持有数据，避免进程退出后 selection 丢失
            spawn_clipboard_daemon(content)
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        // macOS/Windows：系统剪贴板由 OS 持久持有，直接写入即可
        set_clipboard_direct(content)
    }
}

/// 直接调用 arboard 写入系统剪贴板（不等待），适用于数据由 OS/compositor 持久持有的平台。
fn set_clipboard_direct(content: &[u8]) -> bool {
    let mut cb = match Clipboard::new() {
        Ok(cb) => cb,
        Err(e) => {
            eprintln!("system clipboard unavailable: {}", e);
            return false;
        }
    };
    let text = String::from_utf8_lossy(content).into_owned();
    match cb.set_text(text) {
        Ok(()) => true,
        Err(e) => {
            eprintln!("system clipboard unavailable: {}", e);
            false
        }
    }
}

/// 检测是否为 Wayland 会话。arboard 在 Linux 下优先用 Wayland data-control（存在 WAYLAND_DISPLAY 时），
/// 回退 X11。
#[cfg(target_os = "linux")]
fn is_wayland() -> bool {
    env::var_os("WAYLAND_DISPLAY").is_some()
}

/// X11 下 spawn 后台 daemon 进程（重新 exec 自身）持有剪贴板，避免主进程退出后数据丢失。
/// daemon 会一直存活直到剪贴板被其他进程覆盖。返回 spawn 是否成功。
#[cfg(target_os = "linux")]
fn spawn_clipboard_daemon(content: &[u8]) -> bool {
    let exe = match env::current_exe() {
        Ok(exe) => exe,
        Err(e) => {
            eprintln!("failed to locate current executable: {}", e);
            return false;
        }
    };

    let mut child = match process::Command::new(exe)
        .arg(DAEMONIZE_ARG)
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .current_dir("/")
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            eprintln!("failed to spawn clipboard daemon: {}", e);
            return false;
        }
    };

    // 通过 stdin 将内容传给 daemon；写入后关闭写端，daemon 读到 EOF 后开始持有剪贴板
    if let Some(mut stdin) = child.stdin.take()
        && stdin.write_all(content).is_err()
    {
        return false;
    }
    true
}

/// daemon 进程入口：从 stdin 读取内容，写入剪贴板并持有直到被其他进程覆盖。
#[cfg(target_os = "linux")]
fn daemonize() {
    let mut data = Vec::new();
    if io::stdin().read_to_end(&mut data).is_err() {
        process::exit(1);
    }
    let text = String::from_utf8_lossy(&data).into_owned();

    // wait() 会阻塞直到剪贴板被覆盖（收到 SelectionClear），此时 daemon 自动退出
    let result: Result<(), arboard::Error> = (|| {
        let mut cb = Clipboard::new()?;
        cb.set().wait().text(text)
    })();

    if let Err(e) = result {
        eprintln!("clipboard daemon failed: {}", e);
        process::exit(1);
    }
}
