//! 调色板定义和管理

/// RGBA 颜色结构
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self { a, r, g, b }
    }

    /// 转换为 u32 (ABGR 格式用于 little-endian 系统的 ARGB)
    pub fn to_u32(self) -> u32 {
        (self.a as u32) << 24 | (self.b as u32) << 16 | (self.g as u32) << 8 | self.r as u32
    }

    /// 从 u32 创建颜色
    pub fn from_u32(value: u32) -> Self {
        Self {
            a: ((value >> 24) & 0xFF) as u8,
            b: ((value >> 16) & 0xFF) as u8,
            g: ((value >> 8) & 0xFF) as u8,
            r: (value & 0xFF) as u8,
        }
    }

    /// 从 ARGB 各分量创建颜色
    pub fn from_argb(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self { a, r, g, b }
    }

    /// 创建黑色
    pub const fn black() -> Self {
        Self { a: 255, r: 0, g: 0, b: 0 }
    }

    /// 创建白色
    pub const fn white() -> Self {
        Self { a: 255, r: 255, g: 255, b: 255 }
    }

    /// 检查颜色是否完全透明
    pub fn is_transparent(self) -> bool {
        self.a == 0
    }

    /// 检查颜色是否不透明
    pub fn is_opaque(self) -> bool {
        self.a == 255
    }

    /// 计算颜色的亮度 (使用标准亮度公式)
    pub fn brightness(self) -> u8 {
        ((299 * self.r as u32 + 587 * self.g as u32 + 114 * self.b as u32) / 1000) as u8
    }

    /// 混合两个颜色
    pub fn blend(self, other: Color, factor: u8) -> Color {
        let f = factor as u32;
        let inv_f = 255 - f;

        Color {
            a: ((self.a as u32 * inv_f + other.a as u32 * f) / 255) as u8,
            r: ((self.r as u32 * inv_f + other.r as u32 * f) / 255) as u8,
            g: ((self.g as u32 * inv_f + other.g as u32 * f) / 255) as u8,
            b: ((self.b as u32 * inv_f + other.b as u32 * f) / 255) as u8,
        }
    }

    /// 格式化为十六进制颜色字符串 (如 "#FF0000" 或 "#FF0000FF" 带alpha)
    pub fn to_hex_string(self, with_alpha: bool) -> String {
        if with_alpha {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        } else {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        }
    }

    /// 格式化为 RGB 字符串 (如 "rgb(255, 0, 0)")
    pub fn to_rgb_string(self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }

    /// 格式化为 RGBA 字符串 (如 "rgba(255, 0, 0, 255)")
    pub fn to_rgba_string(self) -> String {
        format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }

    /// 格式化为 CSS 颜色字符串 (带 alpha 时使用 rgba)
    pub fn to_css_string(self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.a == 255 {
            write!(f, "Color(#{:02X}{:02X}{:02X})", self.r, self.g, self.b)
        } else {
            write!(f, "Color(#{:02X}{:02X}{:02X}{:02X})", self.r, self.g, self.b, self.a)
        }
    }
}

impl std::fmt::LowerHex for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.a == 255 {
            write!(f, "{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            write!(f, "{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }
}

impl std::fmt::UpperHex for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.a == 255 {
            write!(f, "{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            write!(f, "{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }
}

// 重新导出生成的调色板常量 (也可以在模块内使用)
pub use crate::image::palette_data::{DEFAULT_PALETTE, PALETTE_U32};

/// 调色板类型
pub type Palette = [Color; 256];

/// 创建默认调色板
pub fn create_default_palette() -> Palette {
    DEFAULT_PALETTE
}

/// 从调色板索引获取颜色
#[inline]
pub fn get_color(index: usize) -> Color {
    DEFAULT_PALETTE[index]
}

/// 从调色板索引获取u32格式颜色
#[inline]
pub fn get_color_u32(index: usize) -> u32 {
    PALETTE_U32[index]
}

/// 调色板迭代器
pub fn iter_palette() -> impl Iterator<Item = Color> {
    DEFAULT_PALETTE.iter().copied()
}

/// 按亮度排序的调色板索引
pub struct BrightnessSortedPalette {
    indices: [usize; 256],
}

impl BrightnessSortedPalette {
    pub fn new() -> Self {
        let mut indices: [usize; 256] = [0; 256];
        for i in 0..256 {
            indices[i] = i;
        }

        // 按亮度排序 (使用简化的亮度公式: 0.299*R + 0.587*G + 0.114*B)
        indices.sort_by_key(|&i| {
            let c = DEFAULT_PALETTE[i];
            let brightness = (299 * c.r as u32 + 587 * c.g as u32 + 114 * c.b as u32) / 1000;
            brightness
        });

        Self { indices }
    }

    /// 获取指定亮度范围的索引范围
    pub fn get_range(&self, min_brightness: u8, max_brightness: u8) -> &[usize] {
        let min = min_brightness as u32;
        let max = max_brightness as u32;

        let start = self.indices.partition_point(|&i| {
            let c = DEFAULT_PALETTE[i];
            let brightness = (299 * c.r as u32 + 587 * c.g as u32 + 114 * c.b as u32) / 1000;
            brightness < min
        });

        let end = self.indices.partition_point(|&i| {
            let c = DEFAULT_PALETTE[i];
            let brightness = (299 * c.r as u32 + 587 * c.g as u32 + 114 * c.b as u32) / 1000;
            brightness <= max
        });

        &self.indices[start..end]
    }

    /// 获取排序后的索引数组
    pub fn indices(&self) -> &[usize; 256] {
        &self.indices
    }
}

impl Default for BrightnessSortedPalette {
    fn default() -> Self {
        Self::new()
    }
}

/// 颜色调色板管理器
pub struct PaletteManager {
    palette: Palette,
    brightness_sorted: BrightnessSortedPalette,
}

impl PaletteManager {
    /// 创建使用默认调色板的管理器
    pub fn new() -> Self {
        Self {
            palette: DEFAULT_PALETTE,
            brightness_sorted: BrightnessSortedPalette::new(),
        }
    }

    /// 使用自定义调色板创建管理器
    pub fn with_palette(palette: Palette) -> Self {
        Self {
            palette,
            brightness_sorted: BrightnessSortedPalette::new(),
        }
    }

    /// 获取调色板
    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    /// 获取指定索引的颜色
    pub fn get(&self, index: usize) -> Color {
        self.palette[index]
    }

    /// 获取亮度排序的调色板
    pub fn brightness_sorted(&self) -> &BrightnessSortedPalette {
        &self.brightness_sorted
    }

    /// 查找最接近的颜色索引
    pub fn find_closest(&self, color: Color) -> usize {
        let mut best_index = 0;
        let mut best_distance = u32::MAX;

        for (i, &palette_color) in self.palette.iter().enumerate() {
            // 计算颜色距离 (简单的欧几里得距离)
            let dr = palette_color.r as i32 - color.r as i32;
            let dg = palette_color.g as i32 - color.g as i32;
            let db = palette_color.b as i32 - color.b as i32;
            let distance = (dr * dr + dg * dg + db * db) as u32;

            if distance < best_distance {
                best_distance = distance;
                best_index = i;
            }
        }

        best_index
    }
}

impl Default for PaletteManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_conversion() {
        let color = Color::new(255, 128, 64, 32);
        let u32_val = color.to_u32();
        let color2 = Color::from_u32(u32_val);
        assert_eq!(color, color2);
    }

    #[test]
    fn test_transparent_check() {
        let transparent = Color::new(0, 255, 0, 0);
        assert!(transparent.is_transparent());
        assert!(!transparent.is_opaque());

        let opaque = Color::new(255, 128, 64, 32);
        assert!(!opaque.is_transparent());
        assert!(opaque.is_opaque());
    }

    #[test]
    fn test_palette_size() {
        assert_eq!(DEFAULT_PALETTE.len(), 256);
    }

    #[test]
    fn test_get_color() {
        let color = get_color(1);
        assert_eq!(color.a, 255);
        assert_eq!(color.r, 128);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_brightness() {
        let white = Color::white();
        assert_eq!(white.brightness(), 255);

        let black = Color::black();
        assert_eq!(black.brightness(), 0);
    }

    #[test]
    fn test_brightness_sorted() {
        let sorted = BrightnessSortedPalette::new();
        let dark_range = sorted.get_range(0, 50);
        let light_range = sorted.get_range(200, 255);
        assert!(!dark_range.is_empty());
        assert!(!light_range.is_empty());
    }

    #[test]
    fn test_palette_manager() {
        let manager = PaletteManager::new();
        assert_eq!(manager.palette().len(), 256);

        let color = Color::new(255, 128, 64, 32);
        let closest = manager.find_closest(color);
        assert!(closest < 256);
    }

    #[test]
    fn test_blend() {
        let color1 = Color::new(255, 255, 0, 0);  // 红色
        let color2 = Color::new(255, 0, 255, 0);  // 绿色
        let blended = color1.blend(color2, 128);
        // 由于整数除法：
        // R: (255 * 127 + 0 * 128) / 255 = 32385 / 255 = 127
        // G: (0 * 127 + 255 * 128) / 255 = 32640 / 255 = 128
        // B: (0 * 127 + 0 * 128) / 255 = 0
        assert_eq!(blended.r, 127);
        assert_eq!(blended.g, 128);
        assert_eq!(blended.b, 0);
    }

    #[test]
    fn test_format_hex() {
        let red = Color::new(255, 255, 0, 0);
        assert_eq!(red.to_hex_string(false), "#FF0000");
        assert_eq!(red.to_hex_string(true), "#FF0000FF");

        let semi_transparent = Color::new(128, 255, 0, 0);
        assert_eq!(semi_transparent.to_hex_string(false), "#FF0000");
        assert_eq!(semi_transparent.to_hex_string(true), "#FF000080");
    }

    #[test]
    fn test_format_rgb_string() {
        let color = Color::new(255, 128, 64, 32);
        assert_eq!(color.to_rgb_string(), "rgb(128, 64, 32)");
        assert_eq!(color.to_rgba_string(), "rgba(128, 64, 32, 255)");
    }

    #[test]
    fn test_format_css_string() {
        let opaque = Color::new(255, 255, 0, 0);
        assert_eq!(opaque.to_css_string(), "#FF0000");

        let transparent = Color::new(128, 255, 0, 0);
        assert_eq!(transparent.to_css_string(), "rgba(255, 0, 0, 128)");
    }

    #[test]
    fn test_display() {
        let red = Color::new(255, 255, 0, 0);
        assert_eq!(format!("{}", red), "Color(#FF0000)");

        let semi_transparent = Color::new(128, 255, 0, 0);
        assert_eq!(format!("{}", semi_transparent), "Color(#FF000080)");
    }

    #[test]
    fn test_lower_hex() {
        let color = Color::new(255, 255, 0, 0);
        assert_eq!(format!("{:x}", color), "ff0000");

        let transparent = Color::new(128, 255, 0, 0);
        assert_eq!(format!("{:x}", transparent), "ff000080");
    }

    #[test]
    fn test_upper_hex() {
        let color = Color::new(255, 255, 0, 0);
        assert_eq!(format!("{:X}", color), "FF0000");

        let transparent = Color::new(128, 255, 0, 0);
        assert_eq!(format!("{:X}", transparent), "FF000080");
    }
}
