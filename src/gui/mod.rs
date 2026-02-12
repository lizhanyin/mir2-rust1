//! GUI 模块
//!
//! GUI 模块提供图形界面功能

pub use crate::error::Result;

use slint::SharedString;

slint::include_modules!();

/// 应用状态
struct AppState {
    // 存储当前加载的库文件信息
    file_path: std::path::PathBuf,
    images_loaded: bool,
}

impl AppState {
    fn new() -> Self {
        Self {
            file_path: std::path::PathBuf::new(),
            images_loaded: false,
        }
    }
}

/// 运行 GUI 应用程序
pub fn run() -> Result<()> {
    use crate::error::LibraryError;

    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Library Editor GUI 启动");

    // 创建主窗口
    let window = AppWindow::new()
        .map_err(|e| LibraryError::Gui(format!("创建窗口失败: {:?}", e)))?;

    // 创建应用状态
    let _state = AppState::new();

    // 设置初始状态
    window.set_status_text(SharedString::from("就绪"));
    window.set_file_name(SharedString::from(""));
    window.set_image_count(10); // 示例数据
    window.set_current_index(-1);
    window.set_image_width(48);
    window.set_image_height(48);
    window.set_image_format(SharedString::from("WIL"));

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

            tracing::info!("打开文件对话框被调用");

            // 直接调用文件对话框
            match rfd::FileDialog::new()
                .add_filter("传奇库文件", &["wzl", "wzx", "lib", "wtl", "wil"])
                .add_filter("所有文件", &["*"])
                .set_title("打开库文件")
                .pick_file()
            {
                Some(path) => {
                    tracing::info!("选择的文件: {:?}", path);
                    let file_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    window.set_status_text(SharedString::from(&format!("已加载: {}", path.display())));
                    window.set_file_name(SharedString::from(&file_name));
                    window.set_image_count(10);
                    window.set_current_index(0); // 加载第一个图像

                    // 更新图像信息
                    window.set_image_width(48);
                    window.set_image_height(48);
                    window.set_image_format(SharedString::from("WIL"));
                }
                None => {
                    tracing::info!("用户取消了文件选择");
                    window.set_status_text(SharedString::from("未选择文件"));
                }
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

            tracing::info!("保存文件回调");

            // TODO: 实现保存逻辑
            window.set_status_text(SharedString::from("保存功能待实现"));
        });
    }

    // 设置另存为文件回调
    {
        let window_weak = window_weak.clone();
        window.on_save_as_file(move || {
            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            match rfd::FileDialog::new()
                .add_filter("传奇库文件", &["wzl", "lib", "wtl"])
                .set_title("另存为")
                .save_file()
            {
                Some(path) => {
                    tracing::info!("另存为: {:?}", path);
                    window.set_status_text(SharedString::from(&format!("已保存: {}", path.display())));
                }
                None => {
                    window.set_status_text(SharedString::from("保存取消"));
                }
            }
        });
    }

    // 设置导出PNG回调
    {
        let window_weak = window_weak.clone();
        window.on_export_png(move || {
            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            let current_index = window.get_current_index();
            if current_index < 0 {
                window.set_status_text(SharedString::from("请先选择一张图像"));
                return;
            }

            match rfd::FileDialog::new()
                .add_filter("PNG图像", &["png"])
                .set_title("导出PNG")
                .save_file()
            {
                Some(path) => {
                    tracing::info!("导出PNG: {:?}", path);
                    window.set_status_text(SharedString::from(&format!("已导出: {}", path.display())));
                }
                None => {
                    window.set_status_text(SharedString::from("导出取消"));
                }
            }
        });
    }

    // 设置替换图像回调
    {
        let window_weak = window_weak.clone();
        window.on_replace_image(move || {
            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            let current_index = window.get_current_index();
            if current_index < 0 {
                window.set_status_text(SharedString::from("请先选择一张图像"));
                return;
            }

            match rfd::FileDialog::new()
                .add_filter("图像文件", &["png", "jpg", "jpeg", "bmp"])
                .set_title("选择替换图像")
                .pick_file()
            {
                Some(path) => {
                    tracing::info!("替换图像: {:?}", path);
                    window.set_status_text(SharedString::from(&format!("已替换图像 {}", current_index)));
                }
                None => {
                    window.set_status_text(SharedString::from("替换取消"));
                }
            }
        });
    }

    // 设置上一张图像回调
    {
        let window_weak = window_weak.clone();
        window.on_prev_image(move || {
            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            let current = window.get_current_index();
            if current > 0 {
                window.set_current_index(current - 1);
                tracing::info!("显示图像: {}", current - 1);
                window.set_status_text(SharedString::from(&format!("显示图像 {}", current - 1)));
            }
        });
    }

    // 设置下一张图像回调
    {
        let window_weak = window_weak.clone();
        window.on_next_image(move || {
            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            let current = window.get_current_index();
            let count = window.get_image_count();
            if current < count - 1 {
                window.set_current_index(current + 1);
                tracing::info!("显示图像: {}", current + 1);
                window.set_status_text(SharedString::from(&format!("显示图像 {}", current + 1)));
            }
        });
    }

    // 设置缩略图点击回调
    {
        let window_weak = window_weak.clone();
        window.on_thumbnail_clicked(move |index: i32| {
            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            window.set_current_index(index);
            tracing::info!("点击缩略图: {}", index);
            window.set_status_text(SharedString::from(&format!("选择图像 {}", index)));
        });
    }

    // 显示窗口
    window.show()
        .map_err(|e| LibraryError::Gui(format!("显示窗口失败: {:?}", e)))?;

    // 运行事件循环
    slint::run_event_loop()
        .map_err(|e| LibraryError::Gui(format!("事件循环错误: {:?}", e)))?;

    Ok(())
}
