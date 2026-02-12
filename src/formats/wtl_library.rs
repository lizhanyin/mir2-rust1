//! WTL Library 格式解析
//! 用于处理传奇2的 WTL 格式库文件

use crate::error::{Result, LibraryError};
use crate::image::MImage;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write, Seek, SeekFrom};
use std::path::Path;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

/// WTLLibrary - 用于处理 .wtl 文件
pub struct WTLLibrary {
    /// 文件名
    pub file_name: String,
    /// 图像列表
    pub images: Vec<Option<MImage>>,
    /// 索引列表
    pub index_list: Vec<u32>,
    /// 图像计数
    pub count: usize,
    /// 是否已初始化
    initialized: bool,
}

impl WTLLibrary {
    /// 创建新的 WTLLibrary 实例
    pub fn new(file_name: String) -> Result<Self> {
        let mut library = Self {
            file_name,
            images: Vec::new(),
            index_list: Vec::new(),
            count: 0,
            initialized: false,
        };

        library.initialize()?;
        Ok(library)
    }

    /// 初始化库
    pub fn initialize(&mut self) -> Result<()> {
        self.initialized = true;

        let wtl_path = format!("{}.wtl", self.file_name);

        if !Path::new(&wtl_path).exists() {
            return Err(LibraryError::FileNotFound(wtl_path));
        }

        // WTL 文件结构与 WIL 类似
        self.load_wtl_file(&wtl_path)?;

        // 初始化图像列表
        self.images = vec![None; self.index_list.len()];

        // 加载所有图像
        for i in 0..self.count {
            self.check_image(i)?;
        }

        Ok(())
    }

    /// 加载 WTL 文件
    fn load_wtl_file(&mut self, path: &str) -> Result<()> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // 读取文件头
        let mut header = [0u8; 4];
        reader.read_exact(&mut header)?;

        // 验证文件头
        if &header != b"WTL\x00" && &header != b"WTL\x01" {
            return Err(LibraryError::InvalidFormat);
        }

        // 读取图像计数
        self.count = reader.read_u32::<LittleEndian>()? as usize;

        // 读取所有索引
        self.index_list.clear();
        for _ in 0..self.count {
            let index = reader.read_u32::<LittleEndian>()?;
            self.index_list.push(index);
        }

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
        let wtl_path = format!("{}.wtl", self.file_name);
        let file = File::open(&wtl_path)?;
        let mut reader = BufReader::new(file);

        let offset = self.index_list[index] as u64;
        reader.seek(SeekFrom::Start(offset))?;

        let image = self.read_wtl_image(&mut reader)?;
        self.images[index] = Some(image);

        Ok(())
    }

    /// 读取 WTL 图像
    fn read_wtl_image(&self, reader: &mut BufReader<File>) -> Result<MImage> {
        // 读取图像头部
        let width = reader.read_i16::<LittleEndian>()?;
        let height = reader.read_i16::<LittleEndian>()?;
        let x = reader.read_i16::<LittleEndian>()?;
        let y = reader.read_i16::<LittleEndian>()?;
        let data_size = reader.read_i32::<LittleEndian>()?;

        let mut image = MImage::new();
        image.width = width;
        image.height = height;
        image.x = x;
        image.y = y;

        if data_size > 0 {
            let mut data = vec![0u8; data_size as usize];
            reader.read_exact(&mut data)?;

            // WTL 格式通常使用某种压缩
            image.create_texture(&data)?;
        }

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

    /// 保存库文件
    pub fn save(&self) -> Result<()> {
        let wtl_path = format!("{}.wtl", self.file_name);

        let file = File::create(&wtl_path)?;
        let mut writer = BufWriter::new(file);

        // 写入文件头
        writer.write_all(b"WTL\x00")?;

        // 写入图像计数
        writer.write_u32::<LittleEndian>(self.images.len() as u32)?;

        // 计算偏移量
        let data_offset = 8 + (self.images.len() * 4) as u32;
        let mut current_offset = data_offset;

        // 写入索引列表
        for _ in &self.images {
            writer.write_u32::<LittleEndian>(current_offset)?;
            // 偏移量会在写入数据时更新
            current_offset += 256; // 预估大小
        }

        writer.flush()?;
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
        let lib = WTLLibrary::new("test".to_string());
        assert!(lib.is_err()); // 文件不存在
    }
}
