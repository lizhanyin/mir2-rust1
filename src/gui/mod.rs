//! GUI 模块
//!
//! GUI 模块提供图形界面功能

pub use crate::error::Result;

use slint::SharedString;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing_appender::rolling;

slint::include_modules!();

/// 多线程加载的阈值
const MULTITHREAD_THRESHOLD: usize = 50;

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

    /// 更新缩略图数组（单线程，用于少量图像）
    fn update_thumbnails_single_thread(
        window: &AppWindow,
        loader: &mut crate::formats::LibraryLoader,
        count: usize,
    ) {
        use slint::Image;

        window.set_is_loading(true);
        window.set_load_progress(0);
        window.set_loaded_count(0);

        let mut thumbnails = Vec::with_capacity(count);

        for i in 0..count {
            match loader.get_preview(i) {
                Ok(Some(preview_img)) => {
                    if let Some(slint_image) = rgba_image_to_slint(&preview_img) {
                        thumbnails.push(slint_image);
                    } else {
                        thumbnails.push(Image::default());
                    }
                }
                Ok(None) => {
                    thumbnails.push(Image::default());
                }
                Err(e) => {
                    tracing::warn!("获取缩略图 {} 失败: {:?}", i, e);
                    thumbnails.push(Image::default());
                }
            }

            // 更新进度
            let progress = ((i + 1) * 100 / count) as i32;
            window.set_load_progress(progress);
            window.set_loaded_count((i + 1) as i32);
        }

        // 更新缩略图列表
        let model = slint::VecModel::from(thumbnails);
        window.set_thumbnails(slint::ModelRc::new(model));

        window.set_is_loading(false);
        window.set_load_progress(100);
        window.set_loaded_count(count as i32);
    }

    /// 更新主预览图（加载完整尺寸的图像）
    fn update_main_preview(
        window: &AppWindow,
        loader: &mut crate::formats::LibraryLoader,
        index: usize,
    ) {
        match loader.get_preview(index) {
            Ok(Some(preview_img)) => {
                if let Some(slint_image) = rgba_image_to_slint(&preview_img) {
                    window.set_main_preview(slint_image);
                }
            }
            Ok(None) => {
                window.set_main_preview(slint::Image::default());
            }
            Err(e) => {
                tracing::warn!("获取主预览图 {} 失败: {:?}", index, e);
                window.set_main_preview(slint::Image::default());
            }
        }
    }
}

/// 多线程加载器 - 使用内存存储
struct MultiThreadLoader {
    /// 预览图像存储 (线程安全)
    previews: Arc<Mutex<Vec<Option<image::RgbaImage>>>>,
    /// 已加载计数
    loaded_count: Arc<AtomicU32>,
    /// 总数
    total_count: usize,
    /// 是否完成
    is_complete: Arc<AtomicBool>,
}

impl MultiThreadLoader {
    fn new(total_count: usize) -> Self {
        Self {
            previews: Arc::new(Mutex::new(vec![None; total_count])),
            loaded_count: Arc::new(AtomicU32::new(0)),
            total_count,
            is_complete: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 启动多线程加载
    fn start_loading(&self, base_path: String, library_type: crate::formats::LibraryType) {
        let num_threads = std::cmp::min(
            4,
            std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(2),
        );

        let chunk_size = (self.total_count + num_threads - 1) / num_threads;

        for thread_id in 0..num_threads {
            let start = thread_id * chunk_size;
            let end = std::cmp::min(start + chunk_size, self.total_count);

            if start >= self.total_count {
                break;
            }

            let base_path = base_path.clone();
            let previews = Arc::clone(&self.previews);
            let loaded_count = self.loaded_count.clone();
            let total_count = self.total_count;
            let is_complete = self.is_complete.clone();

            thread::spawn(move || {
                // 在子线程中创建新的加载器实例
                let mut loader = match crate::formats::LibraryLoader::load(
                    &std::path::Path::new(&base_path).with_extension(library_type.main_extension()),
                ) {
                    Ok((_, loader)) => loader,
                    Err(e) => {
                        tracing::error!("子线程加载库失败: {:?}", e);
                        return;
                    }
                };

                for i in start..end {
                    // 获取图像预览
                    match loader.get_preview(i) {
                        Ok(Some(preview_img)) => {
                            // 存入共享内存
                            if let Ok(mut previews) = previews.lock() {
                                previews[i] = Some(preview_img);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("加载图像 {} 失败: {:?}", i, e);
                        }
                        _ => {}
                    }

                    // 更新计数
                    let count = loaded_count.fetch_add(1, Ordering::SeqCst) + 1;
                    tracing::debug!("加载进度: {}/{}", count, total_count);

                    if count >= total_count as u32 {
                        is_complete.store(true, Ordering::SeqCst);
                    }
                }
            });
        }
    }

    /// 获取当前加载进度
    fn get_progress(&self) -> (u32, bool) {
        let count = self.loaded_count.load(Ordering::SeqCst);
        let complete = self.is_complete.load(Ordering::SeqCst);
        (count, complete)
    }

    /// 从内存加载图像到 Slint
    fn load_from_memory(&self, index: usize) -> slint::Image {
        if let Ok(previews) = self.previews.lock() {
            if let Some(ref img) = previews[index] {
                return rgba_image_to_slint(img).unwrap_or_default();
            }
        }
        slint::Image::default()
    }

    /// 检查图像是否已加载
    fn is_loaded(&self, index: usize) -> bool {
        if let Ok(previews) = self.previews.lock() {
            return previews[index].is_some();
        }
        false
    }
}

/// 将 RGBA 图像转换为 Slint Image
fn rgba_image_to_slint(img: &image::RgbaImage) -> Option<slint::Image> {
    let width = img.width();
    let height = img.height();

    if width == 0 || height == 0 {
        return None;
    }

    let buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
        img.as_raw(),
        width,
        height,
    );
    Some(slint::Image::from_rgba8(buffer))
}

/// 初始化日志 - 同时输出到控制台和文件
fn init_logging() {
    use tracing::Level;
    use tracing_subscriber::{Registry, layer::SubscriberExt, util::SubscriberInitExt};

    let file_appender = rolling::daily("./logs", "library-editor.log");

    // 根据编译配置选择日志级别
    #[cfg(debug_assertions)]
    let log_level = Level::DEBUG;
    #[cfg(not(debug_assertions))]
    let log_level = Level::INFO;

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_level(true)
        .with_target(true);

    let console_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_level(true)
        .with_target(false);

    Registry::default()
        .with(file_layer)
        .with(console_layer)
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("library_editor", log_level)
                .with_default(log_level),
        )
        .init();
}

/// 运行 GUI 应用程序
pub fn run() -> Result<()> {
    use crate::error::LibraryError;
    use crate::formats::LibraryLoader;

    // 初始化日志
    init_logging();

    tracing::debug!("Library Editor GUI 启动");
    tracing::debug!("初始化 Slint 组件");

    // 创建主窗口
    let window =
        AppWindow::new().map_err(|e| LibraryError::Gui(format!("创建窗口失败: {:?}", e)))?;

    // 创建应用状态
    let state = AppState::new();

    // 设置初始状态
    window.set_status_text(SharedString::from("就绪"));
    window.set_file_name(SharedString::from(""));
    window.set_image_count(0);
    window.set_current_index(-1);
    window.set_image_width(0);
    window.set_image_height(0);
    window.set_image_x(0);
    window.set_image_y(0);
    window.set_image_format(SharedString::from("-"));
    window.set_load_progress(0);
    window.set_is_loading(false);
    window.set_loaded_count(0);

    tracing::debug!("初始状态设置完成");

    // 克隆窗口弱引用用于回调
    let window_weak = window.as_weak();

    // 设置打开文件回调
    {
        let window_weak = window_weak.clone();
        let library_loader = state.library_loader.clone();

        window.on_open_file(move || {
            tracing::debug!("用户触发打开文件操作");

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
                    tracing::debug!("用户取消了文件选择");
                    window.set_status_text(SharedString::from("未选择文件"));
                    return;
                }
            };

            tracing::debug!("选择的文件: {:?}", path);
            window.set_status_text(SharedString::from("正在加载..."));

            // 加载库文件
            match LibraryLoader::load(&path) {
                Ok((info, mut loader)) => {
                    tracing::debug!("库文件加载成功: {}", info.file_name);
                    tracing::debug!("  格式: {}", info.format_name());
                    tracing::debug!("  图像数: {}", info.image_count);

                    // 更新 UI
                    window.set_file_name(SharedString::from(&info.file_name));
                    window.set_image_count(info.image_count as i32);
                    window.set_image_format(SharedString::from(&info.format_name()));
                    window.set_current_index(if info.image_count > 0 { 0 } else { -1 });

                    // 加载第一张图像信息
                    if info.image_count > 0 {
                        tracing::debug!("加载第一张图像信息");
                        if let Ok(img_info) = loader.get_image_info(0) {
                            window.set_image_width(img_info.width);
                            window.set_image_height(img_info.height);
                            window.set_image_x(img_info.x);
                            window.set_image_y(img_info.y);
                            tracing::debug!("图像尺寸: {}x{}", img_info.width, img_info.height);
                        }
                        // 更新主预览图
                        AppState::update_main_preview(&window, &mut loader, 0);
                    } else {
                        // 没有图像，清空主预览
                        window.set_main_preview(slint::Image::default());
                    }

                    // 根据图像数量选择加载方式
                    if info.image_count > MULTITHREAD_THRESHOLD {
                        // 多线程加载
                        tracing::info!(
                            "图像数量 {} > {}，启用多线程加载",
                            info.image_count,
                            MULTITHREAD_THRESHOLD
                        );
                        window.set_status_text(SharedString::from(&format!(
                            "多线程加载中: {} 张图像...",
                            info.image_count
                        )));
                        window.set_is_loading(true);
                        window.set_load_progress(0);
                        window.set_loaded_count(0);

                        // 创建多线程加载器
                        let mt_loader = Arc::new(MultiThreadLoader::new(info.image_count));
                        let base_path = info.base_path.clone();
                        let library_type = info.library_type;

                        // 启动多线程加载
                        mt_loader.start_loading(base_path, library_type);

                        // 保存加载器引用
                        *library_loader.lock().unwrap() = Some(loader);

                        // 克隆用于定时器
                        let window_weak_timer = window_weak.clone();
                        let mt_loader_timer = Arc::clone(&mt_loader);
                        let total_count = info.image_count;
                        let timer_stopped = Arc::new(AtomicBool::new(false));
                        let timer_stopped_clone = timer_stopped.clone();

                        // 创建定时器轮询进度
                        let timer = Rc::new(slint::Timer::default());
                        let timer_clone = timer.clone();

                        timer.start(
                            slint::TimerMode::Repeated,
                            Duration::from_millis(100),
                            move || {
                                // 检查是否已停止
                                if timer_stopped_clone.load(Ordering::SeqCst) {
                                    return;
                                }

                                if let Some(win) = window_weak_timer.upgrade() {
                                    let (loaded, complete) = mt_loader_timer.get_progress();
                                    let progress = (loaded * 100 / total_count as u32) as i32;

                                    win.set_load_progress(progress);
                                    win.set_loaded_count(loaded as i32);

                                    // 更新已加载的缩略图
                                    let mut thumbnails = Vec::new();
                                    for i in 0..total_count {
                                        if mt_loader_timer.is_loaded(i) {
                                            thumbnails.push(mt_loader_timer.load_from_memory(i));
                                        } else {
                                            thumbnails.push(slint::Image::default());
                                        }
                                    }
                                    let model = slint::VecModel::from(thumbnails);
                                    win.set_thumbnails(slint::ModelRc::new(model));

                                    if complete {
                                        win.set_is_loading(false);
                                        win.set_status_text(SharedString::from(&format!(
                                            "已加载: {} 张图像",
                                            total_count
                                        )));
                                        // 停止定时器
                                        timer_stopped_clone.store(true, Ordering::SeqCst);
                                        timer_clone.stop();
                                    }
                                }
                            },
                        );

                        // 保持定时器引用，防止被 drop
                        std::mem::forget(timer);
                    } else {
                        // 单线程加载
                        tracing::info!(
                            "图像数量 {} <= {}，使用单线程加载",
                            info.image_count,
                            MULTITHREAD_THRESHOLD
                        );
                        window.set_status_text(SharedString::from(&format!(
                            "正在加载缩略图: {} 张图像...",
                            info.image_count
                        )));

                        if info.image_count > 0 {
                            AppState::update_thumbnails_single_thread(
                                &window,
                                &mut loader,
                                info.image_count,
                            );
                            window.set_status_text(SharedString::from(&format!(
                                "已加载: {} ({} 张图像)",
                                info.file_name, info.image_count
                            )));
                        }

                        // 保存加载器
                        *library_loader.lock().unwrap() = Some(loader);
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
                    window.set_main_preview(slint::Image::default());
                }
            }
        });
    }

    // 设置保存文件回调
    {
        let window_weak = window_weak.clone();
        let library_loader = state.library_loader.clone();

        window.on_save_file(move || {
            tracing::debug!("用户触发保存文件操作");

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
                        tracing::debug!("保存成功");
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
            tracing::debug!("用户触发另存为操作");

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
                    tracing::debug!("另存为: {:?}", path);
                    window.set_status_text(SharedString::from(&format!(
                        "已保存: {}",
                        path.display()
                    )));
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
            tracing::debug!("用户触发导出PNG操作");

            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            let current_index = window.get_current_index();
            if current_index < 0 {
                window.set_status_text(SharedString::from("请先选择一张图像"));
                return;
            }

            // 选择保存路径
            let path = match rfd::FileDialog::new()
                .add_filter("PNG 图像", &["png"])
                .set_title("导出PNG")
                .save_file()
            {
                Some(p) => p,
                None => {
                    window.set_status_text(SharedString::from("导出取消"));
                    return;
                }
            };

            // 导出图像
            if let Some(ref mut loader) = *library_loader.lock().unwrap() {
                match loader.export_png(current_index as usize, &path) {
                    Ok(_) => {
                        tracing::debug!("导出成功: {:?}", path);
                        window.set_status_text(SharedString::from(&format!(
                            "已导出: {}",
                            path.display()
                        )));
                    }
                    Err(e) => {
                        tracing::error!("导出失败: {:?}", e);
                        window.set_status_text(SharedString::from(&format!("导出失败: {}", e)));
                    }
                }
            }
        });
    }

    // 设置替换图像回调
    {
        let window_weak = window_weak.clone();
        let library_loader = state.library_loader.clone();

        window.on_replace_image(move || {
            tracing::debug!("用户触发替换图像操作");

            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            let current_index = window.get_current_index();
            if current_index < 0 {
                window.set_status_text(SharedString::from("请先选择一张图像"));
                return;
            }

            // 选择新图像
            let path = match rfd::FileDialog::new()
                .add_filter("图像文件", &["png", "bmp", "jpg", "jpeg"])
                .set_title("选择替换图像")
                .pick_file()
            {
                Some(p) => p,
                None => {
                    window.set_status_text(SharedString::from("替换取消"));
                    return;
                }
            };

            // 加载新图像
            match image::open(&path) {
                Ok(new_img) => {
                    let rgba = new_img.to_rgba8();

                    // TODO: 实现图像替换功能（需要根据库类型调用不同的方法）
                    // 目前仅更新预览
                    tracing::debug!("图像加载成功，替换功能待实现");
                    window.set_status_text(SharedString::from(&format!(
                        "图像已加载 (替换功能开发中)"
                    )));

                    // 更新预览
                    if let Some(slint_image) = rgba_image_to_slint(&rgba) {
                        window.set_main_preview(slint_image);
                    }
                }
                Err(e) => {
                    tracing::error!("加载图像失败: {:?}", e);
                    window.set_status_text(SharedString::from(&format!("加载图像失败: {}", e)));
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
            let count = window.get_image_count();

            if count == 0 {
                return;
            }

            let new_index = if current <= 0 { count - 1 } else { current - 1 };
            window.set_current_index(new_index);

            // 更新图像信息
            if let Some(ref mut loader) = *library_loader.lock().unwrap() {
                if let Ok(img_info) = loader.get_image_info(new_index as usize) {
                    window.set_image_width(img_info.width);
                    window.set_image_height(img_info.height);
                    window.set_image_x(img_info.x);
                    window.set_image_y(img_info.y);
                }
                AppState::update_main_preview(&window, loader, new_index as usize);
            }

            tracing::debug!("切换到上一张图像: {}", new_index);
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

            if count == 0 {
                return;
            }

            let new_index = if current >= count - 1 { 0 } else { current + 1 };
            window.set_current_index(new_index);

            // 更新图像信息
            if let Some(ref mut loader) = *library_loader.lock().unwrap() {
                if let Ok(img_info) = loader.get_image_info(new_index as usize) {
                    window.set_image_width(img_info.width);
                    window.set_image_height(img_info.height);
                    window.set_image_x(img_info.x);
                    window.set_image_y(img_info.y);
                }
                AppState::update_main_preview(&window, loader, new_index as usize);
            }

            tracing::debug!("切换到下一张图像: {}", new_index);
        });
    }

    // 设置缩略图点击回调
    {
        let window_weak = window_weak.clone();
        let library_loader = state.library_loader.clone();

        window.on_thumbnail_clicked(move |index| {
            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            window.set_current_index(index as i32);

            // 更新图像信息
            if let Some(ref mut loader) = *library_loader.lock().unwrap() {
                if let Ok(img_info) = loader.get_image_info(index as usize) {
                    window.set_image_width(img_info.width);
                    window.set_image_height(img_info.height);
                    window.set_image_x(img_info.x);
                    window.set_image_y(img_info.y);
                }
                AppState::update_main_preview(&window, loader, index as usize);
            }

            tracing::debug!("点击缩略图: {}", index);
        });
    }

    // 设置切换预览背景回调
    {
        let window_weak = window_weak.clone();

        window.on_toggle_preview_bg(move || {
            if let Some(window) = window_weak.upgrade() {
                let current = window.get_preview_bg_light();
                window.set_preview_bg_light(!current);
                tracing::debug!("切换预览背景: {}", !current);
            }
        });
    }

    // 设置键盘事件回调 - 在 Rust 端处理导航
    {
        let window_weak = window_weak.clone();
        let library_loader = state.library_loader.clone();

        window.on_key_pressed(move |text| {
            let window = match window_weak.upgrade() {
                Some(w) => w,
                None => return,
            };

            let image_count = window.get_image_count();
            if image_count == 0 {
                return;
            }

            let current = window.get_current_index();
            let mut new_index = current;

            // 判断按键
            if text == "Left" || text == "←" {
                if current > 0 {
                    new_index = current - 1;
                }
            } else if text == "Right" || text == "→" {
                if current < image_count - 1 {
                    new_index = current + 1;
                }
            } else if text == "Home" {
                new_index = 0;
            } else if text == "End" {
                new_index = image_count - 1;
            } else {
                return; // 不是导航键，不处理
            }

            // 如果索引有变化，更新UI
            if new_index != current {
                window.set_current_index(new_index);
                tracing::debug!("切换到图像: {}", new_index);

                if let Some(ref mut loader) = *library_loader.lock().unwrap() {
                    if let Ok(img_info) = loader.get_image_info(new_index as usize) {
                        window.set_image_width(img_info.width);
                        window.set_image_height(img_info.height);
                        window.set_image_x(img_info.x);
                        window.set_image_y(img_info.y);
                    }
                    AppState::update_main_preview(&window, loader, new_index as usize);
                }
            }
        });
    }

    tracing::debug!("运行主窗口");
    window
        .run()
        .map_err(|e| LibraryError::Gui(format!("运行窗口失败: {:?}", e)))?;

    Ok(())
}
