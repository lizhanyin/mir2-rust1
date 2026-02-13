//! MLibrary V2 格式解析 (.Lib)
//! 这是传奇2使用的自定义库文件格式

use crate::error::{Result, LibraryError};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use image::{Rgba, RgbaImage};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

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
    /// 是否加载图像
    pub load: bool,
}

/// MLibrary V2 的 MImage 结构
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
    /// 阴影 X 偏移
    pub shadow_x: i16,
    /// 阴影 Y 偏移
    pub shadow_y: i16,
    /// 阴影值
    pub shadow: u8,
    /// 数据长度
    pub length: i32,
    /// 压缩后的图像数据 (GZip)
    pub fbytes: Vec<u8>,
    /// 纹理是否有效
    pub texture_valid: bool,
    /// 解码后的图像
    pub image: Option<RgbaImage>,
    /// 预览图 (64x64)
    pub preview: Option<RgbaImage>,

    // Layer 2 (Mask)
    /// 是否有遮罩层
    pub has_mask: bool,
    /// 遮罩宽度
    pub mask_width: i16,
    /// 遮罩高度
    pub mask_height: i16,
    /// 遮罩 X 偏移
    pub mask_x: i16,
    /// 遮罩 Y 偏移
    pub mask_y: i16,
    /// 遮罩数据
    pub mask_fbytes: Vec<u8>,
    /// 遮罩图像
    pub mask_image: Option<RgbaImage>,
}

impl MImage {
    /// 创建新的空白图像
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            x: 0,
            y: 0,
            shadow_x: 0,
            shadow_y: 0,
            shadow: 0,
            length: 0,
            fbytes: Vec::new(),
            texture_valid: false,
            image: None,
            preview: None,
            has_mask: false,
            mask_width: 0,
            mask_height: 0,
            mask_x: 0,
            mask_y: 0,
            mask_fbytes: Vec::new(),
            mask_image: None,
        }
    }

    /// 从位图创建 MImage
    pub fn from_image(img: &RgbaImage, x: i16, y: i16) -> Self {
        let width = img.width() as i16;
        let height = img.height() as i16;

        let mut result = Self::new();
        result.width = width;
        result.height = height;
        result.x = x;
        result.y = y;
        result.image = Some(img.clone());

        // 转换为字节数组并压缩
        let pixels = Self::convert_bitmap_to_array(img);
        result.fbytes = Self::compress(&pixels);
        result.length = result.fbytes.len() as i32;
        result.texture_valid = true;

        result
    }

    /// 从位图创建带遮罩的 MImage
    pub fn from_image_with_mask(img: &RgbaImage, mask_img: &RgbaImage, x: i16, y: i16) -> Self {
        let mut result = Self::from_image(img, x, y);
        result.has_mask = true;
        result.mask_width = mask_img.width() as i16;
        result.mask_height = mask_img.height() as i16;
        result.mask_image = Some(mask_img.clone());

        let mask_pixels = Self::convert_bitmap_to_array(mask_img);
        result.mask_fbytes = Self::compress(&mask_pixels);

        result
    }

    /// 将图像转换为字节数组
    fn convert_bitmap_to_array(img: &RgbaImage) -> Vec<u8> {
        let mut pixels = Vec::with_capacity((img.width() * img.height() * 4) as usize);

        for pixel in img.pixels() {
            let [r, g, b, a] = pixel.0;
            pixels.push(r);
            pixels.push(g);
            pixels.push(b);
            // 黑色像素设为透明
            pixels.push(if r == 0 && g == 0 && b == 0 { 0 } else { a });
        }

        pixels
    }

    /// GZip 压缩
    fn compress(data: &[u8]) -> Vec<u8> {
        use std::io::Write;

        let mut compressed = Vec::new();
        {
            let mut encoder = GzEncoder::new(&mut compressed, Compression::default());
            encoder.write_all(data).unwrap();
        }
        compressed
    }

    /// GZip 解压
    fn decompress(data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    /// 创建纹理
    pub fn create_texture(&mut self) -> Result<()> {
        let width = self.width as u32;
        let height = self.height as u32;

        if width == 0 || height == 0 {
            return Err(LibraryError::InvalidImageData);
        }

        if width < 2 || height < 2 {
            return Err(LibraryError::InvalidImageData);
        }

        // 解压数据
        let decompressed = Self::decompress(&self.fbytes)?;

        let mut rgba_img = RgbaImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                if idx + 3 < decompressed.len() {
                    let r = decompressed[idx];
                    let g = decompressed[idx + 1];
                    let b = decompressed[idx + 2];
                    let a = decompressed[idx + 3];
                    rgba_img.put_pixel(x, y, Rgba([r, g, b, a]));
                }
            }
        }

        self.image = Some(rgba_img);
        self.texture_valid = true;

        // 如果有遮罩，创建遮罩图像
        if self.has_mask {
            let mask_width = self.mask_width as u32;
            let mask_height = self.mask_height as u32;

            if mask_width > 0 && mask_height > 0 {
                let mask_decompressed = Self::decompress(&self.mask_fbytes)?;

                let mut mask_img = RgbaImage::new(mask_width, mask_height);

                for y in 0..mask_height {
                    for x in 0..mask_width {
                        let idx = ((y * mask_width + x) * 4) as usize;
                        if idx + 3 < mask_decompressed.len() {
                            let r = mask_decompressed[idx];
                            let g = mask_decompressed[idx + 1];
                            let b = mask_decompressed[idx + 2];
                            let a = mask_decompressed[idx + 3];
                            mask_img.put_pixel(x, y, Rgba([r, g, b, a]));
                        }
                    }
                }

                self.mask_image = Some(mask_img);
            }
        }

        Ok(())
    }

    /// 创建预览图 (64x64)
    pub fn create_preview(&mut self) {
        if let Some(ref image) = self.image {
            let preview = RgbaImage::from_fn(64, 64, |px, py| {
                let w = std::cmp::min(self.width as u32, 64);
                let h = std::cmp::min(self.height as u32, 64);
                let offset_x = (64 - w) / 2;
                let offset_y = (64 - h) / 2;

                if px >= offset_x && px < offset_x + w && py >= offset_y && py < offset_y + h {
                    let x = px - offset_x;
                    let y = py - offset_y;
                    if x < image.width() && y < image.height() {
                        *image.get_pixel(x, y)
                    } else {
                        Rgba([0, 0, 0, 0])
                    }
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
        writer.write_i16::<LittleEndian>(self.shadow_x)?;
        writer.write_i16::<LittleEndian>(self.shadow_y)?;

        let shadow_byte = if self.has_mask {
            self.shadow | 0x80
        } else {
            self.shadow
        };
        writer.write_u8(shadow_byte)?;

        writer.write_i32::<LittleEndian>(self.length)?;
        writer.extend_from_slice(&self.fbytes);

        if self.has_mask {
            writer.write_i16::<LittleEndian>(self.mask_width)?;
            writer.write_i16::<LittleEndian>(self.mask_height)?;
            writer.write_i16::<LittleEndian>(self.mask_x)?;
            writer.write_i16::<LittleEndian>(self.mask_y)?;
            writer.write_i32::<LittleEndian>(self.mask_fbytes.len() as i32)?;
            writer.extend_from_slice(&self.mask_fbytes);
        }

        Ok(())
    }
}

impl Default for MImage {
    fn default() -> Self {
        Self::new()
    }
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
            load: true,
        };

        library.initialize()?;
        Ok(library)
    }

    /// 初始化库
    pub fn initialize(&mut self) -> Result<()> {
        self.initialized = true;

        let lib_path = format!("{}.Lib", self.file_name);

        if !Path::new(&lib_path).exists() {
            return Ok(()); // 文件不存在时直接返回
        }

        let file = File::open(&lib_path)?;
        let mut reader = BufReader::new(file);

        // 读取版本号
        let current_version = reader.read_i32::<LittleEndian>()?;
        if current_version != Self::LIB_VERSION {
            tracing::error!("Wrong version, expecting lib version: {} found version: {}",
                Self::LIB_VERSION, current_version);
            return Err(LibraryError::UnsupportedVersion(current_version));
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
                img.create_texture()?;
            }
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

        let image = Self::read_mimage(&mut reader)?;
        self.images[index] = Some(image);

        Ok(())
    }

    /// 读取 MImage 数据
    fn read_mimage(reader: &mut BufReader<File>) -> Result<MImage> {
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

        let mut img = MImage::new();
        img.width = width;
        img.height = height;
        img.x = x;
        img.y = y;
        img.shadow_x = shadow_x;
        img.shadow_y = shadow_y;
        img.shadow = shadow & 0x7F; // 移除最高位
        img.length = length;
        img.fbytes = fbytes;
        img.has_mask = has_mask;

        if has_mask {
            img.mask_width = reader.read_i16::<LittleEndian>()?;
            img.mask_height = reader.read_i16::<LittleEndian>()?;
            img.mask_x = reader.read_i16::<LittleEndian>()?;
            img.mask_y = reader.read_i16::<LittleEndian>()?;
            let mask_length = reader.read_i32::<LittleEndian>()?;

            img.mask_fbytes = vec![0u8; mask_length as usize];
            reader.read_exact(&mut img.mask_fbytes)?;
        }

        Ok(img)
    }

    /// 获取指定索引的图像
    pub fn get_image(&mut self, index: usize) -> Result<&MImage> {
        self.check_image(index)?;

        self.images[index]
            .as_ref()
            .ok_or_else(|| LibraryError::IndexOutOfBounds(index))
    }

    /// 获取预览图
    pub fn get_preview(&mut self, index: usize) -> Result<Option<&RgbaImage>> {
        self.check_image(index)?;

        if let Some(ref img) = self.images[index] {
            Ok(img.image.as_ref())
        } else {
            Ok(None)
        }
    }

    /// 添加新图像
    pub fn add_image(&mut self, image: &MImage) {
        self.count += 1;
        self.images.push(Some(image.clone()));
    }

    /// 添加带遮罩的图像
    pub fn add_image_with_mask(&mut self, image: &MImage, mask_image: &MImage) {
        let mut new_image = image.clone();
        new_image.has_mask = true;
        new_image.mask_width = mask_image.width;
        new_image.mask_height = mask_image.height;
        new_image.mask_fbytes = mask_image.fbytes.clone();
        new_image.mask_image = mask_image.image.clone();

        self.count += 1;
        self.images.push(Some(new_image));
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

        let offset = 8 + (self.images.len() * 4) as u32;

        for img in self.images.iter().flatten() {
            let current_offset = data_stream.len() as u32 + offset;
            index_list.push(current_offset);
            img.save(&mut data_stream)?;
        }

        // 写入文件
        let lib_path = format!("{}.Lib", self.file_name);
        let file = File::create(&lib_path)?;
        let mut writer = BufWriter::new(file);

        writer.write_i32::<LittleEndian>(Self::LIB_VERSION)?;
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
        let lib = MLibraryV2::new("test".to_string());
        assert!(lib.is_ok()); // 文件不存在时应该返回 Ok
    }

    #[test]
    fn test_mimage_creation() {
        let img = MImage::new();
        assert_eq!(img.width, 0);
        assert_eq!(img.height, 0);
        assert!(!img.has_mask);
    }
}
