//! MLibrary V1 格式解析 (.wzl/.wzx)
//! 这是传奇2使用的库文件格式

use crate::error::{LibraryError, Result};
use crate::image::compression::{compress_gzip, decompress_gzip};
use crate::image::{Color, DEFAULT_PALETTE};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use flate2::read::ZlibDecoder;
use image::{Rgba, RgbaImage};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// MLibrary V1 - 用于处理 .wzl/.wzx 文件
pub struct MLibraryV1 {
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
    /// 是否加载图像数据
    pub load: bool,
    /// 调色板
    palette: [Color; 256],
    /// WZL 文件读取器（全局存放，避免重复打开文件）
    wzl_reader: Option<BufReader<File>>,
}

impl MLibraryV1 {
    const LIB_VERSION: i32 = 1;
    const WZX_HEADER_SIZE: u64 = 48;

    /// 创建新的 MLibrary V1 实例
    pub fn new(file_name: String) -> Result<Self> {
        let mut library = Self {
            file_name,
            images: Vec::new(),
            index_list: Vec::new(),
            count: 0,
            initialized: false,
            load: true,
            palette: DEFAULT_PALETTE,
            wzl_reader: None,
        };

        library.initialize()?;
        Ok(library)
    }

    /// 初始化库，加载索引文件
    pub fn initialize(&mut self) -> Result<()> {
        self.initialized = true;

        let wzx_path = format!("{}.wzx", self.file_name);
        let wzl_path = format!("{}.wzl", self.file_name);

        if !Path::new(&wzx_path).exists() {
            return Err(LibraryError::FileNotFound(wzx_path));
        }

        if !Path::new(&wzl_path).exists() {
            return Err(LibraryError::FileNotFound(wzl_path));
        }

        // 读取索引文件 (.wzx)
        self.load_index_file(&wzx_path)?;

        // 初始化图像列表
        self.images = vec![None; self.index_list.len()];

        // 打开 WZL 文件并全局存放（初始化后读取，完成后关闭）
        let wzl_file = File::open(&wzl_path)?;
        self.wzl_reader = Some(BufReader::new(wzl_file));

        // 初始化时检查所有图像
        // for i in 0..self.index_list.len() {
        //     let _ = self.check_image(i);
        // }
        Ok(())
    }

    /// 加载索引文件
    fn load_index_file(&mut self, path: &str) -> Result<()> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // 跳过 48 字节的头部
        reader.seek(SeekFrom::Start(Self::WZX_HEADER_SIZE))?;

        // 读取所有索引
        self.index_list.clear();
        while let Ok(index) = reader.read_u32::<LittleEndian>() {
            self.index_list.push(index);
            self.images.push(None);
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
        let offset = self.index_list[index] as u64;

        // 使用全局存储的文件流
        if let Some(ref mut reader) = self.wzl_reader {
            reader.seek(SeekFrom::Start(offset))?;
            let image = Self::read_mimage(&self.palette, reader, offset)?;
            self.images[index] = Some(image);
        } else {
            return Err(LibraryError::FileNotFound(
                "WZL reader not initialized".to_string(),
            ));
        }

        Ok(())
    }

    /// 读取 MImage 数据
    fn read_mimage(
        palette: &[Color; 256],
        reader: &mut BufReader<File>,
        offset: u64,
    ) -> Result<MImage> {
        reader.seek(SeekFrom::Start(offset))?;

        // 读取头部信息 (16字节)
        let flag = reader.read_u8()?;

        // 如果位置为0，返回空图像
        if reader.stream_position()? == 1 {
            return Ok(MImage::new());
        }

        let bo16bit = flag == 5;

        // 跳过 3 字节
        reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        let width = reader.read_i16::<LittleEndian>()?;
        let height = reader.read_i16::<LittleEndian>()?;
        let x = reader.read_i16::<LittleEndian>()?;
        let y = reader.read_i16::<LittleEndian>()?;
        let n_size = reader.read_i32::<LittleEndian>()?;

        // 检查图像尺寸是否有效
        // 使用 i32 避免两个 i16 相乘溢出
        if (width as i32) * (height as i32) < 4 {
            return Ok(MImage::new());
        }

        // 跳转到数据开始位置 (偏移 + 16字节头部)
        reader.seek(SeekFrom::Start(offset + 16))?;

        // 读取图像数据
        let bytes = if n_size == 0 {
            // 未压缩 - 直接读取原始数据
            // 使用 i32 避免两个 i16 相乘溢出
            let size = if bo16bit {
                ((width as i32) * (height as i32) * 2) as usize
            } else {
                ((width as i32) * (height as i32)) as usize
            };
            let mut buf = vec![0u8; size];
            reader.read_exact(&mut buf)?;
            buf
        } else {
            // Zlib 压缩
            let mut compressed_data = vec![0u8; n_size as usize];
            reader.read_exact(&mut compressed_data)?;

            // 解压
            let mut decoder = ZlibDecoder::new(&compressed_data[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            decompressed
        };

        // 创建图像
        let mut img = MImage::new();
        img.width = width;
        img.height = height;
        img.x = x;
        img.y = y;
        img.fbytes = bytes.clone();

        // 将原始字节数据转换为图像
        Self::convert_bytes_to_image(palette, &mut img, &bytes, bo16bit)?;

        Ok(img)
    }

    /// 将字节数据转换为图像
    fn convert_bytes_to_image(
        palette: &[Color; 256],
        img: &mut MImage,
        bytes: &[u8],
        bo16bit: bool,
    ) -> Result<()> {
        let width = img.width as u32;
        let height = img.height as u32;

        if width == 0 || height == 0 {
            return Err(LibraryError::InvalidImageData);
        }

        let mut rgba_img = RgbaImage::new(width, height);
        let mut idx = 0;

        // 计算每行字节数 (需要4字节对齐)
        let row_bytes = if bo16bit { width * 2 } else { width };
        let aligned_row_bytes = row_bytes.div_ceil(4) * 4;

        for y in (0..height).rev() {
            for x in 0..width {
                if idx >= bytes.len() {
                    break;
                }

                let pixel = if bo16bit {
                    // 16位颜色格式 (RGB565)
                    let b1 = bytes[idx] as u16;
                    let b2 = bytes[idx + 1] as u16;
                    let color = (b2 << 8) | b1;
                    idx += 2;

                    // RGB565 转 RGB888
                    let r = ((color & 0xF800) >> 8) as u8;
                    let g = ((color & 0x07E0) >> 3) as u8;
                    let b = ((color & 0x001F) << 3) as u8;

                    // 如果全黑则透明
                    if r == 0 && g == 0 && b == 0 {
                        [0, 0, 0, 0]
                    } else {
                        [r, g, b, 255]
                    }
                } else {
                    // 8位索引颜色
                    let palette_idx = bytes[idx] as usize;
                    idx += 1;

                    if palette_idx < palette.len() {
                        let color = &palette[palette_idx];
                        [color.r, color.g, color.b, color.a]
                    } else {
                        [0, 0, 0, 0]
                    }
                };

                rgba_img.put_pixel(x, y, Rgba(pixel));
            }

            // 跳过对齐填充字节
            idx += (aligned_row_bytes - row_bytes) as usize;
        }

        img.image = Some(rgba_img);
        img.texture_valid = true;
        Ok(())
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
        let wzl_path = format!("{}.wzl", self.file_name);

        // 使用内存流计算索引
        let mut data_stream = Vec::new();
        let mut index_list: Vec<u32> = Vec::new();

        let offset = 8 + (self.images.len() * 4) as u32;

        for img in self.images.iter().flatten() {
            // 当前位置 + 初始偏移
            let current_offset = data_stream.len() as u32 + offset;
            index_list.push(current_offset);

            // 写入图像数据
            self.write_mimage_data(img, &mut data_stream)?;
        }

        // 写入 .wzl 文件
        let file = File::create(&wzl_path)?;
        let mut writer = BufWriter::new(file);

        // 写入文件头
        writer.write_i32::<LittleEndian>(Self::LIB_VERSION)?;
        writer.write_i32::<LittleEndian>(self.images.len() as i32)?;

        // 写入索引列表
        for index in &index_list {
            writer.write_i32::<LittleEndian>(*index as i32)?;
        }

        // 写入图像数据
        writer.write_all(&data_stream)?;
        writer.flush()?;

        Ok(())
    }

    /// 写入 MImage 数据 (匹配 C# 的 MImage.Save 方法)
    fn write_mimage_data(&self, image: &MImage, writer: &mut Vec<u8>) -> Result<()> {
        use byteorder::{LittleEndian, WriteBytesExt};

        // 写入基础信息 (14字节: Width + Height + X + Y + ShadowX + ShadowY)
        writer.write_i16::<LittleEndian>(image.width)?;
        writer.write_i16::<LittleEndian>(image.height)?;
        writer.write_i16::<LittleEndian>(image.x)?;
        writer.write_i16::<LittleEndian>(image.y)?;
        writer.write_i16::<LittleEndian>(image.shadow_x)?;
        writer.write_i16::<LittleEndian>(image.shadow_y)?;

        // 写入 Shadow 字节 (1字节) - 最高位表示是否有 Mask
        let shadow_byte = if image.has_mask {
            image.shadow | 0x80
        } else {
            image.shadow
        };
        writer.write_u8(shadow_byte)?;

        // 写入数据长度 (4字节)
        writer.write_i32::<LittleEndian>(image.fbytes.len() as i32)?;

        // 写入图像数据
        writer.extend_from_slice(&image.fbytes);

        // 如果有 Mask 层，写入 Mask 数据
        if image.has_mask {
            writer.write_i16::<LittleEndian>(image.mask_width)?;
            writer.write_i16::<LittleEndian>(image.mask_height)?;
            writer.write_i16::<LittleEndian>(image.mask_x)?;
            writer.write_i16::<LittleEndian>(image.mask_y)?;
            writer.write_i32::<LittleEndian>(image.mask_fbytes.len() as i32)?;
            writer.extend_from_slice(&image.mask_fbytes);
        }

        Ok(())
    }

    /// 获取图像计数
    pub fn count(&self) -> usize {
        self.count
    }

    /// 手动关闭 WZL 文件流
    pub fn close(&mut self) {
        self.wzl_reader = None;
    }
}

/// 自动关闭文件流（当 MLibraryV1 被销毁时）
impl Drop for MLibraryV1 {
    fn drop(&mut self) {
        self.close();
    }
}

/// MImage - 传奇2库文件中的图像结构
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
    /// 压缩后的图像数据
    pub fbytes: Vec<u8>,
    /// 图像纹理是否有效
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
    /// 创建一个新的空白图像
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            x: 0,
            y: 0,
            shadow_x: 0,
            shadow_y: 0,
            shadow: 0,
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

    /// 从图像数据创建 MImage
    pub fn from_image(image: &RgbaImage, x: i16, y: i16) -> Self {
        let width = image.width() as i16;
        let height = image.height() as i16;

        // 固定图像大小为4的倍数
        let fixed_width = width + (4 - width % 4) % 4;
        let fixed_height = height + (4 - height % 4) % 4;

        let mut fixed_image = image.clone();

        if fixed_width != width || fixed_height != height {
            // 需要调整大小
            fixed_image = RgbaImage::from_fn(fixed_width as u32, fixed_height as u32, |x, y| {
                if x < image.width() && y < image.height() {
                    *image.get_pixel(x, y)
                } else {
                    Rgba([0, 0, 0, 0])
                }
            });
        }

        let pixels = Self::convert_image_to_bytes(&fixed_image);
        let fbytes = compress_gzip(&pixels).unwrap_or_default();

        Self {
            width: fixed_width,
            height: fixed_height,
            x,
            y,
            shadow_x: 0,
            shadow_y: 0,
            shadow: 0,
            fbytes,
            texture_valid: true,
            image: Some(fixed_image),
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

    /// 将图像转换为字节数组
    fn convert_image_to_bytes(image: &RgbaImage) -> Vec<u8> {
        let mut pixels = Vec::with_capacity((image.width() * image.height() * 4) as usize);

        for pixel in image.pixels() {
            let [r, g, b, a] = pixel.0;
            // 交换 R 和 B (BGR 格式)
            pixels.push(b);
            pixels.push(g);
            pixels.push(r);
            pixels.push(if r == 0 && g == 0 && b == 0 { 0 } else { a });
        }

        pixels
    }

    /// 从字节数组创建图像
    pub fn create_texture(&mut self, data: &[u8]) -> Result<()> {
        if self.width <= 0 || self.height <= 0 {
            return Err(LibraryError::InvalidImageData);
        }

        let width = self.width as u32;
        let height = self.height as u32;

        // 解压数据
        let decompressed = decompress_gzip(data)?;

        if decompressed.len() != (width * height * 4) as usize {
            return Err(LibraryError::InvalidImageData);
        }

        // 创建图像缓冲区
        let mut img = RgbaImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                if idx + 3 < decompressed.len() {
                    let b = decompressed[idx];
                    let g = decompressed[idx + 1];
                    let r = decompressed[idx + 2];
                    let a = decompressed[idx + 3];
                    img.put_pixel(x, height - 1 - y, Rgba([r, g, b, a]));
                }
            }
        }

        self.image = Some(img);
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
}

impl Default for MImage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_library() {
        let lib = MLibraryV1::new("test".to_string());
        assert!(lib.is_err()); // 文件不存在
    }
}
