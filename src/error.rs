//! 错误类型定义

use thiserror::Error;

/// 库编辑器错误类型
#[derive(Error, Debug)]
pub enum LibraryError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("图片解码错误: {0}")]
    ImageDecode(#[from] image::ImageError),

    #[error("GUI 错误: {0}")]
    Gui(String),

    #[error("压缩/解压缩错误: {0}")]
    Compression(String),

    #[error("无效的文件格式")]
    InvalidFormat,

    #[error("不支持的版本: {0}")]
    UnsupportedVersion(i32),

    #[error("索引超出范围: {0}")]
    IndexOutOfBounds(usize),

    #[error("文件未找到: {0}")]
    FileNotFound(String),

    #[error("无效的图片数据")]
    InvalidImageData,

    #[error("解析错误: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, LibraryError>;
