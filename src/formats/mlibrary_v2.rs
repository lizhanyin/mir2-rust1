//! MLibrary V2 格式解析 (.Lib)
//! 这是传奇2使用的自定义库文件格式

use crate::error::{Result, LibraryError};
use crate::image::{MImage, decompress_gzip};
use crate::formats::mlibrary_v1::MLibraryV1;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write, Seek, SeekFrom};
use std::path::Path;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

/// MLibrary V2 - 用于处理 .Lib 文件
pub struct MLibraryV2 {
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
    /// 版本号
    version: i32,
}

impl MLibraryV2 {
    pub const LIB_VERSION: i32 = 2;

    /// 创建新的 MLibrary V2 实例
    pub fn new(file_name: String) -> Result<Self> {
        let mut library = Self {
            file_name,
            images: Vec::new(),
            index_list: Vec::new(),
            count: 0,
            initialized: false,
            version: Self::LIB_VERSION,
        };

        library.initialize()?;
        Ok(library)
    }

    /// 从 MLibraryV1 转换
    pub fn from_v1(v1: &MLibraryV1) -> Result<Self> {
        let mut v2 = Self {
            file_name: v1.file_name.clone(),
            images: Vec::new(),
            index_list: Vec::new(),
            count: 0,
            initialized: false,
            version: Self::LIB_VERSION,
        };

        // 转换图像数据
        for img_opt in &v1.images {
            if let Some(img) = img_opt {
                v2.images.push(Some(img.clone()));
            } else {
                v2.images.push(None);
            }
        }

        v2.count = v2.images.len();
        Ok(v2)
    }

    /// 初始化库
    pub fn initialize(&mut self) -> Result<()> {
        self.initialized = true;

        let lib_path = format!("{}.Lib", self.file_name);

        if !Path::new(&lib_path).exists() {
            return Err(LibraryError::FileNotFound(lib_path));
        }

        // 打开文件并读取头部
        let file = File::open(&lib_path)?;
        let mut reader = BufReader::new(file);

        // 读取版本号
        self.version = reader.read_i32::<LittleEndian>()?;

        if self.version != Self::LIB_VERSION {
            return Err(LibraryError::UnsupportedVersion(self.version));
        }

        // 读取图像计数
        self.count = reader.read_i32::<LittleEndian>()? as usize;

        // 读取索引列表
        self.index_list.clear();
        for _ in 0..self.count {
            let index = reader.read_u32::<LittleEndian>()?;
            self.index_list.push(index);
        }

        // 初始化图像列表
        self.images = vec![None; self.count];

        // 加载所有图像
        for i in 0..self.count {
            self.check_image(i)?;
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
        let lib_path = format!("{}.Lib", self.file_name);
        let file = File::open(&lib_path)?;
        let mut reader = BufReader::new(file);

        let offset = self.index_list[index] as u64;
        reader.seek(SeekFrom::Start(offset))?;

        let image = self.read_mimage(&mut reader)?;
        self.images[index] = Some(image);

        Ok(())
    }

    /// 读取 MImage 数据
    fn read_mimage(&self, reader: &mut BufReader<File>) -> Result<MImage> {
        // 读取 Layer 1
        let width = reader.read_i16::<LittleEndian>()?;
        let height = reader.read_i16::<LittleEndian>()?;
        let x = reader.read_i16::<LittleEndian>()?;
        let y = reader.read_i16::<LittleEndian>()?;
        let shadow_x = reader.read_i16::<LittleEndian>()?;
        let shadow_y = reader.read_i16::<LittleEndian>()?;
        let shadow = reader.read_u8()?;
        let length = reader.read_i32::<LittleEndian>()?;

        let mut fbytes = vec![0u8; length as usize];
        reader.read_exact(&mut fbytes)?;

        // 检查是否有 Layer 2 (Mask)
        let has_mask = (shadow >> 7) == 1;

        let mut image = MImage::new();
        image.width = width;
        image.height = height;
        image.x = x;
        image.y = y;
        image.shadow_x = shadow_x;
        image.shadow_y = shadow_y;
        image.shadow = shadow & 0x7F; // 移除最高位
        image.fbytes = fbytes.clone();
        image.has_mask = has_mask;

        // 解压并创建图像
        image.create_texture(&fbytes)?;

        if has_mask {
            image.mask_width = reader.read_i16::<LittleEndian>()?;
            image.mask_height = reader.read_i16::<LittleEndian>()?;
            image.mask_x = reader.read_i16::<LittleEndian>()?;
            image.mask_y = reader.read_i16::<LittleEndian>()?;
            let mask_length = reader.read_i32::<LittleEndian>()?;

            image.mask_fbytes = vec![0u8; mask_length as usize];
            reader.read_exact(&mut image.mask_fbytes)?;

            // 创建遮罩图像
            if let Ok(_mask_data) = decompress_gzip(&image.mask_fbytes) {
                // 处理遮罩数据...
            }
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

    /// 获取预览图
    pub fn get_preview(&mut self, index: usize) -> Result<Option<image::RgbaImage>> {
        self.check_image(index)?;

        if let Some(ref mut img) = self.images[index] {
            Ok(img.get_preview().cloned())
        } else {
            Ok(None)
        }
    }

    /// 添加新图像
    pub fn add_image(&mut self, image: &MImage) {
        self.images.push(Some(image.clone()));
        self.count += 1;
    }

    /// 添加带遮罩的图像
    pub fn add_image_with_mask(&mut self, image: &MImage, mask_image: &MImage) {
        let mut new_image = image.clone();
        new_image.has_mask = true;
        new_image.mask_width = mask_image.width;
        new_image.mask_height = mask_image.height;
        new_image.mask_fbytes = mask_image.fbytes.clone();

        self.images.push(Some(new_image));
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

    /// 插入图像
    pub fn insert_image(&mut self, index: usize, image: &MImage) -> Result<()> {
        if index > self.images.len() {
            return Err(LibraryError::IndexOutOfBounds(index));
        }
        self.images.insert(index, Some(image.clone()));
        self.count += 1;
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
        let lib_path = format!("{}.Lib", self.file_name);

        // 计算偏移量
        let header_size = 8 + (self.images.len() * 4) as u32;
        let mut offset = header_size;
        let mut index_list: Vec<u32> = Vec::new();
        let mut data: Vec<u8> = Vec::new();

        // 写入所有图像数据
        for image in &self.images {
            if let Some(img) = image {
                index_list.push(offset);

                // 写入图像数据
                Self::write_mimage_data(img, &mut data)?;

                offset += data.len() as u32;
            }
        }

        // 写入文件
        {
            let file = File::create(&lib_path)?;
            let mut writer = BufWriter::new(file);

            writer.write_i32::<LittleEndian>(Self::LIB_VERSION)?;
            writer.write_i32::<LittleEndian>(self.images.len() as i32)?;

            for index in &index_list {
                writer.write_u32::<LittleEndian>(*index)?;
            }

            writer.write_all(&data)?;
            writer.flush()?;
        }

        Ok(())
    }

    /// 写入 MImage 数据
    fn write_mimage_data(image: &MImage, writer: &mut Vec<u8>) -> Result<()> {
        use byteorder::WriteBytesExt;

        writer.write_i16::<LittleEndian>(image.width)?;
        writer.write_i16::<LittleEndian>(image.height)?;
        writer.write_i16::<LittleEndian>(image.x)?;
        writer.write_i16::<LittleEndian>(image.y)?;
        writer.write_i16::<LittleEndian>(image.shadow_x)?;
        writer.write_i16::<LittleEndian>(image.shadow_y)?;

        let shadow_byte = if image.has_mask {
            image.shadow | 0x80
        } else {
            image.shadow
        };
        writer.write_u8(shadow_byte)?;

        writer.write_i32::<LittleEndian>(image.fbytes.len() as i32)?;
        writer.write_all(&image.fbytes)?;

        if image.has_mask {
            writer.write_i16::<LittleEndian>(image.mask_width)?;
            writer.write_i16::<LittleEndian>(image.mask_height)?;
            writer.write_i16::<LittleEndian>(image.mask_x)?;
            writer.write_i16::<LittleEndian>(image.mask_y)?;
            writer.write_i32::<LittleEndian>(image.mask_fbytes.len() as i32)?;
            writer.write_all(&image.mask_fbytes)?;
        }

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
        let lib = MLibraryV2::new("test".to_string());
        assert!(lib.is_err()); // 文件不存在
    }
}
