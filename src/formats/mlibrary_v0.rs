//! MLibrary V0 格式解析 (.wil 旧格式)
//! 这是传奇1使用的旧版库文件格式

use crate::error::{LibraryError, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use image::{Rgba, RgbaImage};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// MLibrary V0 - 用于处理旧版 .wil 文件
pub struct MLibraryV0 {
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
    /// 是否加载图像
    pub load: bool,
}

/// MLibrary V0 的 MImage 结构
#[derive(Debug, Clone)]
pub struct MImage {
    /// 图像宽度
    pub width: i16,
    /// 图像高度
    pub height: i16,
    /// X 偏移
    pub x: i16,
    /// Y 偏移
    pub y: i16,
    /// 数据长度
    pub length: i32,
    /// 压缩后的图像数据 (DXT1)
    pub fbytes: Vec<u8>,
    /// 纹理是否有效
    pub texture_valid: bool,
    /// 解码后的图像
    pub image: Option<RgbaImage>,
    /// 预览图 (64x64)
    pub preview: Option<RgbaImage>,
}

impl MImage {
    /// 创建新的空白图像
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            x: 0,
            y: 0,
            length: 0,
            fbytes: Vec::new(),
            texture_valid: false,
            image: None,
            preview: None,
        }
    }

    /// 从位图创建 MImage
    pub fn from_image(img: &RgbaImage, x: i16, y: i16) -> Self {
        let width = img.width() as i16;
        let height = img.height() as i16;

        // 固定图像大小为4的倍数
        let fixed_width = width + (4 - width % 4) % 4;
        let fixed_height = height + (4 - height % 4) % 4;

        let mut fixed_image = img.clone();

        if fixed_width != width || fixed_height != height {
            fixed_image = RgbaImage::from_fn(fixed_width as u32, fixed_height as u32, |x, y| {
                if x < img.width() && y < img.height() {
                    *img.get_pixel(x, y)
                } else {
                    Rgba([0, 0, 0, 0])
                }
            });
        }

        // 转换为字节数组 (BGR 格式)
        let mut pixels = Vec::with_capacity((fixed_width * fixed_height * 4) as usize);
        for pixel in fixed_image.pixels() {
            let [r, g, b, a] = pixel.0;
            // 交换 R 和 B (BGR 格式)
            pixels.push(b);
            pixels.push(g);
            pixels.push(r);
            // 黑色像素设为透明
            pixels.push(if r == 0 && g == 0 && b == 0 { 0 } else { a });
        }

        // 注意: C# 使用 DXT1 压缩，这里简化为直接存储
        let fbytes = pixels;

        Self {
            width: fixed_width,
            height: fixed_height,
            x,
            y,
            length: fbytes.len() as i32,
            fbytes,
            texture_valid: true,
            image: Some(fixed_image),
            preview: None,
        }
    }

    /// 从字节数据创建纹理
    pub fn create_texture(&mut self) -> Result<()> {
        if self.width <= 0 || self.height <= 0 {
            return Err(LibraryError::InvalidImageData);
        }

        // 注意: C# 使用 DXT1 解压缩，这里简化处理
        // 直接将 fbytes 作为 RGBA 数据使用
        let expected_size = (self.width * self.height * 4) as usize;

        if self.fbytes.len() != expected_size {
            return Err(LibraryError::InvalidImageData);
        }

        let width = self.width as u32;
        let height = self.height as u32;

        let mut rgba_img = RgbaImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                if idx + 3 < self.fbytes.len() {
                    let b = self.fbytes[idx];
                    let g = self.fbytes[idx + 1];
                    let r = self.fbytes[idx + 2];
                    let a = self.fbytes[idx + 3];
                    rgba_img.put_pixel(x, height - 1 - y, Rgba([r, g, b, a]));
                }
            }
        }

        self.image = Some(rgba_img);
        self.texture_valid = true;
        Ok(())
    }

    /// 创建预览图 (64x64)
    pub fn create_preview(&mut self) {
        if let Some(ref image) = self.image {
            use image::imageops;

            let w = std::cmp::min(image.width(), 64);
            let h = std::cmp::min(image.height(), 64);

            let resized = imageops::resize(image, w, h, imageops::FilterType::Triangle);

            let preview = RgbaImage::from_fn(64, 64, |x, y| {
                let offset_x = (64 - w) / 2;
                let offset_y = (64 - h) / 2;
                if x >= offset_x && x < offset_x + w && y >= offset_y && y < offset_y + h {
                    *resized.get_pixel(x - offset_x, y - offset_y)
                } else {
                    Rgba([0, 0, 0, 0])
                }
            });

            self.preview = Some(preview);
        }
    }

    /// 获取预览图
    pub fn get_preview(&mut self) -> Option<&RgbaImage> {
        if self.preview.is_none() {
            self.create_preview();
        }
        self.preview.as_ref()
    }

    /// 保存图像数据
    pub fn save(&self, writer: &mut Vec<u8>) -> Result<()> {
        writer.write_i16::<LittleEndian>(self.width)?;
        writer.write_i16::<LittleEndian>(self.height)?;
        writer.write_i16::<LittleEndian>(self.x)?;
        writer.write_i16::<LittleEndian>(self.y)?;
        writer.write_i32::<LittleEndian>(self.length)?;
        writer.extend_from_slice(&self.fbytes);
        Ok(())
    }
}

impl Default for MImage {
    fn default() -> Self {
        Self::new()
    }
}

impl MLibraryV0 {
    /// 创建新的 MLibrary V0 实例
    pub fn new(file_name: String) -> Result<Self> {
        let mut library = Self {
            file_name,
            images: Vec::new(),
            index_list: Vec::new(),
            count: 0,
            initialized: false,
            load: true,
        };

        library.initialize()?;
        Ok(library)
    }

    /// 初始化库
    pub fn initialize(&mut self) -> Result<()> {
        self.initialized = true;

        let wil_path = format!("{}.wil", self.file_name);

        if !Path::new(&wil_path).exists() {
            return Ok(()); // 文件不存在时直接返回
        }

        let file = File::open(&wil_path)?;
        let mut reader = BufReader::new(file);

        // 读取图像数量
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
        self.load = false; // 转换时不需要处理所有图像
        for i in 0..self.count {
            self.check_image(i)?;
        }

        Ok(())
    }

    /// 关闭库
    pub fn close(&mut self) {
        self.initialized = false;
    }

    /// 检查并加载指定索引的图像
    pub fn check_image(&mut self, index: usize) -> Result<()> {
        if !self.initialized {
            self.initialize()?;
        }

        if index >= self.images.len() {
            return Ok(());
        }

        if self.images[index].is_none() {
            self.load_image(index)?;
        }

        if !self.load {
            return Ok(());
        }

        if let Some(ref mut img) = self.images[index] {
            if !img.texture_valid {
                // 需要创建纹理
            }
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

        let image = Self::read_mimage(&mut reader)?;
        self.images[index] = Some(image);

        Ok(())
    }

    /// 读取 MImage 数据
    fn read_mimage(reader: &mut BufReader<File>) -> Result<MImage> {
        let width = reader.read_i16::<LittleEndian>()?;
        let height = reader.read_i16::<LittleEndian>()?;
        let x = reader.read_i16::<LittleEndian>()?;
        let y = reader.read_i16::<LittleEndian>()?;
        let length = reader.read_i32::<LittleEndian>()?;

        let mut fbytes = vec![0u8; length as usize];
        reader.read_exact(&mut fbytes)?;

        let mut img = MImage::new();
        img.width = width;
        img.height = height;
        img.x = x;
        img.y = y;
        img.length = length;
        img.fbytes = fbytes;

        Ok(img)
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
        self.count += 1;
        self.images.push(Some(image.clone()));
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
        self.count += 1;
        self.images.insert(index, Some(image.clone()));
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
        let mut data_stream = Vec::new();
        let mut index_list: Vec<u32> = Vec::new();

        let offset = 4 + (self.images.len() * 4) as u32;

        for img in self.images.iter().flatten() {
            let current_offset = data_stream.len() as u32 + offset;
            index_list.push(current_offset);
            img.save(&mut data_stream)?;
        }

        // 写入文件
        let wil_path = format!("{}.wil", self.file_name);
        let file = File::create(&wil_path)?;
        let mut writer = BufWriter::new(file);

        writer.write_i32::<LittleEndian>(self.images.len() as i32)?;

        for index in &index_list {
            writer.write_i32::<LittleEndian>(*index as i32)?;
        }

        writer.write_all(&data_stream)?;
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
        let lib = MLibraryV0::new("test".to_string());
        assert!(lib.is_ok()); // 文件不存在时应该返回 Ok
    }

    #[test]
    fn test_mimage_creation() {
        let img = MImage::new();
        assert_eq!(img.width, 0);
        assert_eq!(img.height, 0);
    }
}
