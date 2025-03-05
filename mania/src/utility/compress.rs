pub mod gzip {
    use std::io::{Read, Write};

    use flate2::{Compression, read::GzDecoder, write::GzEncoder};

    pub fn compress(data: &[u8]) -> Vec<u8> {
        let mut compressed = Vec::new();
        let mut encoder = GzEncoder::new(&mut compressed, Compression::default());
        encoder
            .write_all(data)
            .expect("Failed to write data to encoder");
        encoder.finish().expect("Failed to finish compression");
        compressed
    }

    pub fn decompress(data: &[u8]) -> Option<Vec<u8>> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).ok()?;
        Some(decompressed)
    }
}

pub mod zlib {
    use std::io::{Read, Write};

    use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};

    pub fn compress(data: &[u8]) -> Vec<u8> {
        let mut compressed = Vec::new();
        let mut encoder = ZlibEncoder::new(&mut compressed, Compression::default());
        encoder
            .write_all(data)
            .expect("Failed to write data to encoder");
        encoder.finish().expect("Failed to finish compression");
        compressed
    }

    pub fn decompress(data: &[u8]) -> Option<Vec<u8>> {
        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).ok()?;
        Some(decompressed)
    }
}
