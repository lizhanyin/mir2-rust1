//! 图像处理模块

pub mod bitmap;
pub mod palette;
pub mod palette_data;
pub mod compression;

// 重新导出 MImage（已移至 formats::mlibrary_v1）
pub use crate::formats::MImage;
pub use palette::{Color, DEFAULT_PALETTE};

/// 16位颜色转32位颜色
pub fn convert_16bit_to_32bit(color: u16) -> u32 {
    let red = ((color & 0xf800) >> 8) as u8;
    let green = ((color & 0x07e0) >> 3) as u8;
    let blue = ((color & 0x001f) << 3) as u8;

    if red == 0 && green == 0 && blue == 0 {
        return 0;
    }

    ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32) | (255 << 24)
}

/// 16位颜色转32位颜色（带透明度）
pub fn convert_16bit_to_32bit_with_alpha(color: u16, alpha_byte: u8, x: usize) -> u32 {
    let red = ((color & 0xf800) >> 8) as u8;
    let green = ((color & 0x07e0) >> 3) as u8;
    let blue = ((color & 0x001f) << 3) as u8;

    // 从 alpha 字节中提取透明度值
    let alpha = if x % 2 != 0 {
        ((alpha_byte & 0x0f) * 17) as u32
    } else {
        (((alpha_byte & 0xf0) >> 4) * 17) as u32
    };

    ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32) | (alpha << 24)
}

/// 计算行字节数（用于 BMP 格式）
pub fn width_bytes(bit_count: u32, width: u32) -> u32 {
    ((width * bit_count) + 31) / 32 * 4
}

/// 跳过的字节数
pub fn skip_bytes(bit: u32, width: u32) -> u32 {
    width_bytes(bit * width, width) - width * (bit / 8)
}
