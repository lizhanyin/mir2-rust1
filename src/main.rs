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
mod image;
mod formats;
// mod gui;  // 暂时禁用 GUI

use error::Result;
use tracing::{info};
use tracing_subscriber;

fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Library Editor 启动中...");
    info!("支持格式: MLibrary V1/V2, WeMade, WTL");

    // TODO: 实现 GUI 界面
    info!("GUI 模块暂时禁用");
    info!("项目结构已创建，包含:");
    info!("  - 错误处理模块 (error.rs)");
    info!("  - 图像处理模块 (image/)");
    info!("  - 库文件格式模块 (formats/)");
    info!("");
    info!("使用方法:");
    info!("  library_editor.exe <文件路径>");
    info!("");
    info!("支持格式:");
    info!("  - .wzl/.wzx (MLibrary V1)");
    info!("  - .Lib (MLibrary V2)");
    info!("  - .wil/.wix (WeMade Library)");
    info!("  - .wtl (WTL Library)");

    Ok(())
}

/// 应用程序信息
pub const APP_NAME: &str = "Library Editor";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_AUTHOR: &str = "Rust Implementation";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_info() {
        assert_eq!(APP_NAME, "Library Editor");
    }
}
