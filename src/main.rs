//! Library Editor - 传奇2库文件编辑器 (Rust版本)
//!
//! 这是一个用于编辑传奇2游戏资源库文件的跨平台应用程序。
//! 支持的文件格式：
//! - MLibrary V1 (.wzl/.wzx)
//! - MLibrary V2 (.Lib)
//! - MLibrary V0 (.wil 旧格式)
//! - WeMade Library (.wil/.wix)
//! - WTL Library (.wtl)

#![warn(missing_docs)]
#![allow(dead_code)]

mod error;
mod formats;
#[cfg(feature = "gui")]
mod gui;
mod image;

use error::Result;
use tracing::{Level, info};

fn main() -> Result<()> {
    // 解析命令行参数
    let args: Vec<String> = std::env::args().collect();

    // 检查是否有 --no-gui 或 --cli 参数（强制使用 CLI 模式）
    let no_gui = args.iter().any(|a| a == "--no-gui" || a == "--cli");

    // 默认使用 GUI 模式（因为 gui 现在是默认 feature）
    if !no_gui {
        #[cfg(feature = "gui")]
        {
            return gui::run();
        }

        #[cfg(not(feature = "gui"))]
        {
            eprintln!("提示: 使用 '--features gui' 编译以启用 GUI 模式");
            eprintln!("运行命令: cargo run --features gui");
        }
    }

    run_cli(args)
}

/// 运行 CLI 模式
fn run_cli(args: Vec<String>) -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Library Editor CLI 模式启动中...");
    info!("支持格式: MLibrary V1/V2, WeMade, WTL");

    info!("");
    info!("使用方法:");
    info!("  library_editor.exe [选项] <文件路径>");
    info!("");
    info!("选项:");
    info!("  --no-gui, --cli    强制使用 CLI 模式 (当前默认为 GUI)");
    info!("  --help, -h         显示帮助信息");
    info!("");
    info!("支持格式:");
    info!("  - .wzl/.wzx (MLibrary V1)");
    info!("  - .Lib (MLibrary V2)");
    info!("  - .wil/.wix (WeMade Library)");
    info!("  - .wtl (WTL Library)");
    info!("");
    info!("注意: 程序默认使用 GUI 模式");
    info!("      (gui feature 当前已默认启用)");

    // 显示传入的文件参数
    if args.len() > 1 {
        let file_args: Vec<_> = args
            .iter()
            .filter(|a| !a.starts_with("--") && !a.starts_with('-'))
            .collect();

        if !file_args.is_empty() {
            info!("");
            info!("传入的文件:");
            for file in file_args {
                info!("  - {}", file);
            }
        }
    }

    Ok(())
}

/// 应用程序名称
pub const APP_NAME: &str = "Library Editor";

/// 应用程序版本（从 Cargo.toml 读取）
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 应用程序作者
pub const APP_AUTHOR: &str = "Rust Implementation";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_info() {
        assert_eq!(APP_NAME, "Library Editor");
    }
}
