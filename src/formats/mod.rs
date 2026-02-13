//! 库文件格式解析模块

pub mod mlibrary_v0;
pub mod mlibrary_v1;
pub mod mlibrary_v2;
pub mod wemade_library;
pub mod wtl_library;

pub use mlibrary_v1::MImage;
pub use mlibrary_v2::MLibraryV2;

use crate::error::{LibraryError, Result};
use crate::formats::mlibrary_v1::MLibraryV1;
use std::path::Path;

/// 库文件类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryType {
    /// MLibrary V0 (.wil 旧格式)
    MLV0,
    /// MLibrary V1 (.wzl/.wzx)
    MLV1,
    /// MLibrary V2 (.Lib)
    MLV2,
    /// WeMade Library (.wil/.wix)
    WeMade,
    /// WTL Library
    WTL,
}

impl LibraryType {
    /// 从文件扩展名识别库类型
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            ".wzl" | ".wzx" => Some(LibraryType::MLV1),
            ".lib" => Some(LibraryType::MLV2),
            ".wil" | ".wix" => Some(LibraryType::WeMade),
            ".wtl" => Some(LibraryType::WTL),
            _ => None,
        }
    }

    /// 获取主文件扩展名
    pub fn main_extension(&self) -> &str {
        match self {
            LibraryType::MLV1 => ".wzl",
            LibraryType::MLV2 => ".Lib",
            LibraryType::WeMade => ".wil",
            LibraryType::WTL => ".wtl",
            LibraryType::MLV0 => ".wil",
        }
    }

    /// 获取索引文件扩展名
    pub fn index_extension(&self) -> Option<&str> {
        match self {
            LibraryType::MLV1 => Some(".wzx"),
            LibraryType::WeMade => Some(".wix"),
            _ => None,
        }
    }

    /// 获取格式名称
    pub fn name(&self) -> &str {
        match self {
            LibraryType::MLV0 => "MLibrary V0",
            LibraryType::MLV1 => "MLibrary V1",
            LibraryType::MLV2 => "MLibrary V2",
            LibraryType::WeMade => "WeMade Library",
            LibraryType::WTL => "WTL Library",
        }
    }
}

/// 库文件信息（用于GUI显示）
#[derive(Debug, Clone)]
pub struct LibraryInfo {
    /// 文件路径（不含扩展名）
    pub base_path: String,
    /// 文件名
    pub file_name: String,
    /// 库类型
    pub library_type: LibraryType,
    /// 图像总数
    pub image_count: usize,
    /// 当前选中的索引
    pub current_index: i32,
}

impl LibraryInfo {
    /// 创建新的库信息
    pub fn new(
        base_path: String,
        file_name: String,
        library_type: LibraryType,
        image_count: usize,
    ) -> Self {
        Self {
            base_path,
            file_name,
            library_type,
            image_count,
            current_index: -1,
        }
    }

    /// 获取格式名称字符串
    pub fn format_name(&self) -> String {
        self.library_type.name().to_string()
    }
}

/// 图像信息（用于GUI显示）
/// 注意：由于每个版本有独立的 MImage 结构，这个通用结构已弃用
/// 请使用各版本特定的 MImage 结构
#[derive(Debug, Clone)]
pub struct ImageInfo {
    /// 索引
    pub index: usize,
    /// 宽度
    pub width: i32,
    /// 高度
    pub height: i32,
    /// X偏移
    pub x: i32,
    /// Y偏移
    pub y: i32,
    /// 是否有遮罩
    pub has_mask: ShadowInfo,
}

/// 遮罩信息
#[derive(Debug, Clone)]
pub enum ShadowInfo {
    None,
    Simple {
        shadow: u8,
        shadow_x: i16,
        shadow_y: i16,
    },
    Mask {
        shadow: u8,
        shadow_x: i16,
        shadow_y: i16,
        mask_width: i16,
        mask_height: i16,
    },
}

impl ImageInfo {
    /// 从 MLibraryV1::MImage 创建图像信息
    pub fn from_v1_image(index: usize, image: &mlibrary_v1::MImage) -> Self {
        Self {
            index,
            width: image.width as i32,
            height: image.height as i32,
            x: image.x as i32,
            y: image.y as i32,
            has_mask: ShadowInfo::None,
        }
    }

    /// 从 MLibraryV2::MImage 创建图像信息
    pub fn from_v2_image(index: usize, image: &mlibrary_v2::MImage) -> Self {
        let shadow_info = if image.has_mask {
            ShadowInfo::Mask {
                shadow: image.shadow,
                shadow_x: image.shadow_x,
                shadow_y: image.shadow_y,
                mask_width: image.mask_width,
                mask_height: image.mask_height,
            }
        } else {
            ShadowInfo::Simple {
                shadow: image.shadow,
                shadow_x: image.shadow_x,
                shadow_y: image.shadow_y,
            }
        };

        Self {
            index,
            width: image.width as i32,
            height: image.height as i32,
            x: image.x as i32,
            y: image.y as i32,
            has_mask: shadow_info,
        }
    }

    /// 获取尺寸字符串
    pub fn size_string(&self) -> String {
        format!("{} x {}", self.width, self.height)
    }
}

/// 库加载器 - 统一的库文件加载接口
pub struct LibraryLoader {
    /// 库信息
    info: Option<LibraryInfo>,
    library_v1: Option<MLibraryV1>,
    /// MLibrary V2 实例
    library_v2: Option<MLibraryV2>,
}

impl LibraryLoader {
    /// 创建新的加载器
    pub fn new() -> Self {
        Self {
            info: None,
            library_v1: None,
            library_v2: None,
        }
    }

    /// 从文件路径加载库
    pub fn load(path: &Path) -> Result<(LibraryInfo, Self)> {
        tracing::debug!("开始加载库文件: {:?}", path);
        tracing::debug!("文件存在: {}", path.exists());

        // 获取文件扩展名
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        tracing::debug!("文件扩展名: {}", extension);

        // 识别库类型
        let lib_type =
            LibraryType::from_extension(&format!(".{}", extension)).ok_or_else(|| {
                tracing::error!("不支持的文件格式: {}", extension);
                LibraryError::InvalidFormat
            })?;

        tracing::debug!("识别为格式: {}", lib_type.name());

        // 获取基础路径（去掉扩展名）
        let base_path = path
            .with_extension("")
            .to_str()
            .ok_or_else(|| LibraryError::ParseError("路径转换失败".to_string()))?
            .to_string();

        tracing::debug!("基础路径: {}", base_path);

        // 根据类型加载
        match lib_type {
            LibraryType::MLV1 => {
                tracing::debug!("使用 MLibrary V1 加载器");
                let library = MLibraryV1::new(base_path.clone())?;
                let count = library.count();

                tracing::debug!("成功加载 {count} 张图像");

                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                let info = LibraryInfo::new(base_path, file_name, lib_type, count);

                let mut loader = Self::new();
                loader.info = Some(info.clone());
                loader.library_v1 = Some(library);

                Ok((info, loader))
            }
            LibraryType::MLV2 => {
                tracing::debug!("使用 MLibrary V2 加载器");
                let library = MLibraryV2::new(base_path.clone())?;
                let count = library.count();

                tracing::debug!("成功加载 {} 张图像", count);

                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                let info = LibraryInfo::new(base_path, file_name, lib_type, count);

                let mut loader = Self::new();
                loader.info = Some(info.clone());
                loader.library_v2 = Some(library);

                Ok((info, loader))
            }
            _ => {
                tracing::error!("暂不支持此格式: {}", lib_type.name());
                Err(LibraryError::InvalidFormat)
            }
        }
    }

    /// 获取库信息
    pub fn info(&self) -> Option<&LibraryInfo> {
        self.info.as_ref()
    }

    /// 获取图像信息
    pub fn get_image_info(&mut self, index: usize) -> Result<ImageInfo> {
        tracing::debug!("获取图像信息: index={}", index);

        // 优先从 V2 获取
        if let Some(ref mut lib) = self.library_v2 {
            let image = lib.get_image(index)?;
            let info = ImageInfo::from_v2_image(index, image);
            tracing::debug!("图像信息: {}x{}, offset: ({}, {})", info.width, info.height, info.x, info.y);
            Ok(info)
        } else if let Some(ref mut lib) = self.library_v1 {
            // 从 V1 获取
            let image = lib.get_image(index)?;
            let info = ImageInfo::from_v1_image(index, image);
            tracing::debug!("图像信息: {}x{}, offset: ({}, {})", info.width, info.height, info.x, info.y);
            Ok(info)
        } else {
            Err(LibraryError::ParseError(
                "获取图像信息时异常：库未加载".to_string(),
            ))
        }
    }

    /// 获取图像预览
    pub fn get_preview(&mut self, index: usize) -> Result<Option<image::RgbaImage>> {
        tracing::debug!("获取图像预览: index={}", index);

        // 优先从 V2 获取
        if let Some(ref mut lib) = self.library_v2 {
            let preview = lib.get_preview(index)?.cloned();
            return Ok(preview);
        }

        // 从 V1 获取
        if let Some(ref mut lib) = self.library_v1 {
            let preview = lib.get_preview(index)?.cloned();
            return Ok(preview);
        }

        Err(LibraryError::ParseError(
            "获取图像预览时异常：库未加载".to_string(),
        ))
    }

    /// 获取图像数量
    pub fn image_count(&self) -> usize {
        self.info.as_ref().map(|i| i.image_count).unwrap_or(0)
    }

    /// 保存库
    pub fn save(&self) -> Result<()> {
        tracing::debug!("保存库文件");

        if let Some(ref lib) = self.library_v2 {
            lib.save()?;
            tracing::debug!("保存成功");
            Ok(())
        } else {
            Err(LibraryError::ParseError(
                "保存库文件时异常：库未加载".to_string(),
            ))
        }
    }

    /// 替换图像
    pub fn replace_image(
        &mut self,
        index: usize,
        image: &crate::formats::mlibrary_v2::MImage,
    ) -> Result<()> {
        tracing::debug!("替换图像: index={}", index);

        if let Some(ref mut lib) = self.library_v2 {
            lib.replace_image(index, image)?;
            tracing::debug!("替换成功");
            Ok(())
        } else {
            Err(LibraryError::ParseError(
                "替换图像时异常：库未加载".to_string(),
            ))
        }
    }

    /// 添加图像
    pub fn add_image(&mut self, image: &crate::formats::mlibrary_v2::MImage) -> Result<()> {
        tracing::debug!("添加新图像");

        if let Some(ref mut lib) = self.library_v2 {
            lib.add_image(image);
            tracing::debug!("添加成功");
            Ok(())
        } else {
            Err(LibraryError::ParseError(
                "添加图像时异常：库未加载".to_string(),
            ))
        }
    }

    /// 删除图像
    pub fn remove_image(&mut self, index: usize) -> Result<()> {
        tracing::debug!("删除图像: index={}", index);

        if let Some(ref mut lib) = self.library_v2 {
            lib.remove_image(index)?;
            tracing::debug!("删除成功");
            Ok(())
        } else {
            Err(LibraryError::ParseError(
                "删除图像时异常：库未加载".to_string(),
            ))
        }
    }

    /// 导出图像为 PNG
    pub fn export_png(&mut self, index: usize, path: &Path) -> Result<()> {
        tracing::debug!("导出图像为 PNG: index={}, path={:?}", index, path);

        if let Some(ref mut lib) = self.library_v2 {
            let preview = lib.get_preview(index)?;

            if let Some(img) = preview {
                img.save(path)?;
                tracing::debug!("导出成功");
                Ok(())
            } else {
                Err(LibraryError::InvalidImageData)
            }
        } else {
            Err(LibraryError::ParseError(
                "导出图像时异常：库未加载".to_string(),
            ))
        }
    }
}

impl Default for LibraryLoader {
    fn default() -> Self {
        Self::new()
    }
}
