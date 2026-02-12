//! 位图和图像数据结构

use crate::error::{Result, LibraryError};
use crate::image::compression::{compress_gzip, decompress_gzip};
use image::{Rgba, RgbaImage};

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
            fixed_image = RgbaImage::from_fn(
                fixed_width as u32,
                fixed_height as u32,
                |x, y| {
                    if x < image.width() && y < image.height() {
                        *image.get_pixel(x, y)
                    } else {
                        Rgba([0, 0, 0, 0])
                    }
                },
            );
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

            let resized = imageops::resize(
                image,
                w,
                h,
                imageops::FilterType::Triangle,
            );

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

/// Bitmap 别名
pub type Bitmap = RgbaImage;
