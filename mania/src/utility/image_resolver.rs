use byteorder::{BigEndian, ByteOrder, LittleEndian};
use num_enum::TryFromPrimitive;
use std::fmt::Display;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, SeekFrom};

#[derive(Debug, Eq, PartialEq, Default, TryFromPrimitive)]
#[repr(u32)]
pub enum ImageFormat {
    #[default]
    Unknown = 0,
    Png = 1001,
    Jpeg = 1000,
    Gif = 2000,
    Webp = 1002,
    Bmp = 1005,
    Tiff,
}

impl Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageFormat::Unknown => write!(f, "jpg"),
            ImageFormat::Png => write!(f, "png"),
            ImageFormat::Jpeg => write!(f, "jpeg"),
            ImageFormat::Gif => write!(f, "gif"),
            ImageFormat::Webp => write!(f, "webp"),
            ImageFormat::Bmp => write!(f, "bmp"),
            ImageFormat::Tiff => write!(f, "tiff"),
        }
    }
}

#[derive(Debug, Error)]
pub enum ImageResolverError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unsupported or invalid image format")]
    Unsupported,
}

pub async fn resolve_image_metadata<R>(
    reader: &mut R,
) -> Result<(ImageFormat, u32, u32), ImageResolverError>
where
    R: AsyncRead + AsyncSeek + Unpin + ?Sized,
{
    let current_pos = reader.stream_position().await?;
    reader.seek(SeekFrom::Start(0)).await?;

    let mut header = [0u8; 32];
    reader.read_exact(&mut header).await?;

    if header.starts_with(b"GIF89a") || header.starts_with(b"GIF87a") {
        let width = LittleEndian::read_u16(&header[6..8]) as u32;
        let height = LittleEndian::read_u16(&header[8..10]) as u32;
        reader.seek(SeekFrom::Start(current_pos)).await?;
        return Ok((ImageFormat::Gif, width, height));
    }

    if header.starts_with(&[0xFF, 0xD8]) {
        reader.seek(SeekFrom::Start(0)).await?;
        let mut image_data = Vec::new();
        reader.read_to_end(&mut image_data).await?;
        let mut width = 0;
        let mut height = 0;
        if image_data.len() > 2 && image_data[0] == 0xFF && image_data[1] == 0xD8 {
            for i in 2..image_data.len() - 10 {
                if (u16::from_le_bytes([image_data[i], image_data[i + 1]]) & 0xFCFF) == 0xC0FF {
                    width = LittleEndian::read_u16(&image_data[i + 7..i + 9]) as u32;
                    height = LittleEndian::read_u16(&image_data[i + 5..i + 7]) as u32;
                    break;
                }
            }
            reader.seek(SeekFrom::Start(current_pos)).await?;
            return Ok((ImageFormat::Jpeg, width, height));
        }
    }

    if header.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        let width = BigEndian::read_u32(&header[16..20]);
        let height = BigEndian::read_u32(&header[20..24]);
        reader.seek(SeekFrom::Start(current_pos)).await?;
        return Ok((ImageFormat::Png, width, height));
    }

    if header.starts_with(b"RIFF") && &header[8..12] == b"WEBP" {
        let chunk_type = &header[12..16];
        let width: u32;
        let height: u32;
        match chunk_type {
            b"VP8X" => {
                width = (LittleEndian::read_u16(&header[24..27]) + 1) as u32;
                height = (LittleEndian::read_u16(&header[27..30]) + 1) as u32;
            }
            b"VP8L" => {
                width = (LittleEndian::read_u32(&header[21..25]) & 0x3FFF) + 1;
                height = (LittleEndian::read_u32(&header[21..25]) & 0xFFFC000) + 1;
            }
            _ => {
                width = LittleEndian::read_u16(&header[26..28]) as u32;
                height = LittleEndian::read_u16(&header[28..30]) as u32;
            }
        }
        reader.seek(SeekFrom::Start(current_pos)).await?;
        return Ok((ImageFormat::Webp, width, height));
    }

    if header.starts_with(b"BM") {
        let width = LittleEndian::read_u16(&header[18..20]) as u32;
        let height = LittleEndian::read_u16(&header[22..24]) as u32;
        reader.seek(SeekFrom::Start(current_pos)).await?;
        return Ok((ImageFormat::Bmp, width, height));
    }

    if header.starts_with(b"II") || header.starts_with(b"MM") {
        let width = LittleEndian::read_u16(&header[18..20]) as u32;
        let height = LittleEndian::read_u16(&header[30..32]) as u32;
        reader.seek(SeekFrom::Start(current_pos)).await?;
        return Ok((ImageFormat::Tiff, width, height));
    }

    reader.seek(SeekFrom::Start(current_pos)).await?;
    Err(ImageResolverError::Unsupported)
}
