//! WeMade Library 格式解析
//! 用于处理传奇2的 WeMade 格式库文件

use crate::error::{Result, LibraryError};
use crate::image::{MImage, Color};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use byteorder::{LittleEndian, ReadBytesExt};

/// WeMadLibrary - 用于处理 .wil/.wix 文件
pub struct WeMadeLibrary {
    /// 文件名（不带扩展名）
    pub file_name: String,
    /// 图像列表
    pub images: Vec<Option<WeMadeImage>>,
    /// 索引列表
    pub index_list: Vec<u32>,
    /// 图像计数
    pub count: usize,
    /// 是否已初始化
    initialized: bool,
    /// 库类型
    pub n_type: u8,
    /// 调色板
    palette: Vec<Color>,
    /// 版本号
    version: i32,
}

/// WeMade 图像结构
#[derive(Debug, Clone)]
pub struct WeMadeImage {
    /// 宽度
    pub width: i16,
    /// 高度
    pub height: i16,
    /// X 偏移
    pub x: i16,
    /// Y 偏移
    pub y: i16,
    /// 阴影 X 偏移
    pub shadow_x: i16,
    /// 阴影 Y 偏移
    pub shadow_y: i16,
    /// 是否有阴影
    pub has_shadow: bool,
    /// 是否 16 位颜色
    pub is_16bit: bool,
    /// 数据大小
    pub n_size: i32,
    /// 是否有遮罩
    pub has_mask: bool,
    /// 图像数据
    pub image_data: Option<image::RgbaImage>,
    /// 遮罩数据
    pub mask_data: Option<image::RgbaImage>,
}

impl WeMadeLibrary {
    /// 创建新的 WeMadeLibrary 实例
    pub fn new(file_name: String) -> Result<Self> {
        let mut library = Self {
            file_name,
            images: Vec::new(),
            index_list: Vec::new(),
            count: 0,
            initialized: false,
            n_type: 0,
            palette: Vec::new(),
            version: 0,
        };

        library.initialize()?;
        Ok(library)
    }

    /// 初始化库
    pub fn initialize(&mut self) -> Result<()> {
        self.initialized = true;

        let main_ext = if self.n_type == 1 { ".wzl" } else if self.n_type == 4 { ".miz" } else { ".wil" };
        let index_ext = if self.n_type == 1 { ".wzx" } else if self.n_type == 4 { ".mix" } else { ".wix" };

        let main_path = format!("{}{}", self.file_name, main_ext);
        let index_path = format!("{}{}", self.file_name, index_ext);

        if !Path::new(&index_path).exists() {
            return Err(LibraryError::FileNotFound(index_path));
        }

        if !Path::new(&main_path).exists() {
            return Err(LibraryError::FileNotFound(main_path));
        }

        // 加载图像信息
        self.load_image_info(&index_path)?;

        // 初始化图像列表
        self.images = vec![None; self.index_list.len()];

        // 加载所有图像
        for i in 0..self.count {
            self.check_image(i)?;
        }

        Ok(())
    }

    /// 加载图像信息
    fn load_image_info(&mut self, index_path: &str) -> Result<()> {
        // 设置默认调色板
        self.palette = crate::image::DEFAULT_PALETTE.to_vec();

        let file = File::open(index_path)?;
        let mut reader = BufReader::new(file);

        // 根据类型读取不同长度的头部
        match self.n_type {
            4 => {
                reader.seek(SeekFrom::Start(24))?;
            }
            3 => {
                let mut buf = [0u8; 26];
                reader.read_exact(&mut buf)?;
                if reader.read_u16::<LittleEndian>()? != 0xB13A {
                    reader.seek(SeekFrom::Start(24))?;
                }
            }
            2 => {
                reader.seek(SeekFrom::Start(52))?;
            }
            _ => {
                let skip = if self.version == 0 { 48 } else { 52 };
                reader.seek(SeekFrom::Start(skip as u64))?;
            }
        }

        // 读取所有索引
        self.index_list.clear();
        while let Ok(index) = reader.read_u32::<LittleEndian>() {
            self.index_list.push(index);
        }

        self.count = self.index_list.len();
        Ok(())
    }

    /// 检查并加载指定索引的图像
    pub fn check_image(&mut self, index: usize) -> Result<()> {
        if !self.initialized {
            self.initialize()?;
        }

        if index >= self.images.len() {
            return Err(LibraryError::IndexOutOfBounds(index));
        }

        if self.images[index].is_none() {
            self.load_image(index)?;
        }

        Ok(())
    }

    /// 加载指定索引的图像
    fn load_image(&mut self, index: usize) -> Result<()> {
        let main_ext = if self.n_type == 1 { ".wzl" } else if self.n_type == 4 { ".miz" } else { ".wil" };
        let main_path = format!("{}{}", self.file_name, main_ext);

        let file = File::open(&main_path)?;
        let mut reader = BufReader::new(file);

        let offset = self.index_list[index] as u64;
        reader.seek(SeekFrom::Start(offset))?;

        let image = self.read_wemade_image(&mut reader, offset)?;
        self.images[index] = Some(image);

        Ok(())
    }

    /// 读取 WeMade 图像
    fn read_wemade_image(&self, reader: &mut BufReader<File>, offset: u64) -> Result<WeMadeImage> {
        reader.seek(SeekFrom::Start(offset))?;

        let mut image = WeMadeImage {
            width: 0,
            height: 0,
            x: 0,
            y: 0,
            shadow_x: 0,
            shadow_y: 0,
            has_shadow: false,
            is_16bit: false,
            n_size: 0,
            has_mask: false,
            image_data: None,
            mask_data: None,
        };

        // 根据类型读取不同的头部
        match self.n_type {
            1 | 4 => {
                // WZL / MIZ 格式
                image.is_16bit = reader.read_u8()? == 5;
                reader.read_u8()?;
                reader.read_u8()?;
                reader.read_u8()?;
                image.width = reader.read_i16::<LittleEndian>()?;
                image.height = reader.read_i16::<LittleEndian>()?;
                image.x = reader.read_i16::<LittleEndian>()?;
                image.y = reader.read_i16::<LittleEndian>()?;
                image.n_size = reader.read_i32::<LittleEndian>()?;
            }
            _ => {
                // WIL 格式
                image.width = reader.read_i16::<LittleEndian>()?;
                image.height = reader.read_i16::<LittleEndian>()?;
                image.x = reader.read_i16::<LittleEndian>()?;
                image.y = reader.read_i16::<LittleEndian>()?;
                image.n_size = (image.width * image.height) as i32;
            }
        }

        Ok(image)
    }

    /// 获取指定索引的图像
    pub fn get_image(&mut self, index: usize) -> Result<&WeMadeImage> {
        self.check_image(index)?;

        self.images[index]
            .as_ref()
            .ok_or_else(|| LibraryError::IndexOutOfBounds(index))
    }

    /// 转换为 MLibraryV2
    pub fn to_mlibrary_v2(&self) -> Result<super::MLibraryV2> {
        let mut library = super::MLibraryV2::new(self.file_name.clone())?;

        for img_opt in &self.images {
            if let Some(_wemade_img) = img_opt {
                // 转换 WeMadeImage 到 MImage
                let m_image = MImage::new();
                library.add_image(&m_image);
            }
        }

        Ok(library)
    }

    /// 获取图像计数
    pub fn count(&self) -> usize {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_library() {
        let lib = WeMadeLibrary::new("test".to_string());
        assert!(lib.is_err()); // 文件不存在
    }
}
