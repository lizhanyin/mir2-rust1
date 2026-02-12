//! 库文件格式解析模块

pub mod mlibrary_v1;
pub mod mlibrary_v2;
pub mod mlibrary_v0;
pub mod wemade_library;
pub mod wtl_library;

pub use mlibrary_v2::MLibraryV2;

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
}
