//! MLibrary V0 格式解析 (.wil 旧格式)
//! 这是传奇1使用的旧版库文件格式

use crate::error::{Result, LibraryError};
use crate::image::{MImage, Color, DEFAULT_PALETTE};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use byteorder::{LittleEndian, ReadBytesExt};

/// MLibrary V0 - 用于处理旧版 .wil 文件
pub struct MLibraryV0 {
    /// 文件名（不带扩展名）
    pub file_name: String,
    /// 图像列表
    pub images: Vec<Option<MImage>>,
    /// 索引列表
    pub index_list: Vec<u32>,
    /// 图像计数
    pub count: usize,
    /// 是否已初始化
    initialized: bool,
    /// 版本号
    version: i32,
    /// 调色板
    palette: Vec<Color>,
}

impl MLibraryV0 {
    pub const LIB_VERSION: i32 = 0;
    const WIX_HEADER_SIZE: u64 = 48;

    /// 创建新的 MLibrary V0 实例
    pub fn new(file_name: String) -> Result<Self> {
        let mut library = Self {
            file_name,
            images: Vec::new(),
            index_list: Vec::new(),
            count: 0,
            initialized: false,
            version: Self::LIB_VERSION,
            palette: DEFAULT_PALETTE.to_vec(),
        };

        library.initialize()?;
        Ok(library)
    }

    /// 初始化库
    pub fn initialize(&mut self) -> Result<()> {
        self.initialized = true;

        let wil_path = format!("{}.wil", self.file_name);
        let wix_path = format!("{}.wix", self.file_name);

        if !Path::new(&wix_path).exists() {
            return Err(LibraryError::FileNotFound(wix_path));
        }

        if !Path::new(&wil_path).exists() {
            return Err(LibraryError::FileNotFound(wil_path));
        }

        // 读取索引文件 (.wix)
        self.load_index_file(&wix_path)?;

        // 初始化图像列表
        self.images = vec![None; self.index_list.len()];

        Ok(())
    }

    /// 加载索引文件
    fn load_index_file(&mut self, path: &str) -> Result<()> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // 读取调色板大小
        let palette_size = reader.read_u32::<LittleEndian>()?;

        if palette_size > 256 {
            return Err(LibraryError::InvalidFormat);
        }

        // 跳过 4 字节
        reader.read_u32::<LittleEndian>()?;

        // 读取版本号
        self.version = reader.read_i32::<LittleEndian>()?;

        // 根据版本跳过一些字节
        if self.version == 0 {
            // 跳过 0 字节
        } else {
            reader.read_u32::<LittleEndian>()?;
        }

        // 读取调色板
        self.palette = DEFAULT_PALETTE.to_vec();
        for i in 1..palette_size as usize {
            let color_val = reader.read_u32::<LittleEndian>()?;
            let color = Color {
                a: 255,
                r: ((color_val >> 16) & 0xFF) as u8,
                g: ((color_val >> 8) & 0xFF) as u8,
                b: (color_val & 0xFF) as u8,
            };
            if i < self.palette.len() {
                self.palette[i] = color;
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
        let wil_path = format!("{}.wil", self.file_name);
        let file = File::open(&wil_path)?;
        let mut reader = BufReader::new(file);

        let offset = self.index_list[index] as u64;
        reader.seek(SeekFrom::Start(offset))?;

        let image = self.read_mimage(&mut reader)?;
        self.images[index] = Some(image);

        Ok(())
    }

    /// 读取 MImage 数据
    fn read_mimage(&self, reader: &mut BufReader<File>) -> Result<MImage> {
        // 读取基本信息
        let width = reader.read_u16::<LittleEndian>()? as i16;
        let height = reader.read_u16::<LittleEndian>()? as i16;
        let x = reader.read_u16::<LittleEndian>()? as i16;
        let y = reader.read_u16::<LittleEndian>()? as i16;

        let pixel_data_size = (width * height) as usize;
        let mut pixel_data = vec![0u8; pixel_data_size];
        reader.read_exact(&mut pixel_data)?;

        // 创建图像
        let mut image = MImage::new();
        image.width = width;
        image.height = height;
        image.x = x;
        image.y = y;

        // 将调色板索引转换为像素数据
        let mut rgba_data = Vec::with_capacity(width as usize * height as usize * 4);

        for idx in &pixel_data {
            let color = self.palette.get(*idx as usize).copied()
                .unwrap_or(Color { a: 255, r: 0, g: 0, b: 0 });
            rgba_data.push(color.r);
            rgba_data.push(color.g);
            rgba_data.push(color.b);
            rgba_data.push(color.a);
        }

        image.create_texture(&rgba_data)?;

        Ok(image)
    }

    /// 获取指定索引的图像
    pub fn get_image(&mut self, index: usize) -> Result<&MImage> {
        self.check_image(index)?;

        self.images[index]
            .as_ref()
            .ok_or_else(|| LibraryError::IndexOutOfBounds(index))
    }

    /// 添加新图像
    pub fn add_image(&mut self, image: &MImage) {
        self.images.push(Some(image.clone()));
        self.count += 1;
    }

    /// 替换图像
    pub fn replace_image(&mut self, index: usize, image: &MImage) -> Result<()> {
        if index >= self.images.len() {
            return Err(LibraryError::IndexOutOfBounds(index));
        }
        self.images[index] = Some(image.clone());
        Ok(())
    }

    /// 删除图像
    pub fn remove_image(&mut self, index: usize) -> Result<()> {
        if self.images.len() <= 1 {
            self.images.clear();
            self.count = 0;
            return Ok(());
        }

        if index >= self.images.len() {
            return Err(LibraryError::IndexOutOfBounds(index));
        }

        self.images.remove(index);
        self.count -= 1;
        Ok(())
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
        let lib = MLibraryV0::new("test".to_string());
        assert!(lib.is_err()); // 文件不存在
    }
}
