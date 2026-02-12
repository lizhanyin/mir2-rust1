#![cfg_attr(feature = "gui", windows_subsystem = "windows")]

//! Library Editor GUI 版本
//!
//! 使用 Slint 框架构建图形界面

use slint::SharedString;

slint::include_modules!();

/// 应用状态
struct AppState;

impl AppState {
    fn new() -> Self {
        Self
    }

    fn update_status(window: &AppWindow, text: &str) {
        window.set_status_text(SharedString::from(text));
    }

    fn update_file_info(window: &AppWindow, name: &str, count: i32) {
        window.set_file_name(SharedString::from(name));
        window.set_image_count(count);
    }
}

/// 运行 GUI 应用程序
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Library Editor GUI 启动");

    // 创建主窗口
    let window = AppWindow::new()?;

    // 创建应用状态
    let _state = AppState::new();

    // 设置初始状态
    window.set_status_text(SharedString::from("就绪"));
    window.set_file_name(SharedString::from(""));
    window.set_image_count(4); // 示例数据

    // 克隆窗口弱引用用于回调
    let window_weak = window.as_weak();

    // 设置打开文件回调
    {
        let window_weak = window_weak.clone();
        window.on_open_file(move || {
            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            window.set_status_text(SharedString::from("正在选择文件..."));

            if let Some(path) = rfd::FileDialog::new()
                .add_filter("传奇库文件", &["*.wzl", "*.wzx", "*.lib", "*.wtl", "*.wil"])
                .set_title("打开库文件")
                .pick_file()
            {
                tracing::info!("选择的文件: {:?}", path);
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                window.set_status_text(SharedString::from(&format!("已加载: {}", path.display())));
                window.set_file_name(SharedString::from(&file_name));
                window.set_image_count(0); // TODO: 解析实际图像数量
            } else {
                window.set_status_text(SharedString::from("未选择文件"));
            }
        });
    }

    // 设置保存文件回调
    {
        let window_weak = window_weak.clone();
        window.on_save_file(move || {
            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            if let Some(path) = rfd::FileDialog::new()
                .add_filter("传奇库文件", &["*.wzl", "*.lib", "*.wtl"])
                .set_title("保存库文件")
                .save_file()
            {
                tracing::info!("保存文件: {:?}", path);
                window.set_status_text(SharedString::from(&format!("已保存: {}", path.display())));
            } else {
                window.set_status_text(SharedString::from("保存取消"));
            }
        });
    }

    // 显示窗口
    window.show()?;

    // 运行事件循环
    slint::run_event_loop()?;
    Ok(())
}

// GUI 二进制的独立入口点
#[cfg(feature = "gui")]
fn main() -> Result<(), slint::PlatformError> {
    run().map_err(|e| slint::PlatformError::from(format!("{:?}", e)))
}
