//! WeMade Library 格式解析 (.wil/.wix)
//! 这是传奇1使用的旧版库文件格式
//!
//! WIX 文件（索引文件）结构：
//! - 文件头：44字节，包含 "#INDX v1.0-WEMADE Entertainment inc."
//! - 图片数量：偏移 0x2C (44字节)，4字节，小端序
//! - 图像数据起始位置：偏移 0x30 (48字节)，通常为 1080
//! - 图像位置数组：每个图像4字节，存储在 WIL 文件中的偏移量
//!
//! WIL 文件（数据文件）结构：
//! - 文件头：44字节
//! - 控制信息：偏移 44-55，12字节（可忽略）
//! - 调色板：偏移 56-1079，1024字节（256色 BGRA）
//! - 图像数据：从偏移 1080 开始
//!   - 宽度：2字节
//!   - 高度：2字节
//!   - 固定标识：4字节
//!   - 像素数据：宽度 × 高度 字节（8-bit 调色板索引）

use crate::error::{LibraryError, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use image::{Rgba, RgbaImage};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// WIX 文件头标识 (44字节)
const WIX_HEADER: [u8; 44] = *b"#INDX v1.0-WEMADE Entertainment inc.\0\0\0\0\0\0\0\0";
/// WIL 文件头标识 (44字节)
const WIL_HEADER: [u8; 44] = *b"#WEMADE Entertainment inc.\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
/// 图像数据在 WIL 文件中的起始偏移量
const IMAGE_DATA_OFFSET: u64 = 1080;
/// 调色板在 WIL 文件中的起始偏移量
const PALETTE_OFFSET: u64 = 56;
/// 调色板大小（256色 * 4字节）
const PALETTE_SIZE: usize = 1024;

/// WeMade Library - 用于处理 .wil/.wix 文件
pub struct MLibraryV0 {
    /// 文件基础路径（不含扩展名）
    pub file_name: String,
    /// 图像列表
    pub images: Vec<Option<MImage>>,
    /// 索引列表（存储每个图像在 WIL 文件中的偏移量）
    pub index_list: Vec<u32>,
    /// 图像计数
    pub count: usize,
    /// 是否已初始化
    initialized: bool,
    /// 是否加载图像
    pub load: bool,
    /// 调色板（256色 BGRA）
    palette: [[u8; 4]; 256],
}

/// WeMade Library 的 MImage 结构
#[derive(Debug, Clone)]
pub struct MImage {
    /// 图像宽度
    pub width: u16,
    /// 图像高度
    pub height: u16,
    /// X 偏移（WeMade 格式通常不使用）
    pub x: i16,
    /// Y 偏移（WeMade 格式通常不使用）
    pub y: i16,
    /// 固定标识
    pub flag: u32,
    /// 像素数据（8-bit 调色板索引）
    pub fbytes: Vec<u8>,
    /// 纹理是否有效
    pub texture_valid: bool,
    /// 解码后的 RGBA 图像
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
            flag: 0xFFD40007, // 默认标识
            fbytes: Vec::new(),
            texture_valid: false,
            image: None,
            preview: None,
        }
    }

    /// 从 RGBA 图像创建 MImage（使用调色板量化）
    pub fn from_image(img: &RgbaImage, x: i16, y: i16, palette: &[[u8; 4]; 256]) -> Self {
        let width = img.width() as u16;
        let height = img.height() as u16;

        // 将 RGBA 像素转换为调色板索引
        let mut fbytes = Vec::with_capacity((width * height) as usize);

        for pixel in img.pixels() {
            let [r, g, b, a] = pixel.0;
            // 查找最接近的调色板颜色
            let index = find_closest_palette_color(r, g, b, a, palette);
            fbytes.push(index);
        }

        Self {
            width,
            height,
            x,
            y,
            flag: 0xFFD40007,
            fbytes,
            texture_valid: true,
            image: Some(img.clone()),
            preview: None,
        }
    }

    /// 使用调色板解码图像
    pub fn decode_with_palette(&mut self, palette: &[[u8; 4]; 256]) -> Result<()> {
        if self.width == 0 || self.height == 0 {
            return Err(LibraryError::InvalidImageData);
        }

        let width = self.width as u32;
        let height = self.height as u32;
        let expected_size = (width * height) as usize;

        if self.fbytes.len() != expected_size {
            tracing::warn!(
                "图像数据大小不匹配: 期望 {} 字节, 实际 {} 字节",
                expected_size,
                self.fbytes.len()
            );
            // 调整数据大小
            self.fbytes.resize(expected_size, 0);
        }

        let mut rgba_img = RgbaImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                let palette_idx = self.fbytes[idx] as usize;
                let color = palette[palette_idx];

                // 调色板格式: BGRA
                let b = color[0];
                let g = color[1];
                let r = color[2];
                let a = color[3];

                // 索引0通常表示透明
                let (r, g, b, a) = if palette_idx == 0 {
                    (0u8, 0u8, 0u8, 0u8)
                } else {
                    (r, g, b, if a == 0 { 255 } else { a })
                };

                // 图像需要垂直翻转
                rgba_img.put_pixel(x, height - 1 - y, Rgba([r, g, b, a]));
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

    /// 保存图像数据到字节流
    pub fn save(&self, writer: &mut Vec<u8>) -> Result<()> {
        writer.write_u16::<LittleEndian>(self.width)?;
        writer.write_u16::<LittleEndian>(self.height)?;
        writer.write_u32::<LittleEndian>(self.flag)?;
        writer.extend_from_slice(&self.fbytes);
        Ok(())
    }
}

impl Default for MImage {
    fn default() -> Self {
        Self::new()
    }
}

/// 查找最接近的调色板颜色索引
fn find_closest_palette_color(r: u8, g: u8, b: u8, a: u8, palette: &[[u8; 4]; 256]) -> u8 {
    // 透明像素使用索引0
    if a < 128 {
        return 0;
    }

    let mut min_dist = u32::MAX;
    let mut best_idx = 0u8;

    for (idx, color) in palette.iter().enumerate() {
        if idx == 0 {
            continue; // 跳过透明色
        }

        // 计算颜色距离（使用欧几里得距离）
        let dr = (r as i32 - color[2] as i32).pow(2);
        let dg = (g as i32 - color[1] as i32).pow(2);
        let db = (b as i32 - color[0] as i32).pow(2);
        let dist = (dr + dg + db) as u32;

        if dist < min_dist {
            min_dist = dist;
            best_idx = idx as u8;
        }
    }

    best_idx
}

impl MLibraryV0 {
    /// 创建新的 WeMade Library 实例
    pub fn new(file_name: String) -> Result<Self> {
        let mut library = Self {
            file_name,
            images: Vec::new(),
            index_list: Vec::new(),
            count: 0,
            initialized: false,
            load: true,
            palette: [[0u8; 4]; 256],
        };

        library.initialize()?;
        Ok(library)
    }

    /// 初始化库
    pub fn initialize(&mut self) -> Result<()> {
        self.initialized = true;

        let wix_path = format!("{}.wix", self.file_name);
        let wil_path = format!("{}.wil", self.file_name);

        // 检查文件是否存在
        if !Path::new(&wix_path).exists() || !Path::new(&wil_path).exists() {
            tracing::debug!("WIL/WIX 文件不存在: {}", self.file_name);
            return Ok(());
        }

        // 读取 WIX 索引文件
        self.read_wix_file(&wix_path)?;

        // 读取 WIL 文件的调色板
        self.read_palette(&wil_path)?;

        // 初始化图像列表
        self.images = vec![None; self.count];

        tracing::info!(
            "加载 WeMade Library: {} ({} 张图像)",
            self.file_name,
            self.count
        );

        Ok(())
    }

    /// 读取 WIX 索引文件
    fn read_wix_file(&mut self, wix_path: &str) -> Result<()> {
        tracing::debug!("读取 WIX 文件: {}", wix_path);

        let file = File::open(wix_path).map_err(|e| {
            tracing::error!("无法打开 WIX 文件: {} - {}", wix_path, e);
            e
        })?;

        let metadata = file.metadata()?;
        let file_size = metadata.len();
        tracing::debug!("WIX 文件大小: {} 字节", file_size);

        let mut reader = BufReader::new(file);

        // 读取文件头（最多读取 52 字节用于分析）
        let mut header = [0u8; 52];
        if let Err(e) = reader.read_exact(&mut header) {
            tracing::warn!("读取 WIX 文件头失败: {}, 尝试读取部分头部", e);
            // 如果文件小于 52 字节，尝试读取更少
            reader.seek(SeekFrom::Start(0))?;
            let actual_len = std::cmp::min(file_size as usize, 52);
            let mut partial_header = vec![0u8; actual_len];
            reader.read_exact(&mut partial_header)?;
            header[..actual_len].copy_from_slice(&partial_header);
        }

        tracing::debug!(
            "WIX 文件头前30字节: {:?}",
            String::from_utf8_lossy(&header[..30])
        );

        // 尝试检测文件格式版本
        let (count, header_size) = if header.starts_with(b"#INDX v1.0-WEMADE") {
            // 标准格式: 44字节文件头 + 4字节数量 + 4字节数据起始位置
            tracing::debug!("检测到标准 WeMade 格式 (#INDX v1.0)");
            reader.seek(SeekFrom::Start(44))?;
            let count = reader.read_u32::<LittleEndian>()? as usize;
            let _data_start = reader.read_u32::<LittleEndian>()?;
            (count, 48)
        } else if header.starts_with(b"#INDX") {
            // 简化格式: 44字节文件头 + 4字节数量
            tracing::debug!("检测到简化 WeMade 格式 (#INDX)");
            reader.seek(SeekFrom::Start(44))?;
            let count = reader.read_u32::<LittleEndian>()? as usize;
            (count, 48)
        } else {
            // 尝试不同的偏移量来检测索引数据
            // 标准 WeMade 格式可能直接从偏移 48 或 52 开始索引数组
            tracing::debug!("未知文件头格式，尝试自动检测");

            // 尝试从偏移 48 开始读取索引
            reader.seek(SeekFrom::Start(48))?;
            let mut test_indices = Vec::new();
            for _ in 0..10 {
                match reader.read_u32::<LittleEndian>() {
                    Ok(idx) => test_indices.push(idx),
                    Err(_) => break,
                }
            }

            // 检查索引值是否合理（应该大于 1000 且递增）
            let valid_48 = test_indices.iter().all(|&x| x > 1000 && x < 10_000_000);
            if valid_48 && !test_indices.is_empty() {
                tracing::debug!("检测到索引从偏移 48 开始");
                (0, 48) // 0 表示需要计算数量
            } else {
                // 尝试从偏移 52 开始
                reader.seek(SeekFrom::Start(52))?;
                test_indices.clear();
                for _ in 0..10 {
                    match reader.read_u32::<LittleEndian>() {
                        Ok(idx) => test_indices.push(idx),
                        Err(_) => break,
                    }
                }

                let valid_52 = test_indices.iter().all(|&x| x > 1000 && x < 10_000_000);
                if valid_52 && !test_indices.is_empty() {
                    tracing::debug!("检测到索引从偏移 52 开始");
                    (0, 52) // 0 表示需要计算数量
                } else {
                    // 最后尝试：从偏移 44 开始
                    tracing::debug!("尝试从偏移 44 开始");
                    (0, 44)
                }
            }
        };

        // 如果需要计算数量
        let (count, header_size) = if count == 0 {
            reader.seek(SeekFrom::Start(header_size as u64))?;
            let index_count = ((file_size - header_size as u64) / 4) as usize;
            tracing::debug!("计算得到索引数量: {}", index_count);
            (index_count, header_size)
        } else {
            (count, header_size)
        };

        self.count = count;

        // 读取图像偏移量数组
        self.index_list.clear();
        self.index_list.reserve(self.count);

        reader.seek(SeekFrom::Start(header_size as u64))?;
        for i in 0..self.count {
            let offset = reader.read_u32::<LittleEndian>().map_err(|e| {
                tracing::error!("读取第 {} 个偏移量失败: {}", i, e);
                e
            })?;
            self.index_list.push(offset);
        }

        tracing::debug!(
            "WIX 索引读取完成: {} 张图像, {} 个偏移量",
            self.count,
            self.index_list.len()
        );
        Ok(())
    }

    /// 读取 WIL 文件的调色板
    fn read_palette(&mut self, wil_path: &str) -> Result<()> {
        tracing::debug!("读取 WIL 文件调色板: {}", wil_path);

        let file = File::open(wil_path).map_err(|e| {
            tracing::error!("无法打开 WIL 文件: {} - {}", wil_path, e);
            e
        })?;

        let metadata = file.metadata()?;
        tracing::debug!("WIL 文件大小: {} 字节", metadata.len());

        let mut reader = BufReader::new(file);

        // 读取并验证文件头（44字节）
        let mut header = [0u8; 44];
        reader.read_exact(&mut header).map_err(|e| {
            tracing::error!("读取 WIL 文件头失败: {} (文件可能小于44字节)", e);
            e
        })?;

        tracing::debug!("WIL 文件头: {:?}", String::from_utf8_lossy(&header[..30]));

        // 验证文件头标识
        if !header.starts_with(b"#WEMADE") && !header.starts_with(b"#INDX") {
            tracing::warn!(
                "WIL 文件头标识不匹配: {:?}",
                String::from_utf8_lossy(&header[..20])
            );
        }

        // 跳过控制信息（12字节），定位到调色板位置
        reader.seek(SeekFrom::Start(PALETTE_OFFSET))?;
        tracing::debug!("定位到调色板位置: {} 字节", PALETTE_OFFSET);

        // 读取调色板（256色 * 4字节 = 1024字节）
        for (i, color) in self.palette.iter_mut().enumerate() {
            reader.read_exact(color).map_err(|e| {
                tracing::error!("读取调色板第 {} 色失败: {}", i, e);
                e
            })?;
        }

        tracing::debug!("调色板读取完成");
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

        Ok(())
    }

    /// 加载指定索引的图像
    fn load_image(&mut self, index: usize) -> Result<()> {
        if index >= self.index_list.len() {
            return Err(LibraryError::IndexOutOfBounds(index));
        }

        let wil_path = format!("{}.wil", self.file_name);
        let file = File::open(&wil_path)?;
        let mut reader = BufReader::new(file);

        // 获取图像在 WIL 文件中的偏移量
        let offset = self.index_list[index] as u64;
        reader.seek(SeekFrom::Start(offset))?;

        // 读取图像数据
        let mut image = self.read_wil_image(&mut reader)?;

        // 使用调色板解码图像
        image.decode_with_palette(&self.palette)?;

        self.images[index] = Some(image);
        Ok(())
    }

    /// 从 WIL 文件读取图像数据
    fn read_wil_image(&self, reader: &mut BufReader<File>) -> Result<MImage> {
        // 读取宽度（2字节）
        let width = reader.read_u16::<LittleEndian>()?;
        // 读取高度（2字节）
        let height = reader.read_u16::<LittleEndian>()?;
        // 读取固定标识（4字节）
        let flag = reader.read_u32::<LittleEndian>()?;

        // 读取像素数据（宽度 × 高度 字节）
        let data_size = (width as usize) * (height as usize);
        let mut fbytes = vec![0u8; data_size];
        reader.read_exact(&mut fbytes)?;

        let mut img = MImage::new();
        img.width = width;
        img.height = height;
        img.flag = flag;
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
        let wix_path = format!("{}.wix", self.file_name);
        let wil_path = format!("{}.wil", self.file_name);

        // 计算 WIL 文件中图像数据的起始偏移量
        // = 文件头(44) + 控制信息(12) + 调色板(1024) = 1080
        let base_offset = IMAGE_DATA_OFFSET as u32;

        // 构建图像数据流和索引列表
        let mut data_stream = Vec::new();
        let mut index_list: Vec<u32> = Vec::new();

        for img in self.images.iter().flatten() {
            let current_offset = base_offset + data_stream.len() as u32;
            index_list.push(current_offset);
            img.save(&mut data_stream)?;
        }

        // 写入 WIX 索引文件
        {
            let file = File::create(&wix_path)?;
            let mut writer = BufWriter::new(file);

            // 写入文件头（44字节）
            writer.write_all(&WIX_HEADER)?;
            // 填充到44字节
            let header_pad = 44 - WIX_HEADER.len();
            if header_pad > 0 {
                writer.write_all(&vec![0u8; header_pad])?;
            }

            // 写入图像数量
            writer.write_u32::<LittleEndian>(self.images.len() as u32)?;
            // 写入数据起始位置
            writer.write_u32::<LittleEndian>(base_offset)?;

            // 写入索引数组
            for index in &index_list {
                writer.write_u32::<LittleEndian>(*index)?;
            }

            writer.flush()?;
        }

        // 写入 WIL 数据文件
        {
            let file = File::create(&wil_path)?;
            let mut writer = BufWriter::new(file);

            // 写入文件头（44字节）
            writer.write_all(&WIL_HEADER)?;
            // 填充到44字节
            let header_pad = 44 - WIL_HEADER.len();
            if header_pad > 0 {
                writer.write_all(&vec![0u8; header_pad])?;
            }

            // 写入控制信息（12字节，填充0）
            writer.write_all(&[0u8; 12])?;

            // 写入调色板（1024字节）
            for color in &self.palette {
                writer.write_all(color)?;
            }

            // 写入图像数据
            writer.write_all(&data_stream)?;

            writer.flush()?;
        }

        tracing::info!("保存 WeMade Library 完成: {}", self.file_name);
        Ok(())
    }

    /// 获取图像计数
    pub fn count(&self) -> usize {
        self.count
    }

    /// 获取调色板
    pub fn get_palette(&self) -> &[[u8; 4]; 256] {
        &self.palette
    }

    /// 设置调色板
    pub fn set_palette(&mut self, palette: [[u8; 4]; 256]) {
        self.palette = palette;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_library() {
        let lib = MLibraryV0::new("nonexistent_file".to_string());
        assert!(lib.is_ok()); // 文件不存在时应该返回 Ok
    }

    #[test]
    fn test_mimage_creation() {
        let img = MImage::new();
        assert_eq!(img.width, 0);
        assert_eq!(img.height, 0);
    }

    #[test]
    fn test_palette_color_matching() {
        let mut palette = [[0u8; 4]; 256];
        // 设置一些测试颜色
        palette[1] = [0, 0, 255, 255]; // 红色 (BGRA)
        palette[2] = [0, 255, 0, 255]; // 绿色
        palette[3] = [255, 0, 0, 255]; // 蓝色

        // 测试找到最接近的颜色
        let idx = find_closest_palette_color(255, 0, 0, 255, &palette);
        assert_eq!(idx, 1); // 应该匹配红色

        let idx = find_closest_palette_color(0, 255, 0, 255, &palette);
        assert_eq!(idx, 2); // 应该匹配绿色

        let idx = find_closest_palette_color(0, 0, 255, 255, &palette);
        assert_eq!(idx, 3); // 应该匹配蓝色

        // 透明像素应该返回 0
        let idx = find_closest_palette_color(255, 0, 0, 0, &palette);
        assert_eq!(idx, 0);
    }
}
