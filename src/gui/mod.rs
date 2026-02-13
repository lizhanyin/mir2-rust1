//! GUI 模块
//!
//! GUI 模块提供图形界面功能

pub use crate::error::Result;

use slint::SharedString;
use std::rc::Rc;
use std::sync::Mutex;

slint::include_modules!();

/// 应用状态
struct AppState {
    /// 库加载器
    library_loader: Rc<Mutex<Option<crate::formats::LibraryLoader>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            library_loader: Rc::new(Mutex::new(None)),
        }
    }
}

/// 运行 GUI 应用程序
pub fn run() -> Result<()> {
    use crate::error::LibraryError;
    use crate::formats::LibraryLoader;

    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tracing::info!("Library Editor GUI 启动");
    tracing::debug!("初始化 Slint 组件");

    // 创建主窗口
    let window = AppWindow::new()
        .map_err(|e| LibraryError::Gui(format!("创建窗口失败: {:?}", e)))?;

    // 创建应用状态
    let state = AppState::new();

    // 设置初始状态
    window.set_status_text(SharedString::from("就绪"));
    window.set_file_name(SharedString::from(""));
    window.set_image_count(0);
    window.set_current_index(-1);
    window.set_image_width(0);
    window.set_image_height(0);
    window.set_image_format(SharedString::from("-"));

    tracing::debug!("初始状态设置完成");

    // 克隆窗口弱引用用于回调
    let window_weak = window.as_weak();

    // 设置打开文件回调
    {
        let window_weak = window_weak.clone();
        let library_loader = state.library_loader.clone();

        window.on_open_file(move || {
            tracing::info!("用户触发打开文件操作");

            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => {
                    tracing::error!("无法获取窗口引用");
                    return;
                }
            };

            window.set_status_text(SharedString::from("正在选择文件..."));

            // 调用文件对话框
            tracing::debug!("打开文件对话框");
            let path = match rfd::FileDialog::new()
                .add_filter("传奇库文件", &["lib", "wzl", "wil", "wtl"])
                .add_filter("所有文件", &["*"])
                .set_title("打开库文件")
                .pick_file()
            {
                Some(p) => p,
                None => {
                    tracing::info!("用户取消了文件选择");
                    window.set_status_text(SharedString::from("未选择文件"));
                    return;
                }
            };

            tracing::info!("选择的文件: {:?}", path);
            window.set_status_text(SharedString::from("正在加载..."));

            // 加载库文件
            match LibraryLoader::load(&path) {
                Ok((info, loader)) => {
                    tracing::info!("库文件加载成功: {}", info.file_name);
                    tracing::debug!("  格式: {}", info.format_name());
                    tracing::debug!("  图像数: {}", info.image_count);

                    // 更新 UI
                    window.set_file_name(SharedString::from(&info.file_name));
                    window.set_image_count(info.image_count as i32);
                    window.set_image_format(SharedString::from(&info.format_name()));
                    window.set_current_index(if info.image_count > 0 { 0 } else { -1 });

                    // 保存加载器
                    *library_loader.lock().unwrap() = Some(loader);

                    // 更新状态
                    window.set_status_text(SharedString::from(&format!(
                        "已加载: {} ({} 张图像)",
                        info.file_name, info.image_count
                    )));

                    // 加载第一张图像信息
                    if info.image_count > 0 {
                        tracing::debug!("加载第一张图像信息");
                        if let Some(ref mut loader) = *library_loader.lock().unwrap() {
                            if let Ok(img_info) = loader.get_image_info(0) {
                                window.set_image_width(img_info.width);
                                window.set_image_height(img_info.height);
                                tracing::debug!("图像尺寸: {}x{}", img_info.width, img_info.height);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("加载库文件失败: {:?}", e);
                    window.set_status_text(SharedString::from(&format!("加载失败: {}", e)));

                    // 清空状态
                    window.set_file_name(SharedString::from(""));
                    window.set_image_count(0);
                    window.set_current_index(-1);
                    window.set_image_width(0);
                    window.set_image_height(0);
                }
            }
        });
    }

    // 设置保存文件回调
    {
        let window_weak = window_weak.clone();
        let library_loader = state.library_loader.clone();

        window.on_save_file(move || {
            tracing::info!("用户触发保存文件操作");

            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            // 检查是否有加载的库
            let has_library = library_loader.lock().unwrap().is_some();

            if !has_library {
                tracing::warn!("没有加载的库文件");
                window.set_status_text(SharedString::from("请先打开一个库文件"));
                return;
            }

            window.set_status_text(SharedString::from("正在保存..."));

            // 执行保存
            if let Some(ref loader) = *library_loader.lock().unwrap() {
                match loader.save() {
                    Ok(_) => {
                        tracing::info!("保存成功");
                        window.set_status_text(SharedString::from("保存成功"));
                    }
                    Err(e) => {
                        tracing::error!("保存失败: {:?}", e);
                        window.set_status_text(SharedString::from(&format!("保存失败: {}", e)));
                    }
                }
            }
        });
    }

    // 设置另存为文件回调
    {
        let window_weak = window_weak.clone();

        window.on_save_as_file(move || {
            tracing::info!("用户触发另存为操作");

            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            match rfd::FileDialog::new()
                .add_filter("传奇库文件", &["lib", "wzl", "wtl"])
                .set_title("另存为")
                .save_file()
            {
                Some(path) => {
                    tracing::info!("另存为: {:?}", path);
                    window.set_status_text(SharedString::from(&format!("已保存: {}", path.display())));
                    // TODO: 实现另存为逻辑
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
        let library_loader = state.library_loader.clone();

        window.on_export_png(move || {
            tracing::info!("用户触发导出PNG操作");

            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            let current_index = window.get_current_index();
            if current_index < 0 {
                tracing::warn!("没有选中的图像");
                window.set_status_text(SharedString::from("请先选择一张图像"));
                return;
            }

            // 检查是否有加载的库
            let has_library = library_loader.lock().unwrap().is_some();
            if !has_library {
                tracing::warn!("没有加载的库文件");
                window.set_status_text(SharedString::from("请先打开一个库文件"));
                return;
            }

            match rfd::FileDialog::new()
                .add_filter("PNG图像", &["png"])
                .set_title("导出PNG")
                .save_file()
            {
                Some(path) => {
                    tracing::info!("导出PNG到: {:?}", path);
                    window.set_status_text(SharedString::from("正在导出..."));

                    if let Some(ref mut loader) = *library_loader.lock().unwrap() {
                        match loader.export_png(current_index as usize, &path) {
                            Ok(_) => {
                                tracing::info!("导出成功");
                                window.set_status_text(SharedString::from(&format!("已导出: {}", path.display())));
                            }
                            Err(e) => {
                                tracing::error!("导出失败: {:?}", e);
                                window.set_status_text(SharedString::from(&format!("导出失败: {}", e)));
                            }
                        }
                    }
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
        let library_loader = state.library_loader.clone();

        window.on_replace_image(move || {
            tracing::info!("用户触发替换图像操作");

            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            let current_index = window.get_current_index();
            if current_index < 0 {
                tracing::warn!("没有选中的图像");
                window.set_status_text(SharedString::from("请先选择一张图像"));
                return;
            }

            // 检查是否有加载的库
            let has_library = library_loader.lock().unwrap().is_some();
            if !has_library {
                tracing::warn!("没有加载的库文件");
                window.set_status_text(SharedString::from("请先打开一个库文件"));
                return;
            }

            match rfd::FileDialog::new()
                .add_filter("图像文件", &["png", "jpg", "jpeg", "bmp"])
                .set_title("选择替换图像")
                .pick_file()
            {
                Some(path) => {
                    tracing::info!("选择的图像: {:?}", path);
                    window.set_status_text(SharedString::from("正在替换..."));

                    // TODO: 加载图像并替换
                    tracing::info!("替换图像 {} 为 {:?}", current_index, path);
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
        let library_loader = state.library_loader.clone();

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

                // 更新图像信息
                if let Some(ref mut loader) = *library_loader.lock().unwrap() {
                    if let Ok(img_info) = loader.get_image_info((current - 1) as usize) {
                        window.set_image_width(img_info.width);
                        window.set_image_height(img_info.height);
                    }
                }
            }
        });
    }

    // 设置下一张图像回调
    {
        let window_weak = window_weak.clone();
        let library_loader = state.library_loader.clone();

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

                // 更新图像信息
                if let Some(ref mut loader) = *library_loader.lock().unwrap() {
                    if let Ok(img_info) = loader.get_image_info((current + 1) as usize) {
                        window.set_image_width(img_info.width);
                        window.set_image_height(img_info.height);
                    }
                }
            }
        });
    }

    // 设置缩略图点击回调
    {
        let window_weak = window_weak.clone();
        let library_loader = state.library_loader.clone();

        window.on_thumbnail_clicked(move |index: i32| {
            tracing::info!("点击缩略图: {}", index);

            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            window.set_current_index(index);
            window.set_status_text(SharedString::from(&format!("选择图像 {}", index)));

            // 更新图像信息
            if let Some(ref mut loader) = *library_loader.lock().unwrap() {
                if let Ok(img_info) = loader.get_image_info(index as usize) {
                    window.set_image_width(img_info.width);
                    window.set_image_height(img_info.height);
                    tracing::debug!("图像尺寸: {}x{}", img_info.width, img_info.height);
                }
            }
        });
    }

    tracing::debug!("所有回调设置完成");

    // 显示窗口
    window.show()
        .map_err(|e| LibraryError::Gui(format!("显示窗口失败: {:?}", e)))?;

    tracing::info!("窗口已显示，进入事件循环");

    // 运行事件循环
    slint::run_event_loop()
        .map_err(|e| LibraryError::Gui(format!("事件循环错误: {:?}", e)))?;

    Ok(())
}
