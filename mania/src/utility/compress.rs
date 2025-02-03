pub mod gzip {
    use flate2::read::GzDecoder;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::{Read, Write};

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
    use flate2::read::ZlibDecoder;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::{Read, Write};

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
