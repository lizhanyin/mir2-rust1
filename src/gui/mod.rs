//! GUI 模块
//!
//! GUI 模块提供图形界面功能

pub use crate::error::Result;

use slint::{Model, SharedString};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing_appender::rolling;

slint::include_modules!();

/// 默认 LRU 缓存最大容量
const DEFAULT_CACHE_MAX_SIZE: u64 = 9999999;

/// 默认按键节流间隔（毫秒）
const DEFAULT_KEY_THROTTLE_MS: u64 = 50;

/// 应用程序设置（支持动态修改）
#[derive(Debug)]
struct AppSettings {
    /// LRU 缓存最大容量
    cache_max_size: AtomicU64,
    /// 按键节流间隔（毫秒）
    key_throttle_ms: AtomicU64,
}

impl AppSettings {
    fn new() -> Self {
        Self {
            cache_max_size: AtomicU64::new(DEFAULT_CACHE_MAX_SIZE),
            key_throttle_ms: AtomicU64::new(DEFAULT_KEY_THROTTLE_MS),
        }
    }

    fn get_cache_max_size(&self) -> usize {
        let size = self.cache_max_size.load(Ordering::SeqCst);
        if size == 0 {
            usize::MAX
        } else {
            size as usize
        }
    }

    fn set_cache_max_size(&self, size: u64) {
        self.cache_max_size.store(size, Ordering::SeqCst);
    }

    fn get_key_throttle_ms(&self) -> u128 {
        self.key_throttle_ms.load(Ordering::SeqCst) as u128
    }

    fn set_key_throttle_ms(&self, ms: u64) {
        self.key_throttle_ms.store(ms, Ordering::SeqCst);
    }
}

/// 应用状态
struct AppState {
    /// 库加载器
    library_loader: Rc<Mutex<Option<crate::formats::LibraryLoader>>>,
    /// 缩略图缓存
    thumbnail_cache: Rc<Mutex<Option<Arc<ThumbnailCache>>>>,
    /// 上次按键时间（用于节流）
    last_key_time: Rc<Mutex<Instant>>,
    /// 应用设置
    settings: Rc<AppSettings>,
}

impl AppState {
    fn new() -> Self {
        Self {
            library_loader: Rc::new(Mutex::new(None)),
            thumbnail_cache: Rc::new(Mutex::new(None)),
            last_key_time: Rc::new(Mutex::new(Instant::now())),
            settings: Rc::new(AppSettings::new()),
        }
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

/// 缩略图缓存（LRU 策略）
struct ThumbnailCache {
    /// 缓存的缩略图（索引 -> 图像）
    cache: Mutex<HashMap<usize, slint::Image>>,
    /// LRU 访问顺序（最近使用的在末尾）
    access_order: Mutex<Vec<usize>>,
    /// 总图片数
    total_count: usize,
    /// 正在加载的索引集合
    loading: Mutex<std::collections::HashSet<usize>>,
    /// 已加载数量
    loaded_count: AtomicU32,
    /// 应用设置引用
    settings: Rc<AppSettings>,
}

impl ThumbnailCache {
    fn new(total_count: usize, settings: Rc<AppSettings>) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            access_order: Mutex::new(Vec::new()),
            total_count,
            loading: Mutex::new(std::collections::HashSet::new()),
            loaded_count: AtomicU32::new(0),
            settings,
        }
    }

    /// 获取缓存的缩略图
    fn get(&self, index: usize) -> Option<slint::Image> {
        let cache = self.cache.lock().unwrap();
        if let Some(img) = cache.get(&index) {
            // 更新 LRU 顺序
            let mut order = self.access_order.lock().unwrap();
            order.retain(|&i| i != index);
            order.push(index);
            return Some(img.clone());
        }
        None
    }

    /// 插入缩略图到缓存
    fn put(&self, index: usize, image: slint::Image) {
        let mut cache = self.cache.lock().unwrap();
        let mut order = self.access_order.lock().unwrap();

        // 如果已存在，先移除
        if cache.contains_key(&index) {
            order.retain(|&i| i != index);
        }

        // 如果缓存已满，移除最久未使用的（使用动态配置的 max_size）
        let max_size = self.settings.get_cache_max_size();
        if cache.len() >= max_size
            && let Some(old_index) = order.first().copied()
        {
            cache.remove(&old_index);
            order.remove(0);
        }

        cache.insert(index, image);
        order.push(index);
        self.loaded_count.fetch_add(1, Ordering::SeqCst);
        tracing::trace!("缓存缩略图: {}, 缓存大小: {}", index, cache.len());
    }

    /// 请求加载指定范围的缩略图（使用共享加载器）
    fn request_range_with_loader(
        &self,
        start: usize,
        end: usize,
        window_weak: slint::Weak<AppWindow>,
        library_loader: Rc<Mutex<Option<crate::formats::LibraryLoader>>>,
    ) {
        let start = start.min(self.total_count.saturating_sub(1));
        let end = end.min(self.total_count.saturating_sub(1));

        if start > end {
            return;
        }

        // 找出需要加载的索引
        let indices_to_load: Vec<usize> = {
            let cache = self.cache.lock().unwrap();
            let mut loading = self.loading.lock().unwrap();
            tracing::debug!("缓存大小: {}, 正在加载: {}", cache.len(), loading.len());
            let indices: Vec<usize> = (start..=end)
                .filter(|&i| !cache.contains_key(&i) && !loading.contains(&i))
                .collect();
            for &i in &indices {
                loading.insert(i);
            }
            indices
        };

        if indices_to_load.is_empty() {
            return;
        }

        tracing::debug!(
            "请求加载缩略图: {}..{} ({} 张)",
            start,
            end,
            indices_to_load.len()
        );

        // 直接在主线程同步加载（避免每次重新打开文件）
        if let Some(win) = window_weak.upgrade() {
            let mut loader_guard = library_loader.lock().unwrap();
            if let Some(ref mut loader) = *loader_guard {
                let thumbnails = win.get_thumbnails();
                let mut new_thumbnails: Vec<slint::Image> = thumbnails.iter().collect();

                for i in &indices_to_load {
                    match loader.get_preview(*i) {
                        Ok(Some(preview_img)) => {
                            if let Some(slint_image) = rgba_image_to_slint(&preview_img)
                                && *i < new_thumbnails.len()
                            {
                                new_thumbnails[*i] = slint_image.clone();
                                // 存入缓存，避免重复加载
                                self.put(*i, slint_image);
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            tracing::warn!("加载缩略图 {} 失败: {:?}", i, e);
                        }
                    }
                }

                let model = slint::VecModel::from(new_thumbnails);
                win.set_thumbnails(slint::ModelRc::new(model));
                win.set_loaded_count(self.get_loaded_count() as i32);
            }

            // 清除加载标记
            let mut loading = self.loading.lock().unwrap();
            for i in &indices_to_load {
                loading.remove(i);
            }
        }
    }

    /// 获取已加载数量
    fn get_loaded_count(&self) -> usize {
        self.cache.lock().unwrap().len()
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
        let thumbnail_cache = state.thumbnail_cache.clone();
        let settings = state.settings.clone();

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

                    // 初始化空的缩略图数组
                    let empty_thumbnails: Vec<slint::Image> =
                        vec![slint::Image::default(); info.image_count];
                    let model = slint::VecModel::from(empty_thumbnails);
                    window.set_thumbnails(slint::ModelRc::new(model));
                    window.set_loaded_count(0);

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

                    // 创建缩略图缓存
                    let cache = Arc::new(ThumbnailCache::new(info.image_count, settings.clone()));

                    // 保存引用
                    *library_loader.lock().unwrap() = Some(loader);
                    *thumbnail_cache.lock().unwrap() = Some(Arc::clone(&cache));

                    window.set_status_text(SharedString::from(&format!(
                        "已打开: {} ({} 张图像)",
                        info.file_name, info.image_count
                    )));
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
        let _library_loader = state.library_loader.clone();

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

    // 设置请求缩略图回调（懒加载）
    {
        let window_weak = window_weak.clone();
        let thumbnail_cache = state.thumbnail_cache.clone();
        let library_loader = state.library_loader.clone();

        window.on_request_thumbnails(move |start, end| {
            let start = start as usize;
            let end = end as usize;

            tracing::debug!("请求缩略图: {} - {}", start, end);

            // 使用缓存的加载器加载缩略图
            if let Some(ref cache) = *thumbnail_cache.lock().unwrap() {
                cache.request_range_with_loader(
                    start,
                    end,
                    window_weak.clone(),
                    library_loader.clone(),
                );
            }
        });
    }

    // 设置键盘事件回调 - 在 Rust 端处理导航
    {
        let window_weak = window_weak.clone();
        let library_loader = state.library_loader.clone();
        let last_key_time = state.last_key_time.clone();
        let settings = state.settings.clone();

        window.on_key_pressed(move |text| {
            // 节流检查：使用动态配置的间隔
            {
                let mut last_time = last_key_time.lock().unwrap();
                let elapsed = last_time.elapsed().as_millis();
                let throttle_ms = settings.get_key_throttle_ms();
                if elapsed < throttle_ms {
                    return; // 忽略过快的按键
                }
                *last_time = Instant::now();
            }

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

    // 设置保存设置回调
    {
        let settings = state.settings.clone();

        window.on_save_settings(move |cache_max_size, key_throttle_ms| {
            settings.set_cache_max_size(cache_max_size as u64);
            settings.set_key_throttle_ms(key_throttle_ms as u64);
            tracing::info!(
                "设置已更新: cache_max_size={}, key_throttle_ms={}",
                cache_max_size, key_throttle_ms
            );
        });
    }

    tracing::debug!("运行主窗口");
    window
        .run()
        .map_err(|e| LibraryError::Gui(format!("运行窗口失败: {:?}", e)))?;

    Ok(())
}
