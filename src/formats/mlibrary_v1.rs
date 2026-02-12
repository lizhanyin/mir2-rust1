//! MLibrary V1 格式解析 (.wzl/.wzx)
//! 这是传奇2使用的库文件格式

use crate::error::{Result, LibraryError};
use crate::image::{MImage, Color, DEFAULT_PALETTE};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write, Seek, SeekFrom};
use std::path::Path;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use flate2::read::ZlibDecoder;
// use std::io::Cursor;

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
        let wzl_path = format!("{}.wzl", self.file_name);
        let file = File::open(&wzl_path)?;
        let mut reader = BufReader::new(file);

        let offset = self.index_list[index] as u64;
        reader.seek(SeekFrom::Start(offset))?;

        let image = self.read_mimage(&mut reader, offset)?;
        self.images[index] = Some(image);

        Ok(())
    }

    /// 读取 MImage 数据
    fn read_mimage(&self, reader: &mut BufReader<File>, offset: u64) -> Result<MImage> {
        reader.seek(SeekFrom::Start(offset))?;

        // 读取头部信息
        let flag = reader.read_u8()?;
        let is_16bit = flag == 5;

        // 跳过 3 字节
        reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        let width = reader.read_i16::<LittleEndian>()?;
        let height = reader.read_i16::<LittleEndian>()?;
        let x = reader.read_i16::<LittleEndian>()?;
        let y = reader.read_i16::<LittleEndian>()?;
        let n_size = reader.read_i32::<LittleEndian>()?;

        if width * height < 4 {
            return Ok(MImage::new());
        }

        // 读取图像数据
        let bytes = if n_size == 0 {
            // 未压缩
            let size = if is_16bit {
                (width * height * 2) as usize
            } else {
                (width * height) as usize
            };
            let mut buf = vec![0u8; size];
            reader.read_exact(&mut buf)?;
            buf
        } else {
            // Zlib 压缩
            // let compressed = vec![0u8; n_size as usize];
            // let compressed_reader = Cursor::new(compressed);

            // 先读取压缩数据到临时缓冲区
            reader.seek(SeekFrom::Start(offset + 16))?;
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

        // 转换像素数据
        img.create_texture(&bytes)?;

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
        // let wzx_path = format!("{}.wzx", self.file_name);

        // 计算偏移量
        let mut offset = 8 + (self.images.len() * 4) as u32;
        let mut index_list: Vec<u32> = Vec::new();

        // 写入数据到内存
        let mut data = Vec::new();

        for image in &self.images {
            if let Some(img) = image {
                index_list.push(offset);
                self.write_mimage_data(img, &mut data)?;
                offset += data.len() as u32;
            }
        }

        // 写入 .wzl 文件
        {
            let file = File::create(&wzl_path)?;
            let mut writer = BufWriter::new(file);

            writer.write_u32::<LittleEndian>(Self::LIB_VERSION as u32)?;
            writer.write_u32::<LittleEndian>(self.images.len() as u32)?;

            for index in &index_list {
                writer.write_u32::<LittleEndian>(*index)?;
            }

            writer.write_all(&data)?;
            writer.flush()?;
        }

        Ok(())
    }

    /// 写入 MImage 数据
    fn write_mimage_data(&self, image: &MImage, writer: &mut Vec<u8>) -> Result<()> {
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

        writer.write_u32::<LittleEndian>(image.fbytes.len() as u32)?;
        writer.write_all(&image.fbytes)?;

        if image.has_mask {
            writer.write_i16::<LittleEndian>(image.mask_width)?;
            writer.write_i16::<LittleEndian>(image.mask_height)?;
            writer.write_i16::<LittleEndian>(image.mask_x)?;
            writer.write_i16::<LittleEndian>(image.mask_y)?;
            writer.write_u32::<LittleEndian>(image.mask_fbytes.len() as u32)?;
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
        let lib = MLibraryV1::new("test".to_string());
        assert!(lib.is_err()); // 文件不存在
    }
}
