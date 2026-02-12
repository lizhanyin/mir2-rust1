//! GUI 模块 - 使用 slint 框架

use slint::ModelRc;
use std::rc::Rc;
use std::path::PathBuf;
use crate::error::Result;
use crate::formats::{MLibraryV1, MLibraryV2, LibraryType};

/// 应用状态
pub struct AppState {
    /// 当前打开的文件路径
    pub file_path: Option<PathBuf>,
    /// 当前库类型
    pub library_type: Option<LibraryType>,
    /// MLibrary V1 实例
    pub ml_v1: Option<MLibraryV1>,
    /// MLibrary V2 实例
    pub ml_v2: Option<MLibraryV2>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            file_path: None,
            library_type: None,
            ml_v1: None,
            ml_v2: None,
        }
    }
}

/// 主窗口控制器
pub struct MainWindow {
    /// slint 窗口句柄
    window: slint::Weak<AppWindow>,
    /// 应用状态
    state: Rc<std::cell::RefCell<AppState>>,
}

impl MainWindow {
    /// 创建新的主窗口
    pub fn new() -> Result<Self> {
        // 创建 slint 窗口
        let window = AppWindow::new()?;

        let state = Rc::new(std::cell::RefCell::new(AppState::default()));

        // 设置窗口引用和状态
        let weak_window = window.as_weak();
        let state_clone = state.clone();

        // 处理打开文件回调
        let window_weak = weak_window.clone();
        window.on_open_file(move || {
            if let Some(path) = Self::show_open_dialog() {
                Self::handle_file_open(window_weak.clone(), state_clone.clone(), path);
            }
        });

        // 处理保存文件回调
        let window_weak = weak_window.clone();
        let state_clone = state.clone();
        window.on_save_file(move || {
            let state = state_clone.borrow();
            if let Some(ref path) = state.file_path {
                let _ = Self::save_library(window_weak.clone(), state_clone.clone(), path);
            }
        });

        // 处理图像选择回调
        let window_weak = weak_window.clone();
        window.on_image_selected(move |index| {
            window_weak.set_current_image(index as i32);
        });

        Ok(Self {
            window: weak_window,
            state,
        })
    }

    /// 显示打开文件对话框
    fn show_open_dialog() -> Option<PathBuf> {
        use rfd::FileDialog;

        FileDialog::new()
            .add_filter("库文件", &["Lib", "wil", "wzl", "wix", "wtl"])
            .set_title("打开库文件")
            .pick_file()
    }

    /// 处理文件打开
    fn handle_file_open(
        window: slint::Weak<AppWindow>,
        state: Rc<std::cell::RefCell<AppState>>,
        path: PathBuf,
    ) {
        // 获取文件扩展名
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        // 识别库类型
        let lib_type = match LibraryType::from_extension(extension) {
            Some(t) => t,
            None => {
                window.set_status_text("不支持的文件格式".into());
                return;
            }
        };

        // 更新状态
        {
            let mut state = state.borrow_mut();
            state.file_path = Some(path.clone());
            state.library_type = Some(lib_type);
        }

        // 根据类型加载库
        match lib_type {
            LibraryType::MLV1 => {
                let file_stem = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                match MLibraryV1::new(file_stem) {
                    Ok(library) => {
                        let mut state = state.borrow_mut();
                        state.ml_v1 = Some(library);
                        state.ml_v2 = None;
                        window.set_status_text("文件加载成功".into());
                        window.set_file_name(path.to_string_lossy().to_string().into());
                    }
                    Err(e) => {
                        window.set_status_text(format!("加载失败: {}", e).into());
                    }
                }
            }
            LibraryType::MLV2 => {
                let file_stem = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                match MLibraryV2::new(file_stem) {
                    Ok(library) => {
                        let mut state = state.borrow_mut();
                        state.ml_v2 = Some(library);
                        state.ml_v1 = None;
                        window.set_status_text("文件加载成功".into());
                        window.set_file_name(path.to_string_lossy().to_string().into());
                    }
                    Err(e) => {
                        window.set_status_text(format!("加载失败: {}", e).into());
                    }
                }
            }
            _ => {
                window.set_status_text("暂不支持此格式".into());
            }
        }
    }

    /// 保存库文件
    fn save_library(
        window: slint::Weak<AppWindow>,
        state: Rc<std::cell::RefCell<AppState>>,
        _path: &PathBuf,
    ) -> Result<()> {
        let state = state.borrow();

        if let Some(ref lib) = state.ml_v1 {
            lib.save()?;
            window.set_status_text("保存成功".into());
        } else if let Some(ref lib) = state.ml_v2 {
            lib.save()?;
            window.set_status_text("保存成功".into());
        }

        Ok(())
    }

    /// 显示窗口
    pub fn show(&self) {
        self.window.show().unwrap();
    }

    /// 运行事件循环
    pub fn run(&self) {
        self.window.run().unwrap();
    }
}

// slint 组件生成的代码将在编译时生成
slint::slint! {
    #[include = str_replace_path!("ui/app_window.slint")]
}
